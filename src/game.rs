use crate::{
    api_helpers::{process_rating, AgeRating, InvolvedCompany},
    utils::AppError,
};
use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::Deserialize;
use tera::Context;
#[derive(Deserialize)]
struct AgeRatingWithContent {
    category: u16,
    content_descriptions: Vec<u16>,
    rating: u16,
}
#[derive(Deserialize)]
struct GameResponse {
    id: u64,
    name: String,
    aggregated_rating: Option<f32>,
    rating: Option<f32>,
    first_release_date: Option<i64>,
    involved_companies: Option<Vec<InvolvedCompany>>,
    age_ratings: Option<Vec<AgeRatingWithContent>>,
    cover: Option<Cover>,
}
#[derive(Deserialize)]
struct Cover {
    image_id: String,
}
struct GameListing {
    title: String,
    cover_img_url: String,
    cover_img_alt: String,
    esrb_img_alt: String,
    esrb_img_url: String,
    igdb_rating: String,
    publisher: String,
    aggregate_rating: String,
    release_year: String,
    violence: String,
    language: String,
    sexual_content: String,
    substances: String,
    gambling: String,
}
#[get("/game/<id>")]
pub async fn game_ui(id: u64, client: &State<reqwest::Client>) -> Result<Template, AppError> {
    let api_string = format!(
        r#"fields name, aggregated_rating, rating, first_release_date, involved_companies.company.name, age_ratings.*, cover.image_id; where id = {}"#,
        id
    );

    let response: Vec<GameResponse> = client
        .post("https://api.igdb.com/v4/games")
        .body(api_string)
        .send()
        .await?
        .json()
        .await?;
    if response.is_empty() {
        return Err(anyhow::anyhow!("Couldn't find game").into());
    }
    let response = &response[0];
    let title = response.name.to_string();
    let cover_img_url = response.cover.unwrap_or("".to_string()).image_id;
    let aggregate_rating = process_rating(response.aggregated_rating);
    Ok(Template::render("index", context![]))
}
