use crate::utils::AppError;
use crate::SqliteState;
use anyhow::anyhow;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::Argon2;
use argon2::PasswordHasher;
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::context;
use rocket_dyn_templates::Template;
use sqlx::SqlitePool;
#[get("/auth/login")]
pub fn login_ui() -> Template {
    Template::render("login", context![])
}

struct Login {
    username: String,
    password: String,
}
#[post("/auth/login")]
pub fn login() -> Result<(), AppError> {
    todo!();
}

#[get("/auth/signup")]
pub fn signup_ui() -> Template {
    Template::render("signup", context![])
}
#[derive(FromForm, Debug)]
pub struct SignUp {
    email: String,
    username: String,
    password: String,
    #[field(name = "genrePreferences")]
    genre_preferences: Vec<String>,
}

#[post("/auth/signup", data = "<signup>")]
pub async fn signup(
    signup: Form<SignUp>,
    sqlite_state: &State<SqliteState>,
    cookies: &CookieJar<'_>,
) -> Result<String, AppError> {
    // validation
    let mut is_error = false;
    let mut errors = "".to_string();
    let existing_email: Vec<String> = sqlx::query_scalar("SELECT email FROM users WHERE email = ?")
        .bind(signup.email.clone())
        .fetch_all(&sqlite_state.pool)
        .await?;
    if !existing_email.is_empty() {
        errors.push_str(
            "<span>Email already exists! Log in <a href=\"/auth/login\">here.</a></span>",
        );
        is_error = true;
    }
    let existing_username: Vec<String> =
        sqlx::query_scalar("SELECT username FROM users WHERE username = ?")
            .bind(signup.username.clone())
            .fetch_all(&sqlite_state.pool)
            .await?;

    if !existing_username.is_empty() {
        errors.push_str("<span>Username already exists!</span>");
        is_error = true;
    }
    if signup.genre_preferences.len() < 3 {
        errors.push_str("<span>Please select at least 3 preferences</span>");
        is_error = true;
    }
    if is_error {
        return Ok(errors);
    }
    let uuid = uuid::Uuid::new_v4().to_string();
    let genre_pref = serde_json::to_string(&signup.genre_preferences)?;
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(signup.password.as_bytes(), &salt)?
        .to_string();
    sqlx::query!("INSERT INTO users (userId, email, username, password, genrePreferences) VALUES (?, ?, ?, ?, ?)", uuid, signup.email, signup.username, password_hash, genre_pref).execute(&sqlite_state.pool).await?;
    // private cookies cannot be inspected, tampered with, or manufactured by clients
    cookies.add_private(("user_id", uuid));
    Ok("Signed Up! Go to <a href=\"/browse\"> browse</a>".into())
}
