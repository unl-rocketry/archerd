use std::io;

use rocket::{State, get, routes, tokio::sync::Mutex};
use serde_json::json;

use crate::{
    response::{Error, Success},
    rotator::{Rotator, dummyport::DummyPort},
};

pub mod response;
pub mod rotator;

#[rocket::main]
async fn main() {
    let rocket_config = rocket::Config {
        ..Default::default()
    };

    let serial = serialport::new("/dev/ttyUSB0", 115_200)
        .open()
        .unwrap_or(Box::new(DummyPort::default()));
    let rotator = Mutex::new(Rotator::new(serial).unwrap());

    let rocket = rocket::build()
        .manage(rotator)
        .mount("/", routes![index, get_serialports, set_rotator_port,])
        .mount("/rotator", rotator::endpoints::endpoints())
        .configure(rocket_config)
        .launch()
        .await;

    rocket.expect("Server failed to shutdown gracefully");
}

#[get("/")]
fn index() -> &'static str {
    "The server is running!"
}

#[get("/get_serial_ports")]
async fn get_serialports() -> Result<Success, Error> {
    let ports: Vec<_> = serialport::available_ports()
        .map_err(|e| io::Error::other(e.to_string()))?
        .iter()
        .cloned()
        .map(|sp| sp.port_name)
        .collect();

    Ok(Success::data(json!({
        "ports": ports,
    })))
}

#[get("/set_rotator_port?<port>")]
async fn set_rotator_port(
    rotator_state: &State<Mutex<Rotator>>,
    port: Option<String>,
) -> Result<Success, Error> {
    let rotator_port = match port {
        Some(p) => serialport::new(p, Rotator::BAUD),
        None => todo!(),
    }
    .open()
    .map_err(|e| io::Error::other(e.to_string()))?;

    let rotator = Rotator::new(rotator_port)?;

    *rotator_state.lock().await = rotator;

    Ok(Success::empty())
}
