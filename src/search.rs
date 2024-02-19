use crate::utils::AppError;
use chrono::prelude::*;
use chrono::serde::ts_seconds_option;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::Deserialize;
#[derive(Deserialize, Debug)]
struct SearchResponse {
    id: u64,
    name: String,
    aggregated_rating: Option<f32>,
    rating: Option<f32>,
    #[serde(with = "ts_seconds_option")]
    first_release_date: Option<DateTime<Utc>>,
    involved_companies: Vec<InvolvedCompany>,
    age_ratings: Option<Vec<AgeRating>>,
}
#[derive(Deserialize, Debug)]
struct AgeRating {
    category: u16,
    rating: u16,
}
#[derive(Deserialize, Debug)]
struct InvolvedCompany {
    company: Company,
}
#[derive(Deserialize, Debug)]
struct Company {
    name: String,
}
#[get("/search?<q>")]
pub async fn simple_search(q: &str, client: &State<reqwest::Client>) -> Result<Template, AppError> {
    // TODO: get data from IGDB Api
    let body = format!(
        r#"search "{}"; fields name, aggregated_rating, rating, first_release_date, involved_companies.company.name, age_ratings.rating, age_ratings.category;"#,
        q
    );
    let response: Vec<SearchResponse> = client
        .post("https://api.igdb.com/v4/games")
        .body(body)
        .send()
        .await?
        .json()
        .await?;
    info!("API response was {:#?}", response);
    info!("Query string was {:?}", q);
    Ok(Template::render("index", context![]))
}
