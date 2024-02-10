#[macro_use]
extern crate rocket;
use std::time::Duration;

use rocket::{
    fs::FileServer,
    response::{self, Responder},
    Request,
};
use rocket_dyn_templates::{context, Template};
use sqlx::{sqlite::SqlitePoolOptions, SqliteConnection, SqlitePool};
#[derive(Debug)]
struct AppError(anyhow::Error);
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        AppError(error.into())
    }
}

impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, request: &Request<'_>) -> response::Result<'static> {
        response::Debug(self.0).respond_to(request)
    }
}

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
    let assets_server = FileServer::from("assets");
    rocket::build()
        .mount("/", routes![index])
        .mount("/assets", assets_server)
        .manage(pool)
        .attach(Template::fairing())
}
