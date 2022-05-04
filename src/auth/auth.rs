use super::sql::{get_user_by_email, insert_user};
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, RevocationUrl,
    Scope, TokenResponse, TokenUrl,
};
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::response::Redirect;
use rocket_oauth2::OAuth2;
use serde::{Deserialize, Serialize};
use std::env;

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
            "https://www.googleapis.com/auth/userinfo.email".to_string(),
        ))
        .add_scope(Scope::new("openid".to_string()))
        .url();

    info!(
        "Sending back authorized url:\n\t{}\n",
        authorize_url.to_string()
    );
    Redirect::to(authorize_url.to_string())
}

#[get("/auth/google?<state>&<code>")]
#[allow(unused_variables)] // Rocket doesn't like unused variables naming scheme '_state'
pub async fn google_callback(state: String, code: String, cookies: &CookieJar<'_>) -> Redirect {
    cookies.add_private(
        Cookie::build("token", code.clone())
            .same_site(SameSite::Lax)
            .finish(),
    );
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
    let access_token = match client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(oauth2::reqwest::async_http_client)
        .await
    {
        Ok(token_response) => token_response.access_token().clone(),
        Err(e) => {
            error!("Error exchanging code for token: {}", e);
            return Redirect::to("/");
        }
    };

    let client = reqwest::Client::new();
    match client
        .get(&format!(
            "https://www.googleapis.com/oauth2/v3/userinfo?access_token={}",
            access_token.secret()
        ))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                info!("Response was successful! Converting to json");

                #[derive(Deserialize, Serialize, Debug)]
                struct GoogleResponse {
                    sub: String,
                    picture: String,
                    email: String,
                    email_verified: bool,
                }

                let google_response: GoogleResponse;
                match response.json::<GoogleResponse>().await {
                    Ok(json) => {
                        google_response = json;
                    }
                    Err(e) => {
                        warn!("Error converting response to json: {}", e);
                        return Redirect::to("/");
                    }
                }

                info!("Successfully got google response: {:?}", google_response);
                let email = google_response.email;
                let user = if let Some(user) = get_user_by_email(email.to_owned()) {
                    user
                } else {
                    match insert_user(email.to_owned()) {
                        Some(user) => user,
                        None => {
                            warn!("Failed to insert new user {}", email);
                            return Redirect::to("/");
                        }
                    }
                };
                info!("Got user {:?}", user);
                cookies.add(
                    Cookie::build("user_id", user.user_id.clone())
                        .same_site(SameSite::Lax)
                        .finish(),
                );
            } else {
                warn!("Failed to get google response: {:?}", response);
                warn!("{}", response.text().await.unwrap_or("".to_string()));
                return Redirect::to("/");
            }
        }
        Err(e) => warn!("Failed to send google oauth2 request with error: {}", e),
    }

    Redirect::to("/")
}

#[get("/login/discord")]
pub async fn discord_login(oauth2: OAuth2<Discord>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2
        .get_redirect(cookies, &["identify", "email"])
        .unwrap()
}

#[get("/auth/discord")]
pub async fn discord_callback(
    token: rocket_oauth2::TokenResponse<Discord>,
    cookies: &CookieJar<'_>,
) -> Redirect {
    cookies.add_private(
        Cookie::build("token", token.access_token().to_string())
            .same_site(SameSite::Lax)
            .finish(),
    );
    match reqwest::get(&format!(
        "https://discordapp.com/api/users/@me?access_token={}",
        token.access_token()
    ))
    .await
    {
        Ok(response) => {
            let body = response.text().await.unwrap();
            let email = body
                .split("\"email\":")
                .nth(1)
                .unwrap()
                .split(",")
                .next()
                .unwrap();
            let user = if let Some(user) = get_user_by_email(email.to_owned()) {
                user
            } else {
                match insert_user(email.to_owned()) {
                    Some(user) => user,
                    None => {
                        warn!("Failed to insert new user {}", email);
                        return Redirect::to("/");
                    }
                }
            };
            cookies.add(
                Cookie::build("user_id", user.user_id.clone())
                    .same_site(SameSite::Lax)
                    .finish(),
            );
        }
        Err(e) => warn!("{}", e),
    }
    Redirect::to("/")
}

#[get("/login/hogbisz")]
pub async fn hogbisz_login(oauth2: OAuth2<Hogbisz>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2
        .get_redirect(cookies, &["email", "profile", "roles"])
        .unwrap()
}

#[get("/auth/hogbisz")]
pub async fn hogbisz_callback(
    token: rocket_oauth2::TokenResponse<Hogbisz>,
    cookies: &CookieJar<'_>,
) -> Redirect {
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
