use crate::utils::AppError;
use crate::SqliteState;
use rocket::form::Form;
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
    genrePreferences: Vec<String>,
}
#[post("/auth/signup", data = "<signup>")]
pub async fn signup(
    signup: Form<SignUp>,
    sqlite_state: &State<SqliteState>,
) -> Result<String, AppError> {
    let uuid = uuid::Uuid::new_v4().to_string();
    let mut genre_pref = "".to_string();
    for preference in signup.genrePreferences.iter() {
        genre_pref.push_str(&format!("{} ", preference));
    }

    sqlx::query!("INSERT INTO users (userId, email, username, password, genrePreferences) VALUES (?, ?, ?, ?, ?)", uuid, signup.email, signup.username, signup.password, genre_pref).execute(&sqlite_state.pool).await?;

    Ok("Signed Up".into())
}
