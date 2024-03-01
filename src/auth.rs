use crate::utils::AppError;
use crate::SqliteState;
use anyhow::anyhow;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use rocket::http::{CookieJar, Status};
use rocket::{
    form::Form,
    request::{self, FromRequest},
    response::Redirect,
    Request,
};
use rocket::{http::Header, request::Outcome};
use rocket::{Route, State};
use rocket_dyn_templates::context;
use rocket_dyn_templates::Template;
pub fn _auth_routes() -> Vec<Route> {
    routes![
        already_auth_login,
        login_ui,
        login,
        already_auth_signup,
        signup_ui,
        signup,
        logout_ui,
        logout
    ]
}
pub struct User {
    pub user_id: String,
}
#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = AppError;
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let user_id = req.cookies().get_private("user_id");
        if let Some(user_id) = user_id {
            return Outcome::Success(User {
                user_id: user_id.value_trimmed().into(),
            });
        } else {
            return Outcome::Forward(Status::Unauthorized);
        }
    }
}
#[derive(Responder)]
pub struct AuthResponder {
    inner: String,
    hx_retarget: Header<'static>,
}
#[get("/auth/login")]
pub fn already_auth_login(_user: User) -> Redirect {
    Redirect::to(uri!("/auth/logout"))
}
#[get("/auth/login", rank = 2)]
pub fn login_ui(_cookies: &CookieJar<'_>) -> Template {
    Template::render("login", context![])
}
#[derive(FromForm, Debug)]
pub struct Login {
    email: String,
    password: String,
}

#[post("/auth/login", data = "<login>", rank = 2)]
pub async fn login(
    login: Form<Login>,
    sqlite_state: &State<SqliteState>,
    cookies: &CookieJar<'_>,
) -> Result<AuthResponder, AppError> {
    // check if already signed in
    if cookies.get_private("user_id").is_some() {
        return Err(anyhow!(
            r#"<span>Already logged in!</span>
            Log out <a href="/auth/logout">here</a>"#
        )
        .into());
    }
    // check email

    let error_msg = "<span>Email or password is incorrect</span>".to_string();

    let error_response = AuthResponder {
        inner: error_msg,
        hx_retarget: Header {
            name: "HX-Retarget".into(),
            value: "#response".into(),
        },
    };
    let existing_email: Vec<String> = sqlx::query_scalar("SELECT email FROM users WHERE email = ?")
        .bind(login.email.clone())
        .fetch_all(&sqlite_state.pool)
        .await?;
    if existing_email.is_empty() {
        return Ok(error_response);
    }
    let db_password_hash: String = sqlx::query_scalar("SELECT password FROM users WHERE email = ?")
        .bind(login.email.clone())
        .fetch_one(&sqlite_state.pool)
        .await?;
    let parsed_hash = PasswordHash::new(&db_password_hash)?;
    let correct_password = Argon2::default()
        .verify_password(login.password.as_bytes(), &parsed_hash)
        .is_ok();
    if !correct_password {
        return Ok(error_response);
    }
    // get user id and save it to secure cookies
    let user_id: String = sqlx::query_scalar("SELECT userId FROM users WHERE email = ?")
        .bind(login.email.clone())
        .fetch_one(&sqlite_state.pool)
        .await?;
    cookies.add_private(("user_id", user_id));

    let success_response = AuthResponder {
        inner: "Logged in!".to_string(),
        hx_retarget: Header {
            name: "HX-Retarget".into(),
            value: "this".into(),
        },
    };
    Ok(success_response)
}
#[get("/auth/signup")]
pub fn already_auth_signup(_user: User) -> Redirect {
    Redirect::to(uri!("/auth/logout"))
}
#[get("/auth/signup", rank = 2)]
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
) -> Result<AuthResponder, AppError> {
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

    let error_response = AuthResponder {
        inner: errors,
        hx_retarget: Header {
            name: "HX-Retarget".into(),
            value: "#response".into(),
        },
    };
    if is_error {
        return Ok(error_response);
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

    let success = AuthResponder {
        inner: "Signed Up! Go to <a href=\"/browse\"> browse</a>".into(),
        hx_retarget: Header {
            name: "HX-Retarget".into(),
            value: "this".into(),
        },
    };
    Ok(success)
}

#[delete("/auth/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove_private("user_id");
    Redirect::to(uri!("/auth/login"))
}
#[get("/auth/logout")]
pub fn logout_ui(user: User) -> Template {
    let user_id = user.user_id;
    Template::render("logout", context![user_id])
}
#[get("/auth/logout", rank = 2)]
pub fn logout_ui_no_auth() -> Redirect {
    Redirect::to(uri!("/auth/login"))
}
