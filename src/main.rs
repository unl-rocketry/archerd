use std::{io, sync::Arc};

use aerospace_rocketry_lib::{geospatial::Point, utils::crc::crc8};
use rocket::{State, get, routes, tokio::{self, sync::Mutex}};
use serde_json::{json, Value};
use serialport::SerialPort;
use num_traits::FromPrimitive;

use num_derive::{FromPrimitive, ToPrimitive};

use crate::{
    control_loop::{ControlInfo, rfd_receive_loop, rotator_control_loop}, response::{Error, Success}, rotator::{Rotator, dummyport::DummyPort}
};

mod response;
mod rotator;
mod control_loop;

const ROTATOR_SERIAL_USB: (u16, u16) = (0x10C4, 0xEA60);
const RFD_SERIAL_USB: (u16, u16) = (0x0403, 0x6001);


#[derive(FromPrimitive, ToPrimitive)]
#[repr(u8)]
#[non_exhaustive]
pub enum Commands {
    /// Enable the Taisync radio
    EnableHighPower = 70,
    /// Disable the Taisync radio
    DisableHighPower = 80,

    /// Forcibly reboot without waiting for any processes to finish
    Reboot = 100,
    /// Restart the stream process
    RestartStream = 101,
    /// Get the IP address
    GetIpAddress = 102,
}

#[rocket::main]
async fn main() {
    env_logger::init();

    let rocket_config = rocket::Config {
        address: [0, 0, 0, 0].into(),
        ..Default::default()
    };

    let rotator_serial = autofind_serial_port(ROTATOR_SERIAL_USB.0, ROTATOR_SERIAL_USB.1, 115_200)
        .await
        .unwrap_or_else(|_| Box::new(DummyPort::default()));

    dbg!(&rotator_serial);

    let rotator = Arc::new(Mutex::new(Rotator::new(rotator_serial).unwrap()));

    let version = rotator.lock().await.version().await.unwrap_or_else(|_| "0.0.0".to_string());
    let protocol_version = env!("PROTOCOL_VERSION");
    if !(protocol_version == version) {
        println!("Protocol Version Mismatch please use a version of this program compatible with protocol Version {version}");
    }

    let rotator_position = Arc::new(Mutex::new(None));
    let rocket_position = Arc::new(Mutex::new(None));

    let last_packet = Arc::new(Mutex::new(None));
    let loop_last_packet = Arc::clone(&last_packet);

    let rfd = Arc::new(Mutex::new(autofind_serial_port(RFD_SERIAL_USB.0, RFD_SERIAL_USB.1, 57_600).await.ok()));
    dbg!(&rfd);

    // Spawn RFD receiving loop
    {
        let loop_rocket_position = Arc::clone(&rocket_position);
        let loop_rfd = Arc::clone(&rfd);

        tokio::spawn(rfd_receive_loop(loop_rfd, loop_rocket_position, loop_last_packet));
    }

    // Spawn Rotator control loop
    {
        let control_info = ControlInfo { rocket_position, rotator_position: Arc::clone(&rotator_position) };
        let loop_rotator = Arc::clone(&rotator);

        tokio::spawn(rotator_control_loop(loop_rotator, control_info));
    }

    let rocket = rocket::build()
        .manage(rotator)
        .manage(rotator_position)
        .manage(rfd)
        .manage(last_packet)
        .mount("/", routes![index, get_serialports, get_rotator_port, set_rotator_port, set_rotator_position, get_rotator_position, send_rfd_command, get_last_packet])
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

#[get("/set_rotator_position?<lon>&<lat>&<alt>")]
async fn set_rotator_position(
    rotator_position: &State<Arc<Mutex<Option<Point>>>>,
    lon: f64,
    lat: f64,
    alt: f64,
) {
    *rotator_position.lock().await = Some(Point::new_3d(
        lat,
        lon,
        alt,
    ).unwrap());
}

#[get("/get_rotator_position")]
async fn get_rotator_position(
    rotator_position: &State<Arc<Mutex<Option<Point>>>>,
) -> Result<serde_json::Value, Error> {
    let Some(pos) = *rotator_position.lock().await else {
        return Err(Error("Position not set".to_string()))
    };

    Ok(json!({
        "latitude": pos.latitude(),
        "longitude": pos.longitude(),
        "altitude": pos.altitude(),
    }))
}

/// Enable high power (turn on Taisync): 70
/// Disable high power (turn off Taisync): 80
/// Reboot: 100
/// Restart streaming service: 101
/// Get IP address: 102
#[get("/rfd/send?<cmd>")]
async fn send_rfd_command(
    rfd_state: &State<Arc<Mutex<Option<Box<dyn SerialPort>>>>>,
    cmd: u8,
) -> Result<Success, Error> {
    let mut rfd_lock = rfd_state.lock().await;

    let Some(rfd) = rfd_lock.as_mut() else {
        return Err(Error("RFD not connected".into()));
    };

    let crc = crc8(&[cmd]);
    rfd.write_all(&[cmd, crc, b' '])?;

    Ok(Success::empty())
}

#[get("/rfd/last_packet")]
async fn get_last_packet(last_packet: &State<Arc<Mutex<Option<Value>>>>) -> Result<serde_json::Value, Error> {
    last_packet.lock().await.clone().ok_or(Error("Could not find last packet".into()))
}
