use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::Deserialize;

use crate::utils::AppError;
#[derive(Deserialize, Debug)]
struct GameResponse {
    name: String,
    id: u64,
}
#[get("/search?<q>")]
pub async fn simple_search(q: &str, client: &State<reqwest::Client>) -> Result<Template, AppError> {
    // TODO: get data from IGDB Api
    let response: Vec<GameResponse> = client
        .post("https://api.igdb.com/v4/games")
        .body(r#"search "zelda"; limit 10; fields name;"#)
        .send()
        .await?
        .json()
        .await?;
    info!("API response was {:#?}", response);
    info!("Query string was {:?}", q);
    Ok(Template::render("index", context![]))
}
