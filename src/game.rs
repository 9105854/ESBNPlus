use std::{collections::HashMap, ops::Add};

use crate::{
    api_helpers::{process_esrb, process_rating, process_release_year, AgeRating, InvolvedCompany},
    auth::User,
    utils::{round_1, AppError},
    SqliteState,
};
use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
#[derive(Deserialize, Debug, Clone)]
struct AgeRatingWithContent {
    category: u16,
    content_descriptions: Option<Vec<Category>>,
    rating: u16,
}
#[derive(Deserialize, Debug, Clone)]
struct Category {
    category: u16,
}
#[derive(Deserialize, Debug)]
struct GameResponse {
    id: Option<u64>,
    name: Option<String>,
    aggregated_rating: Option<f32>,
    rating: Option<f32>,
    first_release_date: Option<i64>,
    involved_companies: Option<Vec<InvolvedCompany>>,
    age_ratings: Option<Vec<AgeRatingWithContent>>,
    cover: Option<Cover>,
    summary: Option<String>,
}
#[derive(Deserialize, Debug)]
struct Cover {
    image_id: String,
}
#[derive(Debug, Serialize)]
pub struct GameListing {
    pub title: String,
    pub cover_img_url: String,
    pub cover_img_alt: String,
    pub esrb_img_alt: String,
    pub esrb_img_url: Option<String>,
    pub summary: Option<String>,
    pub igdb_rating: String,
    pub publisher: String,
    pub aggregate_rating: String,
    pub release_year: String,
    pub content_descriptors: Option<ContentDescriptorCategories>,
    pub game_id: u64,
    pub user_metrics: Option<UserMetrics>,
    pub written_reviews: Vec<WrittenReview>,
    pub user_id: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct ContentDescriptorCategories {
    pub violence: f32,
    pub language: f32,
    pub sexual_content: f32,
    pub substances: f32,
    pub gambling: bool,
}
#[derive(Debug, Serialize, FromRow)]
pub struct WrittenReview {
    title: String,
    content: String,
    rating: u16,
    username: String,
}
pub async fn game_logic(
    client: &reqwest::Client,
    sqlite_pool: &SqlitePool,
    user: Option<User>,
    id: u64,
) -> Result<GameListing, AppError> {
    let api_string = format!(
        r#"fields name, aggregated_rating, rating, summary, first_release_date, involved_companies.company.name, age_ratings.*, age_ratings.content_descriptions.category, cover.image_id; where id = {};"#,
        id
    );
    let response: Vec<GameResponse> = client
        .post("https://api.igdb.com/v4/games")
        .body(api_string)
        .send()
        .await?
        .json()
        .await?;
    if response.is_empty() || response[0].id.is_none() {
        dbg!(response);
        return Err(anyhow::anyhow!("Couldn't find game").into());
    }
    let response = &response[0];
    dbg!(&response);
    let title = response.name.clone().unwrap().to_string();
    let cover_img_url = if let Some(cover) = &response.cover {
        let id = cover.image_id.clone();
        format!("https://images.igdb.com/igdb/image/upload/t_cover_big/{id}.jpg")
    } else {
        "https://images.igdb.com/igdb/image/upload/t_cover_big/co7m7e.png".to_string()
    };
    let cover_img_alt = format!("Cover art for the game {title}");
    let igdb_rating = process_rating(response.rating);
    let publisher = if let Some(involved_companies) = &response.involved_companies {
        involved_companies[0].company.name.clone()
    } else {
        "N/A".to_string()
    };
    // Content Descriptors yay!!!
    let mut violence_score = 0.0;
    let mut language_score = 0.0;
    let mut sexual_content_score = 0.0;
    let mut substances_score = 0.0;
    let mut gambling = false;
    let mut ratings_weights: Vec<HashMap<u16, f32>> = Vec::new();
    let ratings_rankings_table = include_str!("esrb_content_descriptors.txt").to_string();
    println!("{}", &ratings_rankings_table);
    let mut groups = ratings_rankings_table.split("---");
    // Skip empty first line
    groups.next();
    for group in groups {
        let mut weights: HashMap<u16, f32> = HashMap::new();

        for (score, level_group) in group.split("===").enumerate() {
            for line in level_group.lines() {
                let num = line.split("|").nth(2);
                if let Some(num) = num {
                    let num: u16 = num.trim().parse().unwrap();
                    let score = score + 1;
                    weights.insert(num, score as f32);
                }
            }
        }
        dbg!(&weights);
        ratings_weights.push(weights);
    }
    let mut content_descriptors: Option<ContentDescriptorCategories> = None;

    if let Some(age_ratings) = &response.age_ratings {
        let rating = age_ratings.iter().find(|e| e.category == 1);
        if let Some(rating) = rating {
            if let Some(content_descriptions) = &rating.content_descriptions {
                for descriptor in content_descriptions.iter() {
                    for (index, category) in ratings_weights.iter().enumerate() {
                        let score = category.get(&descriptor.category);
                        if let Some(score) = score {
                            match index {
                                0 => violence_score += score,
                                1 => language_score += score,
                                2 => sexual_content_score += score,
                                3 => gambling = true,
                                4 => substances_score += score,
                                _ => (),
                            }
                        }
                    }
                }
                // max is (n+1)/2
                violence_score = violence_score.min(5.0);
                language_score = language_score.min(5.0);
                sexual_content_score = sexual_content_score.min(5.0);
                substances_score = substances_score.min(5.0);

                // substances_score =
                // violence_score / ((ratings_weights[4].len() as f32 + 1.0) / 2.0) * 5.0;
                let violence = round_1(violence_score);
                let language = round_1(language_score);
                let sexual_content = round_1(sexual_content_score);
                let substances = round_1(substances_score);
                content_descriptors = Some(ContentDescriptorCategories {
                    violence,
                    language,
                    sexual_content,
                    substances,
                    gambling,
                })
            }
        }
    };
    let aggregate_rating = process_rating(response.aggregated_rating);
    let release_year = process_release_year(response.first_release_date);
    let simplified_esrb = response.age_ratings.clone().map(|e| {
        e.iter()
            .map(|f| AgeRating {
                category: f.category,
                rating: f.rating,
            })
            .collect()
    });
    let (esrb_img_alt, esrb_img_url) = process_esrb(simplified_esrb);
    // TODO: Round the fields, check for no reviews as well
    let user_metrics_from_db : Vec<UserMetrics> = sqlx::query_as("SELECT AVG(enjoyability) as enjoyability, AVG(educationalValue) as educationalValue, AVG(replayability) as replayability, AVG(usability) as usability, COUNT(*) as count FROM reviews WHERE gameId = ? HAVING count > 0").bind(id as i64).fetch_all(sqlite_pool).await?;
    let user_metrics = if user_metrics_from_db.is_empty() {
        None
    } else {
        Some(user_metrics_from_db[0])
    };
    let written_reviews: Vec<WrittenReview> = sqlx::query_as(
        "SELECT reviews.content, reviews.title, users.username, reviews.enjoyability as rating FROM reviews, users WHERE reviews.gameId = ? AND reviews.content IS NOT NULL AND users.userId = reviews.userId ",
    )
    .bind(id as i64)
    .fetch_all(sqlite_pool)
    .await?;
    let summary = if let Some(mut summary) = response.summary.clone() {
        summary.truncate(400);
        let position = summary.rfind('.');
        if let Some(position) = position {
            summary.truncate(position)
        }
        summary.push('.');
        Some(summary)
        // max characters 400
    } else {
        None
    };
    let user_id = user.map(|e| e.user_id);
    Ok(GameListing {
        title,
        cover_img_url,
        cover_img_alt,
        summary,
        esrb_img_alt,
        esrb_img_url,
        igdb_rating,
        publisher,
        aggregate_rating,
        release_year,
        content_descriptors,
        game_id: id,
        written_reviews,
        user_metrics,
        user_id,
    })
}
#[get("/game/<id>")]
pub async fn game_ui(
    id: u64,
    client: &State<reqwest::Client>,
    sqlite_state: &State<SqliteState>,
    user: Option<User>,
) -> Result<Template, AppError> {
    let game_listing = game_logic(client, &sqlite_state.pool, user.clone(), id).await?;
    Ok(Template::render("game", game_listing))
}

#[derive(FromRow, Debug, Serialize, Clone, Copy)]
#[sqlx(rename_all = "camelCase")]
pub struct UserMetrics {
    pub enjoyability: f32,
    pub educational_value: f32,
    pub replayability: f32,
    pub usability: f32,
}
