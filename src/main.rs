#[macro_use]
extern crate rocket;
use std::time::Duration;

use log;
use rocket::response::Responder;
use rocket_dyn_templates::{context, Template};
use sqlx::{sqlite::SqlitePoolOptions, SqliteConnection, SqlitePool};
struct AppError(anyhow::Error);
#[get("/")]
fn index() -> Template {
    Template::render("index", context![])
}
#[launch]
async fn rocket() -> _ {
    env_logger::init();
    log::info!("Starting");
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect("sqlite:data.db")
        .await
        .expect("Couldn't connect to db");
    rocket::build()
        .mount("/", routes![index])
        .manage(pool)
        .attach(Template::fairing())
}
