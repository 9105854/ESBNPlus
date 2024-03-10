use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};

use crate::{
    api_helpers::{process_esrb, AgeRating},
    auth::User,
    game::Cover,
    utils::{genre_slug_to_name, AppError},
    SqliteState,
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
#[derive(Serialize)]
struct BrowseSection {
    games: Vec<BrowseGameListing>,
    browse_section_name: String,
}
#[get("/browse")]
pub async fn browse_ui(
    client: &State<reqwest::Client>,
    user: Option<User>,
    sqlite_state: &State<SqliteState>,
) -> Result<Template, AppError> {
    let mut browse_sections = Vec::new();
    let url = "https://api.igdb.com/v4/games";
    let new_api_string = r#"limit 30; fields name, cover.image_id, age_ratings.*; sort first_release_date desc; where aggregated_rating_count > 5 | rating_count > 50;"#;
    let highly_rated_string = r#"limit 30; fields name, cover.image_id, age_ratings.*; sort first_release_date desc; where (aggregated_rating_count > 5 | rating_count > 100) & (aggregated_rating > 90 | rating > 95);"#;
    browse_sections.push(
        make_game_section(new_api_string.to_string(), &client, "New Games".to_string()).await?,
    );

    browse_sections.push(
        make_game_section(
            highly_rated_string.to_string(),
            &client,
            "Highly Rated Games".to_string(),
        )
        .await?,
    );

    if let Some(user) = &user {
        let genre_prefs: String =
            sqlx::query_scalar("SELECT genrePreferences FROM users WHERE userId = ?")
                .bind(&user.user_id)
                .fetch_one(&sqlite_state.pool)
                .await?;
        let genre_prefs: Result<Vec<String>, _> = serde_json::from_str(&genre_prefs);
        if let Ok(genre_prefs) = genre_prefs {
            for genre in genre_prefs.iter() {
                let genre_string = format!(
                    r#"limit 30; fields name, cover.image_id, age_ratings.*; sort first_release_date desc; where genres.slug = "{}" & (aggregated_rating_count > 1 | rating_count > 1);"#,
                    genre
                );
                info!("API string for browse is {:?}", genre_string);
                let genre_section =
                    make_game_section(genre_string, &client, genre_slug_to_name(genre.to_string()))
                        .await?;
                browse_sections.push(genre_section);
            }
        } // let genres =
    }
    let user_id = user.map(|e| e.user_id);
    Ok(Template::render(
        "browse",
        context![browse_sections, user_id],
    ))
}
async fn make_game_section(
    api_string: String,
    client: &reqwest::Client,
    browse_section_name: String,
) -> Result<BrowseSection, AppError> {
    let url = "https://api.igdb.com/v4/games";
    let games_api_response: Vec<BrowseGameApiResponse> = client
        .post(url)
        .body(api_string)
        .send()
        .await?
        .json()
        .await?;
    let mut games = Vec::new();
    for game in games_api_response.into_iter() {
        games.push({
            let title = game.name.clone();
            let cover_img_url = if let Some(cover) = &game.cover {
                let id = cover.image_id.clone();
                format!("https://images.igdb.com/igdb/image/upload/t_cover_big/{id}.jpg")
            } else {
                "https://images.igdb.com/igdb/image/upload/t_cover_big/co7m7e.png".to_string()
            };

            let cover_img_alt = format!("Cover art for the game {title}");
            let (esrb_img_alt, esrb_img_url) = process_esrb(game.age_ratings);

            BrowseGameListing {
                id: game.id,
                cover_img_url,
                cover_img_alt,
                esrb_img_url,
                esrb_img_alt,
            }
        });
    }
    Ok(BrowseSection {
        games,
        browse_section_name,
    })
}
