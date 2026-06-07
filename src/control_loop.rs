use std::{sync::Arc, time::Duration};

use aerospace_rocketry_lib::{geospatial::Point, utils::crc8};
use log::{debug, info, warn};
use rocket::tokio::{self, sync::Mutex};
use serde_json::Value;
use serialport::SerialPort;

use crate::rotator::Rotator;

pub struct ControlInfo {
    pub rocket_position: Arc<Mutex<Option<Point>>>,
    pub rotator_position: Arc<Mutex<Option<Point>>>,
}

pub async fn rotator_control_loop(rotator: Arc<Mutex<Rotator>>, control_info: ControlInfo) {
    info!("Started control loop");

    let mut ticker = tokio::time::interval(Duration::from_millis(200));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        ticker.tick().await;

        let ground = if let Some(gp) = control_info.rotator_position.lock().await.as_ref() {
            *gp
        } else {
            continue;
        };

        let rocket = if let Some(rp) = control_info.rocket_position.lock().await.as_ref() {
            *rp
        } else {
            continue;
        };

        let bearing = ground.bearing_to(rocket, false);
        let elevation = ground.elevation_to(rocket).unwrap();

        rotator.lock().await.set_position_vertical(elevation as f32).await.unwrap();
        rotator.lock().await.set_position_horizontal(bearing.degrees() as f32).await.unwrap();
    }
}

pub async fn rfd_receive_loop(mut rfd: Option<Box<dyn SerialPort>>, rocket_position: Arc<Mutex<Option<Point>>>) {
    info!("Started RFD-900x recieve loop");

    let mut leftover_string = String::new();
    let mut buf = [0u8; 4096];
    let mut buf_pos = 0;
    loop {
        let Some(rfd) = rfd.as_mut() else {
            continue;
        };

        let Ok(bytes_read) = rfd.read(&mut buf[buf_pos..]) else {
            continue
        };
        buf_pos += bytes_read;

        let Ok(packet_string) = String::from_utf8(buf[..buf_pos].to_vec()) else {
            continue
        };

        let new_packet_string = if let Some((data, leftover)) = packet_string.split_once('\n') {
            let mut leftover_copy = leftover_string.clone();
            leftover_copy.push_str(data);

            leftover_string.clear();
            leftover_string.push_str(leftover);
            buf_pos = 0;

            leftover_copy
        } else {
            continue;
        };

        let Some((crc, data)) = new_packet_string.split_once(' ') else {
            continue
        };
        let data = data.trim();

        if let Ok(parsed_crc) = crc.parse::<u8>() {
            let new_crc = crc8(data.as_bytes());
            if new_crc == parsed_crc {
                debug!("CRC is valid.");
            } else {
                warn!("CRC is invalid! {parsed_crc} != {new_crc}");
                continue
            }
        }

        let packet: Value = serde_json::from_str(data).unwrap();

        info!("Got packet from air side:\n{packet}");

        let new_position = if let Some(gps) = packet.get("gps") && !gps.is_null() {
             Point::new_3d(
                packet["p_alt"].as_f64().unwrap(),
                gps["latitude"].as_f64().unwrap(),
                gps["longitude"].as_f64().unwrap(),
            ).unwrap()
        } else {
            continue;
        };

        // Set the position of the rocket to the new position
        *rocket_position.lock().await = Some(new_position);
    }
}
