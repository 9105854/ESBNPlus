use crate::utils::{AppError, ESRBRating, HXRequest};
use chrono::prelude::*;
use chrono::serde::ts_seconds_option;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use tera::Context;
#[derive(Deserialize, Debug)]
struct SearchResponse {
    id: u64,
    name: String,
    aggregated_rating: Option<f32>,
    rating: Option<f32>,
    first_release_date: Option<i64>,
    involved_companies: Option<Vec<InvolvedCompany>>,
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
#[derive(Serialize, Debug)]
struct SearchResult {
    game_title: String,
    igdb_rating: String,
    publisher: String,
    aggregate_rating: String,
    release_year: String,
    esrb_rating: String,
    esrb_img: String,
}

#[get("/search", rank = 3)]
pub async fn base_search_ui() -> Template {
    let results: Vec<SearchResult> = Vec::new();

    Template::render("search", context![results])
}
async fn search_logic(q: &str, client: &reqwest::Client) -> Result<Vec<SearchResult>, AppError> {
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
    let mut results: Vec<SearchResult> = Vec::new();
    for entity in response.into_iter() {
        let game_title = entity.name;
        let igdb_rating = entity
            .rating
            .map(|e| e.to_string())
            .unwrap_or("N/A".to_string());
        let publisher = if let Some(involved_companies) = entity.involved_companies {
            involved_companies[0].company.name.clone()
        } else {
            "N/A".to_string()
        };
        let aggregate_rating = entity
            .aggregated_rating
            .map(|e| e.to_string())
            .unwrap_or("N/A".to_string());

        let release_year = if let Some(release_year) = entity.first_release_date {
            let parsed_date = chrono::DateTime::from_timestamp(release_year, 0);
            if let Some(parsed_date) = parsed_date {
                parsed_date.year().to_string()
            } else {
                "N/A".to_string()
            }
        } else {
            "N/A".to_string()
        };
        let mut esrb_rating = "N/A".to_string();

        let mut esrb_img = "".to_string();
        if let Some(age_ratings) = entity.age_ratings {
            let rating_number = age_ratings
                .iter()
                .filter(|rating| rating.category == 1) // Gets only ESRB Rating
                .next();
            if let Some(rating_number) = rating_number {
                let esrb_classification = ESRBRating::from_number(rating_number.rating);
                if let Some(esrb_classification) = esrb_classification {
                    esrb_img = esrb_classification.to_img_url();
                    esrb_rating = esrb_classification.to_string();
                }
            }
        }
        let result = SearchResult {
            game_title,
            igdb_rating,
            publisher,
            aggregate_rating,
            release_year,
            esrb_rating,
            esrb_img,
        };
        results.push(result);
    }
    Ok(results)
}
#[get("/search?<q>")]
pub async fn hx_search(
    q: &str,
    client: &State<reqwest::Client>,
    hx_request: HXRequest,
) -> Result<Template, AppError> {
    let results = search_logic(q, client).await?;
    Ok(Template::render("search_results", context![results]))
}
#[get("/search?<q>", rank = 2)]
pub async fn simple_search(q: &str, client: &State<reqwest::Client>) -> Result<Template, AppError> {
    // TODO: get data from IGDB Api
    let results = search_logic(q, client).await?;
    dbg!(&results);
    Ok(Template::render("search", context![results]))
}
