use super::sql::{get_user_by_email, insert_user};
use super::util::oauth_token_is_valid;
use crate::auth::sql::get_user_by_user_id;
use crate::{make_json_response, unwrap_or_return_option};
use oauth2::basic::{BasicClient, BasicErrorResponseType, BasicTokenType};
use oauth2::reqwest::async_http_client;
use oauth2::{
    AccessToken, AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken,
    EmptyExtraTokenFields, RedirectUrl, RevocationErrorResponseType, RevocationUrl, Scope,
    StandardErrorResponse, StandardRevocableToken, StandardTokenIntrospectionResponse,
    StandardTokenResponse, TokenResponse, TokenUrl,
};
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::response::content::Json;
use rocket::response::Redirect;
use rocket_oauth2::OAuth2;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

pub type OAuth2Client = Client<
    StandardErrorResponse<BasicErrorResponseType>,
    StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    BasicTokenType,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
>;

pub struct Hogbisz;

fn generate_oauth_client<T: ToString>(
    client_id: T,
    client_secret: T,
    auth_url: T,
    token_url: T,
    redirect_url: T,
    revocation_url: T,
) -> Option<OAuth2Client> {
    let client = BasicClient::new(
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
    let client = unwrap_or_return_option!(
        generate_oauth_client(
            client_id,
            client_secret,
            auth_url,
            token_url,
            redirect_url,
            revocation_url,
        ),
        "Failed to create client."
    );
    let mut auth_request = client.authorize_url(CsrfToken::new_random);
    for scope in scopes {
        auth_request = auth_request.add_scope(Scope::new(scope.to_string()));
    }
    let (authorize_url, _) = auth_request.url();
    Some(authorize_url.to_string())
}

async fn revoke_oauth<T: ToString>(
    token: AccessToken,
    client_id: T,
    client_secret: T,
    auth_url: T,
    token_url: T,
    redirect_url: T,
    revocation_url: T,
) -> bool {
    let client = match generate_oauth_client(
        client_id,
        client_secret,
        auth_url,
        token_url,
        redirect_url,
        revocation_url,
    ) {
        Some(client) => client,
        None => return false,
    };

    match client.revoke_token(token.into()) {
        Ok(c) => match c.request_async(async_http_client).await {
            Ok(_) => true,
            Err(e) => {
                warn!("Failed to revoke client with error {}", e);
                false
            }
        },
        Err(e) => {
            warn!("Failed to revoke client with error {}", e);
            false
        }
    }
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
                + "/api/auth/google",
            "https://oauth2.googleapis.com/revoke".to_string(),
            vec!["https://www.googleapis.com/auth/userinfo.email".to_string()],
        ) {
            Some(url) => url,
            None => "/login".to_string(),
        },
    )
}

#[get("/auth/google?<state>&<code>")]
#[allow(unused_variables)] // Rocket doesn't like unused variables naming scheme '_state'
pub async fn google_callback(state: String, code: String, cookies: &CookieJar<'_>) -> Redirect {
    let failure_redirect = Redirect::to("/login");
    let client = match generate_oauth_client(
        env::var("GOOGLE_CLIENT_ID").expect("Missing the GOOGLE_CLIENT_ID environment variable."),
        env::var("GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
        "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        "https://www.googleapis.com/oauth2/v3/token".to_string(),
        env::var("BASE_URL").expect("Missing the BASE_URL environment variable.")
            + "/api/auth/google",
        "https://oauth2.googleapis.com/revoke".to_string(),
    ) {
        Some(client) => client,
        None => return failure_redirect,
    };
    let token_response = match client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(oauth2::reqwest::async_http_client)
        .await
    {
        Ok(token_response) => token_response,
        Err(e) => {
            error!("Error exchanging code for token: {}", e);
            return failure_redirect;
        }
    };
    let access_token = token_response.access_token();

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
                        return failure_redirect;
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
                            return failure_redirect;
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
                return failure_redirect;
            }
        }
        Err(e) => warn!("Failed to send google oauth2 request with error: {}", e),
    }
    cookies.add(
        Cookie::build("oauth", "google".to_string())
            .same_site(SameSite::Lax)
            .finish(),
    );
    cookies.add_private(
        Cookie::build("token", access_token.secret().clone())
            .same_site(SameSite::Lax)
            .finish(),
    );
    Redirect::to("/")
}

