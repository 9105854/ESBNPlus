use std::fmt::Display;

use chrono::Datelike;
use rocket::{
    http::Status,
    request::{self, FromRequest, Outcome},
    response::{self, Responder},
    Request,
};
#[derive(Debug)]
pub struct AppError(pub anyhow::Error);
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        AppError(error.into())
    }
}

impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, request: &Request<'_>) -> response::Result<'static> {
        response::Debug(self.0).respond_to(request)
    }
}
pub enum ESRBRating {
    RatingPending,
    RatingPendingLikelyMature,
    Everyone,
    Everyone10,
    Teen,
    Mature17,
    AdultsOnly18,
}
impl ESRBRating {
    /// Creates an ESRBRating enum from a number
    pub fn from_number(num: u16) -> Option<ESRBRating> {
        match num {
            6 => Some(ESRBRating::RatingPending),
            8 => Some(ESRBRating::Everyone),
            9 => Some(ESRBRating::Everyone10),
            10 => Some(ESRBRating::Teen),
            11 => Some(ESRBRating::Mature17),
            12 => Some(ESRBRating::AdultsOnly18),
            _ => None,
        }
    }
    /// Gives the matching ESRB rating url for each rating
    pub fn to_img_url(&self) -> String {
        let base_string = "https://www.esrb.org/wp-content/uploads/2019/05/";
        let suffix = match self {
            ESRBRating::RatingPending => "RP",
            ESRBRating::RatingPendingLikelyMature => "RP-LM17-English",
            ESRBRating::Everyone => "E",
            ESRBRating::Everyone10 => "E10plus",
            ESRBRating::Teen => "T",
            ESRBRating::Mature17 => "M",
            ESRBRating::AdultsOnly18 => "AO",
        };
        format!("{}{}.svg", base_string, suffix)
    }
}
impl Display for ESRBRating {
    /// Does not fail with valid ESRBRating enum
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            ESRBRating::RatingPending => "Rating Pending",
            ESRBRating::RatingPendingLikelyMature => "Rating Pending - Likely Mature",
            ESRBRating::Everyone => "Everyone",
            ESRBRating::Everyone10 => "Everyone 10+",
            ESRBRating::Teen => "Teen",
            ESRBRating::Mature17 => "Mature 17+",
            ESRBRating::AdultsOnly18 => "Adults Only 18+",
        };
        f.write_str(string);
        std::fmt::Result::Ok(())
    }
}
pub fn round_2(num: f64) -> f64 {
    (num * 100.0).round() / 100.0
}
pub fn round_1(num: f32) -> f32 {
    (num * 10.0).round() / 10.0
}
pub struct HXRequest;
#[rocket::async_trait]
impl<'r> FromRequest<'r> for HXRequest {
    type Error = AppError;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let headers = req.headers();
        if headers.contains("HX-Request") && !headers.contains("HX-Boosted") {
            return Outcome::Success(HXRequest);
        } else {
            return Outcome::Forward(Status::Continue);
        }
    }
}

use chrono::NaiveDate;
use chrono::ParseError;

use rocket::form::{self, DataField, FromFormField, ValueField};

pub struct NaiveDateForm(pub NaiveDate);
#[rocket::async_trait]
impl<'r> FromFormField<'r> for NaiveDateForm {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        match NaiveDate::parse_from_str(field.value, "%Y-%m-%d") {
            Ok(date) => Ok(NaiveDateForm(date)),
            Err(e) => Err(form::Error::validation(format!(
                "Couldn't parse time: {}",
                e
            )))?,
        }
    }
}
pub fn genre_slug_to_name(mut slug: String) -> String {
    if slug != "role-playing-rpg" {
        let name: String = slug
            .char_indices()
            .map(|(i, c)| {
                if i == 0 {
                    c.to_uppercase().next().unwrap()
                } else {
                    c
                }
            })
            .collect();
        return name;
    } else {
        "Role Playing (RPG)".to_string()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn esrb_to_string() {
        let rating = ESRBRating::Everyone10;
        assert_eq!(rating.to_string(), "Everyone 10+".to_string())
    }
}
