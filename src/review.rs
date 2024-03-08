use crate::{auth::User, utils::AppError, SqliteState};
use rocket::{
    form::Form,
    response::{content::RawHtml, Redirect},
    State,
};
use rocket_dyn_templates::{context, Template};
use sqlx::SqlitePool;
#[get("/review?<game_id>")]
pub async fn review_ui(game_id: u64, _user: User) -> Template {
    Template::render("review_ui", context![game_id])
}
#[get("/review?<game_id>", rank = 2)]
pub async fn review_auth_response(game_id: u64) -> RawHtml<String> {
    RawHtml(
        r#"Must be logged in to review! Log in <a class="hover:underline text-emerald-700" href="/auth/login" hx-boost="false">here.</a>"#
            .to_string(),
    )
}
#[derive(FromForm)]
pub struct ReviewData {
    game_id: u64,
    enjoyability: u16,
    educational_value: u16,
    replayability: u16,
    usability: u16,
    title: String,
    content: String,
}
pub struct ReviewSQL {
    review_id: String,
    enjoyability: u16,
    educational_value: u16,
    usability: u16,
    replayability: u16,
    title: Option<String>,
    content: Option<String>,
    game_id: u64,
    user_id: String,
}
#[post("/review", data = "<review>")]
pub async fn save_review(
    review: Form<ReviewData>,
    user: User,
    pool: &State<SqliteState>,
) -> Result<RawHtml<&'static str>, AppError> {
    // TODO: Add form validation for max and min review scores
    let user_id = user.user_id;
    let review_id = uuid::Uuid::new_v4().to_string();
    let content = if &review.content == "" {
        None
    } else {
        Some(&review.content)
    };

    let title = if &review.title == "" {
        None
    } else {
        Some(&review.title)
    };
    sqlx::query("INSERT INTO reviews VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(review_id)
        .bind(review.enjoyability)
        .bind(review.educational_value)
        .bind(review.usability)
        .bind(review.replayability)
        .bind(&review.title)
        .bind(content)
        .bind(review.game_id as i64)
        .bind(user_id)
        .execute(&pool.pool)
        .await?;
    info!("Review successfully saved");
    Ok(RawHtml("<span>Review Posted!<span>"))
}
