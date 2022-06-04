use crate::auth::{sql, util};
use crate::{auth::sql::dump_user_table, make_json_response};
use rocket::http::CookieJar;
use rocket::response::content::RawJson;
use serde_json::json;

#[get("/users")]
pub async fn get_all_users(cookies: &CookieJar<'_>) -> RawJson<String> {
    let user_id = match cookies.get("user_id") {
        Some(user_id) => user_id.value().to_string(),
        None => {
            info!("Failed to get user_id from cookies");
            return make_json_response!(401, "Unauthorized");
        }
    };

    if !util::user_is_admin(&user_id) {
        info!("User {} does not have permission to get all users", user_id);
        return make_json_response!(403, "Forbidden");
    }

    match dump_user_table() {
        Some(users) => make_json_response!(200, "OK", users),
        None => make_json_response!(500, "Failed to dump table"),
    }
}

#[get("/users?<id>")]
pub async fn get_user_by_id(id: i32, cookies: &CookieJar<'_>) -> RawJson<String> {
    let user_id = match cookies.get("user_id") {
        Some(user_id) => user_id.value().to_string(),
        None => {
            info!("Failed to get user_id from cookies");
            return make_json_response!(401, "Unauthorized");
        }
    };

    if !util::user_is_admin(&user_id) {
        info!("User {} does not have permission to get all users", user_id);
        return make_json_response!(403, "Forbidden");
    }

    match sql::get_user_by_id(id) {
        Some(user) => make_json_response!(200, "OK", user),
        None => make_json_response!(404, "User not found"),
    }
}

#[get("/videos")]
pub async fn get_all_videos(cookies: &CookieJar<'_>) -> RawJson<String> {
    let user_id = match cookies.get("user_id") {
        Some(user_id) => user_id.value().to_string(),
        None => {
            info!("Failed to get user_id from cookies");
            return make_json_response!(401, "Unauthorized");
        }
    };

    if !util::user_is_admin(&user_id) {
        info!(
            "User {} does not have permission to get all videos",
            user_id
        );
        return make_json_response!(403, "Forbidden");
    }

    match crate::api::sql::get_all_videos() {
        Some(videos) => make_json_response!(200, "OK", videos),
        None => make_json_response!(500, "Failed to load videos"),
    }
}

#[get("/videos?<id>")]
pub async fn get_video_with_id(id: String, cookies: &CookieJar<'_>) -> RawJson<String> {
    let user_id = match cookies.get("user_id") {
        Some(user_id) => user_id.value().to_string(),
        None => {
            info!("Failed to get user_id from cookies");
            return make_json_response!(401, "Unauthorized");
        }
    };

    let user = match sql::get_user_by_user_id(&user_id) {
        Some(user) => user,
        None => {
            info!("Failed to get user with user_id {}", user_id);
            return make_json_response!(401, "Unauthorized");
        }
    };

    match crate::api::sql::get_video_with_id(&id) {
        Some(v) => {
            if v.owner_id != user.id && !util::user_is_admin(user) {
                info!(
                    "User {} does not have permission to view video {}",
                    user_id, id
                );
                return make_json_response!(403, "Forbidden");
            }
            make_json_response!(200, "OK", v)
        }
        None => make_json_response!(404, "Video not found"),
    }
}
