use std::{sync::Arc, time::Duration};

use log::info;
use rocket::tokio::{self, join, sync::Mutex, time::Instant};
use serde_json::Value;
use serialport::SerialPort;

use crate::rotator::Rotator;

struct ControlInfo {
    rocket_position: Arc<Mutex<RocketPosition>>,
    rotator_position: RotatorPosition,
}

pub struct RocketPosition {
    pub latitude: f32,
    pub longitude: f32,
    pub altitude: f32,
}

struct RotatorPosition {
    altitude: f32,
    azimuth: f32,
}

pub async fn rotator_control_loop(rotator: Arc<Mutex<Rotator>>) {
    info!("Started control loop");

    let mut ticker = tokio::time::interval(Duration::from_millis(200));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        let Ok((v, h)) = rotator.lock().await.position().await else {
            ticker.tick().await;
            continue;
        };

        println!("Az: {h}, Alt: {v}");

        ticker.tick().await;
    }
}

// pub async fn telemetry_receive() {
//     let rfd_loop = tokio::spawn(rfd_receive_loop());
//     let taisync_loop = tokio::spawn(taisync_receive_loop());


//     let _ = join!(rfd_loop, taisync_loop);
// }

pub async fn rfd_receive_loop(mut rfd: Box<dyn SerialPort>, rocket_position: Arc<Mutex<RocketPosition>>) {
    info!("Started RFD-900x recieve loop");

    let mut buf = [0u8; 4096];

    loop {
        let Ok(bytes_read) = rfd.read(&mut buf) else {
            continue
        };
        let Ok(packet_string) = String::from_utf8(buf[..bytes_read].to_vec()) else {
            continue
        };
        let Some((crc, data)) = packet_string.split_once(' ') else {
            continue
        };

        let packet: Value = serde_json::from_str(data).unwrap();
        let new_position = RocketPosition {
            altitude: packet["p_alt"].as_f64().unwrap() as f32,
            latitude: packet["gps"]["latitude"].as_f64().unwrap() as f32,
            longitude: packet["gps"]["longitude"].as_f64().unwrap() as f32,
        };

        *rocket_position.lock().await = new_position;
    }
}

pub async fn taisync_receive_loop() {
    info!("Started Taisync UDP recieve loop");

    let mut ticker = tokio::time::interval(Duration::from_millis(500));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {

        ticker.tick().await;
    }
}
