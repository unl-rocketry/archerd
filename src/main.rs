use rocket::{
    get, routes, tokio::sync::Mutex
};

use crate::rotator::{Rotator, endpoints};

pub mod rotator;

#[rocket::main]
async fn main() {
    let rocket_config = rocket::Config {
        ..Default::default()
    };

    let serial = serialport::new("/dev/ttyUSB0", 115_200).open().unwrap();
    let rotator = Mutex::new(Rotator::new(serial).unwrap());

    let rocket = rocket::build()
        .manage(rotator)
        .mount(
            "/",
            routes![
                index,
                endpoints::set_position_vertical,
                endpoints::set_position_horizontal,
                endpoints::calibrate_vertical,
                endpoints::calibrate_horizontal,
                endpoints::move_direction,
                endpoints::move_vertical_steps,
                endpoints::move_horizontal_steps,
                endpoints::position,
                endpoints::calibrated,
                endpoints::halt,
                endpoints::version,
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
