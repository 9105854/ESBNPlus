mod auth;
mod search;
mod utils;

#[macro_use]
extern crate rocket;
use std::time::Duration;

use auth::{
    already_auth_login, already_auth_signup, login, login_ui, logout, logout_ui, logout_ui_no_auth,
    signup, signup_ui,
};
use reqwest::header;
use rocket::fs::FileServer;
use rocket::http::CookieJar;
use rocket_dyn_templates::{context, Template};
use search::simple_search;
use serde::Deserialize;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use utils::AppError;

pub struct SqliteState {
    pool: SqlitePool,
}
#[get("/")]
fn index(cookies: &CookieJar<'_>) -> Result<Template, AppError> {
    let user_id = cookies.get_private("user_id").map(|e| {
        let value = e.value_trimmed();
        value.to_string()
    });

    Ok(Template::render("index", context![user_id]))
}
#[derive(Deserialize)]
struct IGDBAuth {
    access_token: String,
    expires_in: u64,
    token_type: String,
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
    dotenvy::dotenv().expect("Couldn't load .env file");
    let client_id = std::env::var("CLIENT_ID").expect("Couldn't find Client ID");
    let client_secret = std::env::var("CLIENT_SECRET").expect("Couldn't find Client Secret");
    let assets_server = FileServer::from("assets");
    let client = reqwest::ClientBuilder::new().build().unwrap();
    let response = client
        .post("https://id.twitch.tv/oauth2/token")
        .query(&[
            ("client_id", &client_id),
            ("client_secret", &client_secret),
            ("grant_type", &"client_credentials".to_string()),
        ])
        .send()
        .await
        .expect("Wasn't able to retrieve access token")
        .json::<IGDBAuth>()
        .await
        .expect("Couldn't parse access response");
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Client-ID",
        header::HeaderValue::from_str(&client_id).unwrap(),
    );
    let auth_header = format!("Bearer {}", response.access_token);
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&auth_header).unwrap(),
    );
    info!("{:?}", headers.values());
    let client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()
        .unwrap();
    let routes = routes![
        index,
        already_auth_login,
        login_ui,
        login,
        already_auth_signup,
        signup_ui,
        signup,
        logout_ui,
        logout,
        logout_ui_no_auth,
        simple_search
    ];
    rocket::build()
        .mount("/", routes)
        .mount("/assets", assets_server)
        .manage(SqliteState { pool })
        .manage(client)
        .attach(Template::fairing())
}
