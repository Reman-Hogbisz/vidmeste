use oauth2::reqwest::async_http_client;
use oauth2::{basic::BasicClient, revocation::StandardRevocableToken};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, RevocationUrl,
    Scope, TokenUrl,
};
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::response::Redirect;
use rocket_oauth2::{OAuth2, TokenResponse};
use std::borrow::Cow;
use std::env;
use std::fmt::Display;

use super::sql::insert_user;

pub struct Google;
pub struct Hogbisz;
pub struct Discord;

#[get("/login/google")]
pub async fn google_login() -> Redirect {
    let google_client_id = ClientId::new(
        env::var("GOOGLE_CLIENT_ID").expect("Missing the GOOGLE_CLIENT_ID environment variable."),
    );
    let google_client_secret = ClientSecret::new(
        env::var("GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
    );
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        .expect("Invalid token endpoint URL");

    let client = BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(
        RedirectUrl::new(
            env::var("BASE_URL").expect("Missing the BASE_URL environment variable.")
                + "/auth/google",
        )
        .expect("Invalid redirect URL"),
    )
    .set_revocation_uri(
        RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
            .expect("Invalid revocation endpoint URL"),
    );

    let (authorize_url, _) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/calendar".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/plus.me".to_string(),
        ))
        .url();

    info!(
        "Sending back authorized url:\n\t{}\n",
        authorize_url.to_string()
    );
    Redirect::to(authorize_url.to_string())
}

#[get("/auth/google?<state>&<code>")]
pub async fn google_callback(state: String, code: String, cookies: &CookieJar<'_>) -> Redirect {
    cookies.add_private(
        Cookie::build("token", code)
            .same_site(SameSite::Lax)
            .finish(),
    );
    Redirect::to("/")
}

#[get("/login/discord")]
pub async fn discord_login(oauth2: OAuth2<Discord>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2
        .get_redirect(cookies, &["identify", "email"])
        .unwrap()
}

#[get("/auth/discord")]
pub async fn discord_callback(token: TokenResponse<Discord>, cookies: &CookieJar<'_>) -> Redirect {
    cookies.add_private(
        Cookie::build("token", token.access_token().to_string())
            .same_site(SameSite::Lax)
            .finish(),
    );
    Redirect::to("/")
}

#[get("/login/hogbisz")]
pub async fn hogbisz_login(oauth2: OAuth2<Hogbisz>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2
        .get_redirect(cookies, &["email", "profile", "roles"])
        .unwrap()
}

#[get("/auth/hogbisz")]
pub async fn hogbisz_callback(token: TokenResponse<Hogbisz>, cookies: &CookieJar<'_>) -> Redirect {
    cookies.add_private(
        Cookie::build("token", token.access_token().to_string())
            .same_site(SameSite::Lax)
            .finish(),
    );
    Redirect::to("/")
}

#[post("/auth/create_user?<email>")]
pub async fn create_user(email: String) -> Status {
    match insert_user(email) {
        Some(user) => {
            info!("Created user: {}", user.email);
            Status::Ok
        }
        None => {
            info!("Failed to create user");
            Status::InternalServerError
        }
    }
}