pub async fn google_logout(token: String) -> bool {
    revoke_oauth(
        AccessToken::new(token),
        env::var("GOOGLE_CLIENT_ID").expect("Missing the GOOGLE_CLIENT_ID environment variable."),
        env::var("GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
        "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        "https://www.googleapis.com/oauth2/v3/token".to_string(),
        env::var("BASE_URL").expect("Missing the BASE_URL environment variable.")
            + "/api/auth/google",
        "https://oauth2.googleapis.com/revoke".to_string(),
    )
    .await
}

#[get("/login/discord")]
pub async fn discord_login() -> Redirect {
    Redirect::to(
        match generate_oauth_redirect(
            env::var("DISCORD_CLIENT_ID")
                .expect("Missing the DISCORD_CLIENT_ID environment variable."),
            env::var("DISCORD_CLIENT_SECRET")
                .expect("Missing the DISCORD_CLIENT_SECRET environment variable."),
            "https://discordapp.com/api/oauth2/authorize".to_string(),
            "https://discordapp.com/api/oauth2/token".to_string(),
            env::var("BASE_URL").expect("Missing the BASE_URL environment variable.")
                + "/api/auth/discord",
            "https://discordapp.com/api/oauth2/token/revoke".to_string(),
            vec!["identify".to_string(), "email".to_string()],
        ) {
            Some(url) => url,
            None => String::from("/login"),
        },
    )
}

#[get("/auth/discord?<code>")]
pub async fn discord_callback(code: String, cookies: &CookieJar<'_>) -> Redirect {
    info!("Got discord callback with code: {}", code);
    let failure_redirect = Redirect::to("/login");
    let client = match generate_oauth_client(
        env::var("DISCORD_CLIENT_ID").expect("Missing the DISCORD_CLIENT_ID environment variable."),
        env::var("DISCORD_CLIENT_SECRET")
            .expect("Missing the DISCORD_CLIENT_SECRET environment variable."),
        "https://discordapp.com/api/oauth2/authorize".to_string(),
        "https://discordapp.com/api/oauth2/token".to_string(),
        env::var("BASE_URL").expect("Missing the BASE_URL environment variable.")
            + "/api/auth/discord",
        "https://discordapp.com/api/oauth2/token/revoke".to_string(),
    ) {
        Some(client) => client,
        None => return failure_redirect,
    };
    let access_token = match client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(oauth2::reqwest::async_http_client)
        .await
    {
        Ok(token_response) => token_response.access_token().clone(),
        Err(e) => {
            error!("Error exchanging code for token: {}", e);
            return failure_redirect;
        }
    };
    cookies.add_private(
        Cookie::build("token", access_token.secret().to_owned())
            .same_site(SameSite::Lax)
            .finish(),
    );

    let client = reqwest::Client::new();
    match client
        .get("https://discordapp.com/api/users/@me")
        .header(
            "Authorization",
            format!("{} {}", "Bearer", access_token.secret()),
        )
        .send()
        .await
    {
        Ok(response) => {
            let body = response.text().await.unwrap();
            info!("Got discord response: {}", body);
            let email = match body.split("\"email\":").nth(1) {
                Some(email) => email,
                None => {
                    warn!("Failed to get email from discord response: {}", body);
                    return failure_redirect;
                }
            }
            .split(",")
            .next()
            .unwrap()
            .replace("\"", "")
            .replace(" ", "");
            info!("Got email: {}", email);
            let user = if let Some(user) = get_user_by_email(email.to_owned()) {
                user
            } else {
                match insert_user(email.to_owned()) {
                    Some(user) => user,
                    None => {
                        warn!("Failed to insert new user {}", email);
                        return failure_redirect;
                    }
                }
            };
            info!("Got user {:?}", user);
            cookies.add(
                Cookie::build("user_id", user.user_id.clone())
                    .same_site(SameSite::Lax)
                    .finish(),
            );
        }
        Err(e) => {
            warn!("Failed to get discord api response! Error: {}", e);
            return failure_redirect;
        }
    }
    cookies.add(
        Cookie::build("oauth", "discord".to_string())
            .same_site(SameSite::Lax)
            .finish(),
    );
    Redirect::to("/")
}

async fn discord_logout(token: String) -> bool {
    revoke_oauth(
        AccessToken::new(token),
        env::var("DISCORD_CLIENT_ID").expect("Missing the DISCORD_CLIENT_ID environment variable."),
        env::var("DISCORD_CLIENT_SECRET")
            .expect("Missing the DISCORD_CLIENT_SECRET environment variable."),
        "https://discordapp.com/api/oauth2/authorize".to_string(),
        "https://discordapp.com/api/oauth2/token".to_string(),
        env::var("BASE_URL").expect("Missing the BASE_URL environment variable.")
            + "/api/auth/discord",
        "https://discordapp.com/api/oauth2/token/revoke".to_string(),
    )
    .await
}

// TODO hogbisz oauth update to oauth2 crate
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
    cookies.add(
        Cookie::build("oauth", "hogbisz".to_string())
            .same_site(SameSite::Lax)
            .finish(),
    );
    Redirect::to("/")
}

async fn hogbisz_logout(_token: String) -> bool {
    false
}

#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> Redirect {
    let mut redirect = Redirect::to("/?logout=false");

    if let (Some(oauth_type), Some(token), Some(user_id)) = (
        cookies.get("oauth"),
        cookies.get_private("token"),
        cookies.get("user_id"),
    ) {
        let oauth_type = oauth_type.value().to_string();
        let token = token.value().to_string();
        let user_id = user_id.value().to_string();

        if oauth_token_is_valid(oauth_type.clone(), token.clone(), user_id).await {
            // Implement error response query
            if match oauth_type.as_str() {
                "discord" => discord_logout(token).await,
                "hogbisz" => hogbisz_logout(token).await,
                "google" => google_logout(token).await,
                _ => false,
            } {
                redirect = Redirect::to("/?logout=true");
            }
        }
    }

    cookies.remove_private(Cookie::named("token"));
    cookies.remove(Cookie::named("user_id"));
    cookies.remove(Cookie::named("oauth"));

    redirect
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

#[get("/auth/me")]
pub async fn me(cookies: &CookieJar<'_>) -> Json<String> {
    let oauth_type = match cookies.get("oauth") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            info!("Failed to get oauth type");
            return make_json_response!(401, "Unauthorized");
        }
    };

    let token = match cookies.get_private("token") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            info!("Failed to get oauth token");
            return make_json_response!(401, "Unauthorized");
        }
    };
    let user_id = match cookies.get("user_id") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            info!("No user_id cookie found");
            return make_json_response!(401, "Unauthorized");
        }
    };

    if !oauth_token_is_valid(oauth_type.clone(), token.clone(), user_id.clone()).await {
        info!("Failed to validate oauth token!");
        return make_json_response!(401, "Unauthorized");
    }

    match get_user_by_user_id(&user_id) {
        Some(user) => make_json_response!(200, "OK", user),
        None => make_json_response!(500, "Internal Server Error"),
    }
}
