use std::{io::Error, num::ParseFloatError, ops::Neg, str::ParseBoolError};

use rocket::{
    State, get, routes, serde::json::Value, tokio::sync::Mutex
};

use crate::rotator::{Command, Direction, Rotator};

pub mod rotator;

#[rocket::main]
async fn main() {
    let rocket_config = rocket::Config {
        ..Default::default()
    };

    let serial = serialport::new("/dev/ttyUSB0", 115_200).open().unwrap();
    let mut rotator = Rotator::new(serial).unwrap();

    let cmd_string = rotator.send_command(Command::GetVersion, &[]).unwrap();
    let version = rotator.validate_parse(&cmd_string).unwrap()
        .ok_or(Error::other("ExpectedValue"))
        .map(|v| v[0].clone()).unwrap();
    let protocol_version = env!("PROTOCOL_VERSION");
    if !(protocol_version == protocol_version) {
        log::warn!("Protocol Version Mismatch please use a version of this program compatible with protocol Version {version}")
    }

    let rotator = Mutex::new(rotator);

    let rocket = rocket::build()
        .manage(rotator)
        .mount(
            "/",
            routes![
                index,
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
            ],
        )
        .configure(rocket_config)
        .launch()
        .await;

    rocket.expect("Server failed to shutdown gracefully");
}

#[get("/")]
fn index() -> &'static str {
    "The server is running!"
}

type StatePort = State<Mutex<Rotator>>;

/// Set a defined position for the rotator to move tow
#[get("/dver?<degrees>")]
async fn set_position_vertical(serial: &StatePort, degrees: f32) -> Result<(), std::io::Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = rotator.send_command(Command::DegreesVertical, &[&format!("{:0.3}", degrees)])?;
    rotator.validate_parse(&cmd_string)?;

    Ok(())
}

/// Set a defined position for the rotator in the horizontal axis.
#[get("/dhor?<degrees>")]
async fn set_position_horizontal(serial: &StatePort, degrees: f32) -> Result<(), std::io::Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = rotator.send_command(Command::DegreesHorizontal, &[&format!("{:0.3}", degrees.neg())])?;
    rotator.validate_parse(&cmd_string)?;

    Ok(())
}

/// Calibrates the vertical axis.
#[get("/calv?<set>")]
async fn calibrate_vertical(serial: &StatePort, set: bool) -> Result<(), std::io::Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = if set {
        rotator.send_command(Command::CalibrateVertical, &["SET"])?
    } else {
        rotator.send_command(Command::CalibrateVertical, &[])?
    };

    rotator.validate_parse(&cmd_string)?;
    Ok(())
}

/// Calibrates the horizontal axis.
#[get("/calh")]
async fn calibrate_horizontal(serial: &StatePort) -> Result<(), Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = rotator.send_command(Command::CalibrateHorizontal, &[])?;
    rotator.validate_parse(&cmd_string)?;
    Ok(())
}

/// Moves in a direction indefinitely specified by the command, or stops, if the command is to stop.
#[get("/movc?<direction>")]
async fn move_direction(serial: &StatePort, direction: Direction) -> Result<(), Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = rotator.send_command(Command::CalibrateHorizontal, &[&direction.to_string()])?;
    rotator.validate_parse(&cmd_string)?;
    Ok(())
}

/// Moves by the specified number of steps in the vertical axis.
#[get("/movv?<steps>")]
async fn move_vertical_steps(serial: &StatePort, steps: i32) -> Result<(), Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = rotator.send_command(Command::MoveVerticalSteps, &[&steps.to_string()])?;
    rotator.validate_parse(&cmd_string)?;
    Ok(())
}

/// Moves by the specified number of steps in the horizontal axis.
#[get("/movh?<steps>")]
async fn move_horizontal_steps(serial: &StatePort, steps: i32) -> Result<(), Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = rotator.send_command(Command::MoveHorizontalSteps, &[&steps.to_string()])?;
    rotator.validate_parse(&cmd_string)?;
    Ok(())
}

#[get("/position")]
/// Gets the current position for both the vertical and horizontal axes.
async fn position(serial: &StatePort) -> Result<Value, Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = rotator.send_command(Command::GetPosition, &[])?;
    let value_list = rotator.validate_parse(&cmd_string)?
            .ok_or(Error::other("ExpectedValue"))?;

    if value_list.len() != 2 {
        return Err(Error::other("InvalidResponse"))
    }

    let (v, h) = (
        value_list[0].parse::<f32>().map_err(|e: ParseFloatError| Error::other(e.to_string()))?,
        value_list[1].parse::<f32>().map_err(|e: ParseFloatError| Error::other(e.to_string()))?
    );

    Ok(rocket::serde::json::json!({
        "vertical": v,
        "horizontal": h,
    }))
}

/// Gets the calibration status of the rotator. This must be true to use
/// `set_position_vertical` and `set_position_horizontal`.
#[get("/calibrated")]
async fn calibrated(serial: &StatePort) -> Result<Value, Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = rotator.send_command(Command::GetCalibrated, &[])?;

    let value_list = rotator.validate_parse(&cmd_string)?
        .ok_or(Error::other("ExpectedValue"))?;

    let calibrated = value_list[0].parse::<bool>()
        .map_err(|e: ParseBoolError| Error::other(e.to_string()))?;

    Ok(rocket::serde::json::json!({
        "calibrated": calibrated
    }))
}

/// Immediately stops both motors by locking them to perform an emergency stop.
#[get("/halt")]
async fn halt(serial: &StatePort) -> Result<(), Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = rotator.send_command(Command::Halt, &[])?;
    rotator.validate_parse(&cmd_string)?;

    Ok(())
}

/// Gets the current version of the software on the rotator.
#[get("/version")]
async fn version(serial: &StatePort) -> Result<String, Error> {
    let mut rotator = serial.lock().await;

    let cmd_string = rotator.send_command(Command::GetVersion, &[])?;

    rotator.validate_parse(&cmd_string)?
        .ok_or(Error::other("ExpectedValue"))
        .map(|v| v[0].clone())
}
