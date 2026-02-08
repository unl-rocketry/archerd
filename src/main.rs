use rocket::{
    get, routes
};

#[get("/")]
fn index() -> &'static str {
    "The server is running!"
}

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
