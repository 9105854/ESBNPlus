use chrono::prelude::*;
use serde::Deserialize;

use crate::utils::ESRBRating;
pub fn process_release_year(input: Option<i64>) -> String {
    if let Some(release_year) = input {
        let parsed_date = chrono::DateTime::from_timestamp(release_year, 0);
        if let Some(parsed_date) = parsed_date {
            parsed_date.year().to_string()
        } else {
            "N/A".to_string()
        }
    } else {
        "N/A".to_string()
    }
}

pub fn process_rating(input: Option<f32>) -> String {
    input
        .map(|e| ((e * 100.0).round() / 100.0).to_string())
        .unwrap_or("N/A".to_string())
}

#[derive(Deserialize, Debug)]
pub struct InvolvedCompany {
    pub company: Company,
}
#[derive(Deserialize, Debug)]
pub struct Company {
    pub name: String,
}
#[derive(Deserialize, Debug)]
pub struct AgeRating {
    pub category: u16,
    pub rating: u16,
}
/// Returns the esrb rating text and image url in tuple (esrb_rating, esrb_img)
pub fn process_esrb(input: Option<Vec<AgeRating>>) -> (String, Option<String>) {
    let mut esrb_rating = "N/A".to_string();

    let mut esrb_img = None;
    if let Some(age_ratings) = input {
        let rating_number = age_ratings.iter().find(|rating| rating.category == 1); // Gets only ESRB Rating
        if let Some(rating_number) = rating_number {
            let esrb_classification = ESRBRating::from_number(rating_number.rating);
            if let Some(esrb_classification) = esrb_classification {
                esrb_img = Some(esrb_classification.to_img_url());
                esrb_rating = esrb_classification.to_string();
            }
        }
    }
    (esrb_rating, esrb_img)
}
