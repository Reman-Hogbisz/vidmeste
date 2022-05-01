use oauth2::reqwest::async_http_client;
use oauth2::{basic::BasicClient, revocation::StandardRevocableToken, TokenResponse};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, RevocationUrl,
    Scope, TokenUrl,
};
use rocket::response::content::Json;
use serde_json::json;
use std::borrow::Cow;
use std::env;

use crate::make_json_response;

#[get("/me")]
pub async fn get_auth() -> Json<String> {
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

    // Set up the config for the Google OAuth2 process.
    let client = BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    // This example will be running its own server at localhost:9999.
    // See below for the server implementation.
    .set_redirect_uri(
        RedirectUrl::new("https://vidmeste.hogbisz.com/me/s2".to_string())
            .expect("Invalid redirect URL"),
    )
    // Google supports OAuth 2.0 Token Revocation (RFC-7009)
    .set_revocation_uri(
        RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
            .expect("Invalid revocation endpoint URL"),
    );

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        // This example is requesting access to the "calendar" features and the user's profile.
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/calendar".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/plus.me".to_string(),
        ))
        .url();

    info!(
        "Google returned the following state:\n\t{}\n",
        csrf_state.secret()
    );

    info!(
        "Sending back authorized url:\n\t{}\n",
        authorize_url.to_string()
    );
    make_json_response!(200, authorize_url)
}

#[get("/me/s2?<code>&<state>")]
pub async fn do_work(code: String, state: String) -> Json<String> {
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

    // Set up the config for the Google OAuth2 process.
    let client = BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    );

    let code = AuthorizationCode::new(code);
    let state = CsrfToken::new(state);

    info!(
        "Received code: {} with state {}",
        code.secret(),
        state.secret()
    );

    info!("Google returned the following code:\n\t{}\n", code.secret());

    // Exchange the code with a token.
    let token_response = client
        .exchange_code(code)
        .set_redirect_uri(Cow::Borrowed(
            &RedirectUrl::new("https://vidmeste.hogbisz.com/me/s2".to_string())
                .expect("Invalid redirect URL"),
        ))
        .add_extra_param("revocation", value)
        .request_async(async_http_client)
        .await;

    info!(
        "Google returned the following token:\n\t{:?}\n",
        token_response
    );

    // Revoke the obtained token
    let token_response = token_response.unwrap();
    let token_to_revoke: StandardRevocableToken = match token_response.refresh_token() {
        Some(token) => token.into(),
        None => token_response.access_token().into(),
    };

    client
        .revoke_token(token_to_revoke)
        .unwrap()
        .request_async(async_http_client)
        .await
        .expect("Failed to revoke token");
    make_json_response!(200, "OK! Go back to your terminal :)")
}
