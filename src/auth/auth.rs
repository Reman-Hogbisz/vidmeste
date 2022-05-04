use crate::unwrap_or_return_option;

use super::sql::{get_user_by_email, insert_user};
use oauth2::basic::{BasicClient, BasicErrorResponseType, BasicTokenType};
use oauth2::{
    AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    RedirectUrl, RevocationErrorResponseType, RevocationUrl, Scope, StandardErrorResponse,
    StandardRevocableToken, StandardTokenIntrospectionResponse, StandardTokenResponse,
    TokenResponse, TokenUrl,
};
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::response::Redirect;
use rocket_oauth2::OAuth2;
use serde::{Deserialize, Serialize};
use std::env;

pub struct Google;
pub struct Hogbisz;
pub struct Discord;

fn generate_oauth_client<T: ToString>(
    client_id: T,
    client_secret: T,
    auth_url: T,
    token_url: T,
    redirect_url: T,
    revocation_url: T,
) -> Option<
    Client<
        StandardErrorResponse<BasicErrorResponseType>,
        StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
        BasicTokenType,
        StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
        StandardRevocableToken,
        StandardErrorResponse<RevocationErrorResponseType>,
    >,
> {
    let mut client = BasicClient::new(
        ClientId::new(client_id.to_string()),
        Some(ClientSecret::new(client_secret.to_string())),
        unwrap_or_return_option!(AuthUrl::new(auth_url.to_string()).ok(), "Invalid auth URL."),
        Some(unwrap_or_return_option!(
            TokenUrl::new(token_url.to_string()).ok(),
            "Failed to create token url"
        )),
    );
    let client = client
        .set_redirect_uri(unwrap_or_return_option!(
            RedirectUrl::new(redirect_url.to_string()).ok(),
            "Invalid redirect URL."
        ))
        .set_revocation_uri(unwrap_or_return_option!(
            RevocationUrl::new(revocation_url.to_string()).ok(),
            "Invalid revocation URL."
        ));
    Some(client)
}

fn generate_oauth_redirect<T: ToString>(
    client_id: T,
    client_secret: T,
    auth_url: T,
    token_url: T,
    redirect_url: T,
    revocation_url: T,
    scopes: Vec<T>,
) -> Option<String> {
    let client = unwrap_or_return_option!(generate_oauth_client(
        client_id,
        client_secret,
        auth_url,
        token_url,
        redirect_url,
        revocation_url,
    ));
    let mut auth_request = client.authorize_url(CsrfToken::new_random);
    for scope in scopes {
        auth_request = auth_request.add_scope(Scope::new(scope.to_string()));
    }
    let (authorize_url, _) = auth_request.url();
    Some(authorize_url.to_string())
}

#[get("/login/google")]
pub async fn google_login() -> Redirect {
    Redirect::to(
        match generate_oauth_redirect(
            env::var("GOOGLE_CLIENT_ID")
                .expect("Missing the GOOGLE_CLIENT_ID environment variable."),
            env::var("GOOGLE_CLIENT_SECRET")
                .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
            "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            "https://www.googleapis.com/oauth2/v3/token".to_string(),
            env::var("BASE_URL").expect("Missing the BASE_URL environment variable.")
                + "/auth/google",
            "https://oauth2.googleapis.com/revoke".to_string(),
            vec!["https://www.googleapis.com/auth/userinfo.email".to_string()],
        ) {
            Some(url) => url,
            None => return Redirect::to("."),
        },
    )
}

#[get("/auth/google?<state>&<code>")]
#[allow(unused_variables)] // Rocket doesn't like unused variables naming scheme '_state'
pub async fn google_callback(state: String, code: String, cookies: &CookieJar<'_>) -> Redirect {
    cookies.add_private(
        Cookie::build("token", code.clone())
            .same_site(SameSite::Lax)
            .finish(),
    );
    let client = generate_oauth_client(
        env::var("GOOGLE_CLIENT_ID").expect("Missing the GOOGLE_CLIENT_ID environment variable."),
        env::var("GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
        "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        "https://www.googleapis.com/oauth2/v3/token".to_string(),
        env::var("BASE_URL").expect("Missing the BASE_URL environment variable.") + "/auth/google",
        "https://oauth2.googleapis.com/revoke".to_string(),
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
