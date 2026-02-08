use rocket::{
    get, routes
};

pub mod rotator;

#[rocket::main]
async fn main() {
    let rocket_config = rocket::Config {
        ..Default::default()
    };

    let rocket = rocket::build()
        .mount(
            "/",
            routes![
                index
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
