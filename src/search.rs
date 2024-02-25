use crate::api_helpers::{
    process_esrb, process_rating, process_release_year, AgeRating, InvolvedCompany,
};
use crate::utils::{AppError, HXRequest};
use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
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
        let igdb_rating = process_rating(entity.rating);
        let publisher = if let Some(involved_companies) = entity.involved_companies {
            involved_companies[0].company.name.clone()
        } else {
            "N/A".to_string()
        };
        let aggregate_rating = process_rating(entity.aggregated_rating);

        let release_year = process_release_year(entity.first_release_date);
        let (esrb_rating, esrb_img) = process_esrb(entity.age_ratings);
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
    _hx_request: HXRequest,
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
