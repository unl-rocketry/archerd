use std::{io, num::ParseFloatError, ops::Neg as _, str::ParseBoolError};

use rocket::{State, get, response::Responder, serde::json::Value, tokio::sync::Mutex};
use serde::Serialize;
use serde_json::json;

use super::{Command, Rotator};

#[derive(Serialize)]
struct InnerResponse {
    message: String,
    data: Option<Value>,
}

#[derive(Responder)]
#[response(status = 200, content_type = "json")]
pub struct Success(String);

#[derive(Responder)]
#[response(status = 500, content_type = "json")]
pub struct Error(String);

impl Success {
    fn empty() -> Self {
        Self(serde_json::ser::to_string(&InnerResponse {
            message: "success".to_string(),
            data: None,
        }).unwrap())
    }

    fn data(data: Value) -> Self {
        Self(serde_json::ser::to_string(&InnerResponse {
            message: "success".to_string(),
            data: Some(data),
        }).unwrap())
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self(serde_json::ser::to_string(&InnerResponse {
            message: value.to_string(),
            data: None,
        }).unwrap())
    }
}

type StatePort = State<Mutex<Rotator>>;

/// Set a defined position for the rotator to move tow
#[get("/dver?<degrees>")]
pub async fn set_position_vertical(serial: &StatePort, degrees: f32) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = rotator.send_command(Command::DegreesVertical, &[&format!("{degrees:0.3}")])?;
    rotator.validate_parse(&cmd_string)?;
    drop(rotator);

    Ok(Success::empty())
}

/// Set a defined position for the rotator in the horizontal axis.
#[get("/dhor?<degrees>")]
pub async fn set_position_horizontal(serial: &StatePort, degrees: f32) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = rotator.send_command(Command::DegreesHorizontal, &[&format!("{:0.3}", degrees.neg())])?;
    rotator.validate_parse(&cmd_string)?;
    drop(rotator);

    Ok(Success::empty())
}

/// Calibrates the vertical axis.
#[get("/calv?<set>")]
pub async fn calibrate_vertical(serial: &StatePort, set: bool) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = if set {
        rotator.send_command(Command::CalibrateVertical, &["SET"])?
    } else {
        rotator.send_command(Command::CalibrateVertical, &[])?
    };

    rotator.validate_parse(&cmd_string)?;
    drop(rotator);

    Ok(Success::empty())
}

/// Calibrates the horizontal axis.
#[get("/calh")]
pub async fn calibrate_horizontal(serial: &StatePort) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = rotator.send_command(Command::CalibrateHorizontal, &[])?;
    rotator.validate_parse(&cmd_string)?;
    drop(rotator);

    Ok(Success::empty())
}

/// Moves in a direction indefinitely specified by the command, or stops, if the command is to stop.
#[get("/movc?<direction>")]
pub async fn move_direction(serial: &StatePort, direction: super::Direction) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = rotator.send_command(Command::CalibrateHorizontal, &[&direction.to_string()])?;
    rotator.validate_parse(&cmd_string)?;
    drop(rotator);

    Ok(Success::empty())
}

/// Moves by the specified number of steps in the vertical axis.
#[get("/movv?<steps>")]
pub async fn move_vertical_steps(serial: &StatePort, steps: i32) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = rotator.send_command(Command::MoveVerticalSteps, &[&steps.to_string()])?;
    rotator.validate_parse(&cmd_string)?;
    drop(rotator);

    Ok(Success::empty())
}

/// Moves by the specified number of steps in the horizontal axis.
#[get("/movh?<steps>")]
pub async fn move_horizontal_steps(serial: &StatePort, steps: i32) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = rotator.send_command(Command::MoveHorizontalSteps, &[&steps.to_string()])?;
    rotator.validate_parse(&cmd_string)?;
    drop(rotator);

    Ok(Success::empty())
}

#[get("/position")]
/// Gets the current position for both the vertical and horizontal axes.
pub async fn position(serial: &StatePort) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = rotator.send_command(Command::GetPosition, &[])?;
    let value_list = rotator.validate_parse(&cmd_string)?
        .ok_or_else(|| io::Error::other("ExpectedValue"))?;
    drop(rotator);

    if value_list.len() != 2 {
        Err(io::Error::other("InvalidResponse"))?
    }

    let (v, h) = (
        value_list[0].parse::<f32>().map_err(|e: ParseFloatError| io::Error::other(e.to_string()))?,
        value_list[1].parse::<f32>().map_err(|e: ParseFloatError| io::Error::other(e.to_string()))?
    );

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

    let cmd_string = rotator.send_command(Command::GetCalibrated, &[])?;

    let value_list = rotator.validate_parse(&cmd_string)?
        .ok_or_else(|| io::Error::other("ExpectedValue"))?;
    drop(rotator);

    let calibrated = value_list[0].parse::<bool>()
        .map_err(|e: ParseBoolError| io::Error::other(e.to_string()))?;

    Ok(Success::data(json!({
        "calibrated": calibrated
    })))
}

/// Immediately stops both motors by locking them to perform an emergency stop.
#[get("/halt")]
pub async fn halt(serial: &StatePort) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = rotator.send_command(Command::Halt, &[])?;
    rotator.validate_parse(&cmd_string)?;
    drop(rotator);

    Ok(Success::empty())
}

/// Gets the current version of the software on the rotator.
#[get("/version")]
pub async fn version(serial: &StatePort) -> Result<Success, Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = rotator.send_command(Command::GetVersion, &[])?;

    Ok(rotator.validate_parse(&cmd_string)?
        .ok_or_else(|| io::Error::other("ExpectedValue"))
        .map(|v| Success::data(json!({
            "version": v[0].clone()
        })))?)
}
