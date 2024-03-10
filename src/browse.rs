use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};

use crate::{
    api_helpers::{process_esrb, AgeRating},
    game::Cover,
    utils::AppError,
};
#[derive(Deserialize)]
struct BrowseGameApiResponse {
    id: u64,
    name: String,
    age_ratings: Option<Vec<AgeRating>>,
    cover: Option<Cover>,
}
#[derive(Serialize)]
struct BrowseGameListing {
    id: u64,
    cover_img_url: String,
    cover_img_alt: String,
    esrb_img_url: Option<String>,
    esrb_img_alt: String,
}
#[get("/browse")]
pub async fn browse_ui(client: &State<reqwest::Client>) -> Result<Template, AppError> {
    let url = "https://api.igdb.com/v4/games";
    let new_api_string = r#"limit 30; fields name, cover.image_id, age_ratings.*; sort first_release_date desc; where aggregated_rating_count > 5 | rating_count > 50;"#;
    let highly_rated_string = r#"limit 30; fields name, cover.image_id, age_ratings.*; sort first_release_date desc; where (aggregated_rating_count > 5 | rating_count > 100) & (aggregated_rating > 90 | rating > 95);"#;
    let new_games_api: Vec<BrowseGameApiResponse> = client
        .post(url)
        .body(new_api_string)
        .send()
        .await?
        .json()
        .await?;
    let highly_rated_games_api: Vec<BrowseGameApiResponse> = client
        .post(url)
        .body(highly_rated_string)
        .send()
        .await?
        .json()
        .await?;
    let mut new_games = Vec::new();
    for game in new_games_api.into_iter() {
        let title = game.name.clone();
        let cover_img_url = if let Some(cover) = &game.cover {
            let id = cover.image_id.clone();
            format!("https://images.igdb.com/igdb/image/upload/t_cover_big/{id}.jpg")
        } else {
            "https://images.igdb.com/igdb/image/upload/t_cover_big/co7m7e.png".to_string()
        };

        let cover_img_alt = format!("Cover art for the game {title}");
        let (esrb_img_alt, esrb_img_url) = process_esrb(game.age_ratings);

        new_games.push(BrowseGameListing {
            id: game.id,
            cover_img_url,
            cover_img_alt,
            esrb_img_url,
            esrb_img_alt,
        })
    }
    let mut highly_rated_games = Vec::new();
    for game in highly_rated_games_api.into_iter() {
        let title = game.name.clone();
        let cover_img_url = if let Some(cover) = &game.cover {
            let id = cover.image_id.clone();
            format!("https://images.igdb.com/igdb/image/upload/t_cover_big/{id}.jpg")
        } else {
            "https://images.igdb.com/igdb/image/upload/t_cover_big/co7m7e.png".to_string()
        };

        let cover_img_alt = format!("Cover art for the game {title}");
        let (esrb_img_alt, esrb_img_url) = process_esrb(game.age_ratings);

        highly_rated_games.push(BrowseGameListing {
            id: game.id,
            cover_img_url,
            cover_img_alt,
            esrb_img_url,
            esrb_img_alt,
        });
    }
    Ok(Template::render(
        "browse",
        context![highly_rated_games, new_games],
    ))
}
