use crate::{auth::sql::get_user_by_user_id, models::User};

pub fn user_is_admin<T: TryInto<User>>(user_id: T) -> bool {
    let user: User = match user_id.try_into() {
        Ok(user) => user,
        Err(_) => {
            info!("Failed to convert user_id to User");
            return false;
        }
    };

    if !user.permissions.contains(&1) {
        info!(
            "User {} does not have permission to get all users",
            user.user_id
        );
        return false;
    }
    true
}

pub async fn oauth_token_is_valid<T: Into<String>>(oauth: T, token: T, user_id: T) -> bool {
    let oauth_type = oauth.into();
    match oauth_type.as_str() {
        "google" => validate_google_token(token.into(), user_id.into()).await,
        "discord" => validate_discord_token(token.into(), user_id.into()).await,
        "hogbisz" => validate_hogbisz_token(token.into(), user_id.into()).await,
        _ => {
            info!("Provided oauth type {} is not supported", oauth_type);
            false
        }
    }
}

async fn validate_google_token(token: String, user_id: String) -> bool {
    let client = reqwest::Client::new();

    match client
        .get(&format!(
            "https://www.googleapis.com/oauth2/v3/userinfo?access_token={}",
            token
        ))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            info!("Google's response status: {}", status);
            let body = response.text().await.unwrap();
            info!("Google's response body: {}", body);
            if status.is_success() {
                let google_email = match body.split("email\": \"").nth(1) {
                    Some(email) => match email.split("\"").nth(0) {
                        Some(email) => email.to_string(),
                        None => {
                            info!("Failed to get email from Google's response body");
                            return false;
                        }
                    },
                    None => {
                        info!("Failed to get email from Google's response body");
                        return false;
                    }
                };
                info!("Google's email: {}", google_email);
                let user = match get_user_by_user_id(&user_id) {
                    Some(user) => user,
                    None => {
                        info!("No user found with user_id {}", user_id);
                        return false;
                    }
                };

                google_email == user.email
            } else {
                false
            }
        }
        Err(e) => {
            info!("Failed to validate google token with error {}", e);
            false
        }
    }
}

async fn validate_discord_token(token: String, user_id: String) -> bool {
    let client = reqwest::Client::new();

    match client
        .get("https://discordapp.com/api/users/@me")
        .header("Authorization", format!("{} {}", "Bearer", token))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            info!("Discord's response status: {}", status);
            let body = response.text().await.unwrap();
            info!("Discord's response body: {}", body);
            if status.is_success() {
                let discord_email = match body.split("email\": \"").nth(1) {
                    Some(email) => match email.split("\"").nth(0) {
                        Some(email) => email.to_string(),
                        None => {
                            info!("Failed to get email from Discord's response body");
                            return false;
                        }
                    },
                    None => {
                        info!("Failed to get email from Discord's response body");
                        return false;
                    }
                };
                info!("Discord's email: {}", discord_email);
                let user = match get_user_by_user_id(&user_id) {
                    Some(user) => user,
                    None => {
                        info!("No user found with user_id {}", user_id);
                        return false;
                    }
                };
                discord_email == user.email
            } else {
                false
            }
        }
        Err(e) => {
            info!("Failed to validate discord token with error {}", e);
            false
        }
    }
}

// TODO: Implement
async fn validate_hogbisz_token(_token: String, _user_id: String) -> bool {
    false
}
