mod auth;
mod utils;

#[macro_use]
extern crate rocket;
use std::time::Duration;

use auth::{login, login_ui, signup, signup_ui};
use rocket::fs::FileServer;
use rocket_dyn_templates::{context, Template};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use utils::AppError;

pub struct SqliteState {
    pool: SqlitePool,
}
#[get("/")]
fn index() -> Result<Template, AppError> {
    Ok(Template::render("index", context![]))
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
    let assets_server = FileServer::from("assets");
    rocket::build()
        .mount("/", routes![index, login_ui, signup_ui, login, signup])
        .mount("/assets", assets_server)
        .manage(SqliteState { pool })
        .attach(Template::fairing())
}
