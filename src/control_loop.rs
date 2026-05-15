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
            gp.clone()
        } else {
            continue;
        };

        let rocket = if let Some(rp) = control_info.rocket_position.lock().await.as_ref() {
            rp.clone()
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

    let mut buf = [0u8; 4096];

    loop {
        let Some(rfd) = rfd.as_mut() else {
            continue;
        };

        let Ok(bytes_read) = rfd.read(&mut buf) else {
            continue
        };
        let Ok(packet_string) = String::from_utf8(buf[..bytes_read].to_vec()) else {
            continue
        };
        let Some((crc, data)) = packet_string.split_once(' ') else {
            continue
        };

        if let Ok(parsed_crc) = crc.parse::<u8>() && crc8(data.as_bytes()) == parsed_crc {
            debug!("CRC is valid.");
        } else {
            warn!("CRC is invalid!");
            continue
        };

        let packet: Value = serde_json::from_str(data).unwrap();
        let new_position = Point::new_3d(
            packet["p_alt"].as_f64().unwrap(),
            packet["gps"]["latitude"].as_f64().unwrap(),
            packet["gps"]["longitude"].as_f64().unwrap(),
        ).unwrap();

        // Set the position of the rocket to the new position
        *rocket_position.lock().await = Some(new_position);
    }
}
