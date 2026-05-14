use std::{io, sync::Arc};

use rocket::{State, get, routes, tokio::{self, sync::Mutex}};
use serde_json::json;
use serialport::SerialPort;

use crate::{
    control_loop::rotator_control_loop, response::{Error, Success}, rotator::{Rotator, dummyport::DummyPort}
};

mod response;
mod rotator;
mod control_loop;

#[rocket::main]
async fn main() {
    let rocket_config = rocket::Config {
        address: [0, 0, 0, 0].into(),
        ..Default::default()
    };

    let serial = serialport::new("/dev/ttyUSB0", 115_200)
        .open()
        .unwrap_or(Box::new(DummyPort::default()));
    let rotator = Arc::new(Mutex::new(Rotator::new(serial).unwrap()));

    let loop_rotator = Arc::clone(&rotator);
    tokio::spawn(rotator_control_loop(loop_rotator));

    let rocket = rocket::build()
        .manage(rotator)
        .mount("/", routes![index, get_serialports, get_rotator_port, set_rotator_port,])
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
    let ports = serialport::available_ports()
        .map_err(|e| io::Error::other(e.to_string()))?
        .to_vec();

    Ok(Success::data(json!({
        "ports": ports,
    })))
}

#[get("/get_rotator_port")]
async fn get_rotator_port(rotator_state: &State<Arc<Mutex<Rotator>>>) -> Result<Success, Error> {
    let rotator = rotator_state.lock().await;
    let port = rotator.port().name();

    Ok(Success::data(json!({
        "port": port,
    })))
}

#[get("/set_rotator_port?<port>")]
async fn set_rotator_port(
    rotator_state: &State<Arc<Mutex<Rotator>>>,
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

async fn autofind_serial_port(vid: u16, pid: u16, baud: u32) -> Result<Box<dyn SerialPort>, Box<dyn std::error::Error>> {
    for port in serialport::available_ports()? {
        if let serialport::SerialPortType::UsbPort(u) = port.port_type && (u.vid == vid && u.pid == pid) {
            let port = serialport::new(port.port_name, baud)
                .open()?;

            return Ok(port);
        }
    }

    Err("Failed to find device".into())
}


