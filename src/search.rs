use crate::api_helpers::{
    process_esrb, process_rating, process_release_year, AgeRating, InvolvedCompany,
};
use crate::auth::User;
use crate::game::game_logic;
use crate::utils::{AppError, HXRequest, NaiveDateForm};
use crate::SqliteState;
use chrono::Utc;
use rocket::form::Form;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::__private::de::ContentDeserializer;
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
    id: String,
    game_title: String,
    igdb_rating: String,
    publisher: String,
    aggregate_rating: String,
    release_year: String,
    esrb_rating: String,
    esrb_img: Option<String>,
}

#[get("/search", rank = 3)]
pub async fn base_search_ui(user: Option<User>) -> Template {
    let results: Vec<SearchResult> = Vec::new();
    let query = "";
    match user {
        Some(user) => {
            let user_id = user.user_id;
            Template::render("search", context![results, query, user_id])
        }
        None => Template::render("search", context![results, query]),
    }
}
async fn search_logic(
    body: String,
    client: &reqwest::Client,
) -> Result<Vec<SearchResult>, AppError> {
    let response: Vec<SearchResponse> = client
        .post("https://api.igdb.com/v4/games")
        .body(body)
        .send()
        .await?
        .json()
        .await?;

    info!("API response was {:#?}", response);
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
        let id = entity.id.to_string();
        let result = SearchResult {
            id,
            game_title,
            igdb_rating,
            publisher,
            aggregate_rating,
            release_year,
            esrb_rating,
            esrb_img,
        };
        dbg!(&result);
        results.push(result);
    }
    Ok(results)
}

#[get("/search?<q>")]
pub async fn hx_search(
    q: &str,
    client: &State<reqwest::Client>,
    _hx_request: HXRequest,
    user: Option<User>,
) -> Result<Template, AppError> {
    let body = format!(
        r#"search "{}"; fields name, aggregated_rating, rating, first_release_date, involved_companies.company.name, age_ratings.rating, age_ratings.category; where category = (0, 8, 9);"#,
        q
    );
    let results = search_logic(body, client).await?;
    let query = q;
    if let Some(user) = user {
        let user_id = user.user_id;
        Ok(Template::render(
            "search_results",
            context![results, query, user_id],
        ))
    } else {
        Ok(Template::render("search_results", context![results, query]))
    }
}
#[get("/search?<q>", rank = 2)]
pub async fn simple_search(
    q: &str,
    client: &State<reqwest::Client>,
    user: Option<User>,
) -> Result<Template, AppError> {
    let body = format!(
        r#"search "{}"; fields name, aggregated_rating, rating, first_release_date, involved_companies.company.name, age_ratings.rating, age_ratings.category; where category = (0, 8, 9);"#,
        q
    );
    let results = search_logic(body, client).await?;
    dbg!(&results);
    let query = q;
    match user {
        Some(user) => {
            let user_id = user.user_id;

            Ok(Template::render(
                "search",
                context![results, query, user_id],
            ))
        }
        None => Ok(Template::render("search", context![results, query])),
    }
}
#[derive(FromForm)]
pub struct AdvancedSearch {
    query: String,
    #[field(name = "esrbAgeRatings")]
    esrb_age_ratings: Vec<String>,
    violence: f32,
    language: f32,
    #[field(name = "sexualContent")]
    sexual_content: f32,
    substances: f32,
    gambling: bool,
    enjoyability: f32,

    #[field(name = "educationalValue")]
    educational_value: f32,
    replayability: f32,
    usability: f32,
    #[field(name = "dateFrom")]
    date_from: NaiveDateForm,
    #[field(name = "dateTo")]
    date_to: NaiveDateForm,
}
#[derive(Deserialize)]
pub struct GameIdOnly {
    id: u64,
}
#[post("/search/advanced", data = "<search>")]
pub async fn advanced_search(
    search: Form<AdvancedSearch>,
    client: &State<reqwest::Client>,
    sqlite_state: &State<SqliteState>,
    user: Option<User>,
) -> Result<Template, AppError> {
    let query = &search.query;

    let body = format!(r#"search "{}"; limit 30; where category = (0,8,9);"#, query);
    let response: Vec<GameIdOnly> = client
        .post("https://api.igdb.com/v4/games")
        .body(body)
        .send()
        .await?
        .json()
        .await?;
    let mut rich_game_results = Vec::new();
    for game in response.iter() {
        let rich_listing = game_logic(client, &sqlite_state.pool, user.clone(), game.id).await;
        if rich_listing.is_ok() {
            rich_game_results.push(rich_listing.unwrap())
        }
    }
    let results: Vec<SearchResult> = rich_game_results
        .iter()
        .filter(|game| search.esrb_age_ratings.contains(&game.esrb_img_alt))
        .filter(|game| {
            if let Some(content_descriptors) = &game.content_descriptors {
                content_descriptors.violence <= search.violence
                    && content_descriptors.language <= search.language
                    && content_descriptors.sexual_content <= search.sexual_content
                    && content_descriptors.substances <= search.substances
                    && (content_descriptors.gambling == search.gambling || search.gambling)
            } else {
                // if search.violence == 5 && search.language == 5 && search.sexual_content == 5 && search.substances == 5 && search.gambling = true {
                //     true
                // }
                true
            }
        })
        .filter(|game| {
            if let Some(user_metrics) = game.user_metrics {
                user_metrics.enjoyability >= search.enjoyability
                    && user_metrics.educational_value >= search.educational_value
                    && user_metrics.replayability >= search.replayability
                    && user_metrics.usability >= search.usability
            } else {
                search.replayability == 0.0
                    && search.educational_value == 0.0
                    && search.usability == 0.0
                    && search.enjoyability == 0.0
            }
        })
        .map(|rich_game| SearchResult {
            id: rich_game.game_id.to_string(),
            game_title: rich_game.title.clone(),
            igdb_rating: rich_game.igdb_rating.clone(),
            publisher: rich_game.publisher.clone(),
            aggregate_rating: rich_game.aggregate_rating.clone(),
            release_year: rich_game.release_year.clone(),
            esrb_rating: rich_game.esrb_img_alt.clone(),
            esrb_img: rich_game.esrb_img_url.clone(),
        })
        .collect();
    Ok(Template::render("search_results", context![results, query]))
}
