//! Rocket endpoints for managing the rotator remotely.

use std::sync::Arc;

use rocket::{Route, State, get, routes, tokio::sync::Mutex};
use serde_json::json;
use crate::response::{Error, Success};

use super::Rotator;

pub fn endpoints() -> Vec<Route> {
    routes![
        set_position_vertical,
        set_position_horizontal,
        calibrate_vertical,
        calibrate_horizontal,
        move_direction,
        move_vertical_steps,
        move_horizontal_steps,
        position,
        calibrated,
        halt,
        version,
    ]
}

type StatePort = State<Arc<Mutex<Rotator>>>;

/// Set a defined position for the rotator to move tow
#[get("/dver?<degrees>")]
pub async fn set_position_vertical(serial: &StatePort, degrees: f32) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;
    rotator.set_position_vertical(degrees).await?;

    Ok(Success::empty())
}

/// Set a defined position for the rotator in the horizontal axis.
#[get("/dhor?<degrees>")]
pub async fn set_position_horizontal(serial: &StatePort, degrees: f32) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;
    rotator.set_position_horizontal(degrees).await?;

    Ok(Success::empty())
}

/// Calibrates the vertical axis.
#[get("/calv?<set>")]
pub async fn calibrate_vertical(serial: &StatePort, set: bool) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;
    let _ = rotator.calibrate_vertical(set).await;

    Ok(Success::empty())
}

/// Calibrates the horizontal axis.
#[get("/calh")]
pub async fn calibrate_horizontal(serial: &StatePort) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;
    rotator.calibrate_horizontal().await?;

    Ok(Success::empty())
}

/// Moves in a direction indefinitely specified by the command, or stops, if the command is to stop.
#[get("/movc?<direction>")]
pub async fn move_direction(
    serial: &StatePort,
    direction: super::Direction,
) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;
    rotator.move_direction(direction).await?;

    Ok(Success::empty())
}

/// Moves by the specified number of steps in the vertical axis.
#[get("/movv?<steps>")]
pub async fn move_vertical_steps(serial: &StatePort, steps: i32) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;
    rotator.move_vertical_steps(steps).await?;

    Ok(Success::empty())
}

/// Moves by the specified number of steps in the horizontal axis.
#[get("/movh?<steps>")]
pub async fn move_horizontal_steps(serial: &StatePort, steps: i32) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;
    rotator.move_horizontal_steps(steps).await?;

    Ok(Success::empty())
}

#[get("/position")]
/// Gets the current position for both the vertical and horizontal axes.
pub async fn position(serial: &StatePort) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;
    let (v, h) = rotator.position().await?;

    Ok(Success::data(json!({
        "vertical": v,
        "horizontal": h,
    })))
}

/// Gets the calibration status of the rotator. This must be true to use
/// `set_position_vertical` and `set_position_horizontal`.
#[get("/calibrated")]
pub async fn calibrated(serial: &StatePort) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;
    let calibrated = rotator.calibrated().await?;

    Ok(Success::data(json!({
        "calibrated": calibrated
    })))
}

/// Immediately stops both motors by locking them to perform an emergency stop.
#[get("/halt")]
pub async fn halt(serial: &StatePort) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;
    rotator.halt().await?;

    Ok(Success::empty())
}

/// Gets the current version of the software on the rotator.
#[get("/version")]
pub async fn version(serial: &StatePort) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;
    let version = rotator.version().await?;

    Ok(Success::data(json!({
        "version": version
    })))
}
