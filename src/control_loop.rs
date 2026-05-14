use std::{sync::Arc, time::Duration};

use aerospace_rocketry_lib::geospatial::Point;
use log::info;
use rocket::tokio::{self, join, sync::Mutex, time::Instant};
use serde_json::Value;
use serialport::SerialPort;

use crate::rotator::Rotator;

pub struct ControlInfo {
    pub rocket_position: Arc<Mutex<RocketPosition>>,
    pub rotator_position: Arc<Mutex<RotatorPosition>>,
}

pub struct RocketPosition {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}

pub struct RotatorPosition {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64
}

pub async fn rotator_control_loop(rotator: Arc<Mutex<Rotator>>, control_info: ControlInfo) {
    info!("Started control loop");

    let mut ticker = tokio::time::interval(Duration::from_millis(200));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        let ground = Point::new_3d(
            control_info.rotator_position.lock().await.latitude,
            control_info.rotator_position.lock().await.longitude,
            control_info.rotator_position.lock().await.altitude
        ).unwrap();
        let rocket = Point::new_3d(
            control_info.rocket_position.lock().await.latitude,
            control_info.rocket_position.lock().await.longitude,
            control_info.rocket_position.lock().await.altitude
        ).unwrap();

        let bearing = ground.bearing_to(rocket, false);
        let elevation = ground.elevation_to(rocket).unwrap();

        rotator.lock().await.set_position_vertical(elevation as f32).await.unwrap();
        rotator.lock().await.set_position_horizontal(bearing.degrees() as f32).await.unwrap();

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
            altitude: packet["p_alt"].as_f64().unwrap(),
            latitude: packet["gps"]["latitude"].as_f64().unwrap(),
            longitude: packet["gps"]["longitude"].as_f64().unwrap(),
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
