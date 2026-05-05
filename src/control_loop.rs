use std::{sync::Arc, time::Duration};

use log::{debug, info};
use rocket::{futures::future::join, tokio::{self, join, sync::Mutex}};
use serde_json::Value;

use crate::rotator::Rotator;

struct ControlInfo {
    rocket_position: RocketPosition,
    rotator_position: RotatorPosition,
}

struct RocketPosition {
    latitude: f32,
    longitude: f32,
    altitude: f32,
}

struct RotatorPosition {
    altitude: f32,
    azimuth: f32,
}

pub async fn rotator_control_loop(rotator: Arc<Mutex<Rotator>>) {
    info!("Started control loop");

    let mut ticker = tokio::time::interval(Duration::from_millis(500));
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

pub async fn telemetry_recieve() {
    let rfd_loop = tokio::spawn(rfd_recieve_loop());
    let taisync_loop = tokio::spawn(taisync_recieve_loop());


    let _ = join!(rfd_loop, taisync_loop);
}

pub async fn rfd_recieve_loop() {
    info!("Started RFD-900x recieve loop");

    let mut ticker = tokio::time::interval(Duration::from_millis(500));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {

        ticker.tick().await;
    }
}

pub async fn taisync_recieve_loop() {
    info!("Started Taisync UDP recieve loop");

    let mut ticker = tokio::time::interval(Duration::from_millis(500));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {

        ticker.tick().await;
    }
}
