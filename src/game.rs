use std::collections::HashMap;

use crate::{
    api_helpers::{process_rating, InvolvedCompany},
    utils::AppError,
};
use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::Deserialize;
#[derive(Deserialize, Debug)]
struct AgeRatingWithContent {
    category: u16,
    content_descriptions: Option<Vec<Category>>,
    rating: u16,
}
#[derive(Deserialize, Debug)]
struct Category(u32);
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
}
#[derive(Deserialize, Debug)]
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
        r#"fields name, aggregated_rating, rating, first_release_date, involved_companies.company.name, age_ratings.*, cover.image_id; where id = {};"#,
        id
    );
    dbg!(&api_string);
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
    let mut violence = 0;
    let mut language = 0;
    let mut sexual_content = 0;
    let mut substances = 0;
    let mut gambling = "No";
    let mut ratings_weights: Vec<HashMap<u16, f32>> = Vec::new();
    let ratings_rankings_table = include_str!("esrb_content_descriptors.txt").to_string();
    println!("{}", &ratings_rankings_table);
    let mut groups = ratings_rankings_table.split("---");
    // Skip empty first line
    groups.next();
    let vec_groups: Vec<&str> = groups.clone().collect();
    dbg!(vec_groups);
    for group in groups {
        let mut weights: HashMap<u16, f32> = HashMap::new();
        let mut lines = group.lines();
        // Skips table label
        lines.next();
        for line in lines.clone() {
            println!("{line}");
        }
        let max = lines.clone().count();
        for (ranking, line) in lines.enumerate() {
            let num = line.split("|").nth(2);
            if let Some(num) = num {
                dbg!(&num);
                let num: u16 = num.trim().parse().unwrap();
                let score = (ranking + 1) as f32 / max as f32;
                weights.insert(num, score);
            }
        }
        ratings_weights.push(weights);
    }
    dbg!(&ratings_weights);
    let aggregate_rating = process_rating(response.aggregated_rating);
    Ok(Template::render("index", context![]))
}
