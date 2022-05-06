use crate::{
    auth::{sql::get_user_by_user_id, util::oauth_token_is_valid},
    make_json_response,
    models::{Video, VideoNoId},
    unwrap_or_return_option,
    video::sql::{generate_new_video_id, get_video_by_id, insert_new_video},
};
use rocket::response::content::Json;
use rocket::{
    data::{Data, ToByteUnit},
    http::CookieJar,
};
use rocket_seek_stream::SeekStream;
use sanitize_html::rules::predefined::DEFAULT;
use sanitize_html::sanitize_str;
use serde_json::json;
use std::path::PathBuf;

use super::util::{get_filename_ending, valid_video_filename_ending};

#[get("/get?<id>&<one_time>")]
#[allow(unused_variables)]
pub async fn get_video<'a>(
    id: String,
    one_time: Option<String>,
    cookies: &CookieJar<'_>,
) -> Option<SeekStream<'a>> {
    // TODO : Implement authentication

    let user_id = match cookies.get("user_id") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            info!("No user_id cookie found");
            return None;
        }
    };
    let user = match get_user_by_user_id(&user_id) {
        Some(user) => user,
        None => {
            info!("No user found with user_id {}", user_id);
            return None;
        }
    };

    let video: Video = unwrap_or_return_option!(get_video_by_id(&id), "Video not found");

    if video.owner_id != user.id && !crate::auth::util::user_is_admin(user) {
        // TODO : One time password
        return None;
    }

    SeekStream::from_path(video.video_path).ok()
}

#[post("/add?<name>", data = "<video>")]
pub async fn add_video(name: String, video: Data<'_>, cookies: &CookieJar<'_>) -> Json<String> {
    let user_id = match cookies.get("user_id") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            info!("No user_id cookie found");
            return make_json_response!(401, "Unauthorized");
        }
    };
    let oauth_type = match cookies.get("oauth") {
        Some(cookie) => cookie.value().to_string(),
        None => return make_json_response!(401, "Unauthorized"),
    };

    let token = match cookies.get_private("token") {
        Some(cookie) => cookie.value().to_string(),
        None => return make_json_response!(401, "Unauthorized"),
    };

    if !oauth_token_is_valid(oauth_type.clone(), token.clone(), user_id.clone()).await {
        return make_json_response!(401, "Unauthorized");
    }

    let user = match get_user_by_user_id(&user_id) {
        Some(user) => user,
        None => {
            info!("No user found with user_id {}", user_id);
            return make_json_response!(401, "Unauthorized");
        }
    };

    let name = name.replace("..", "").replace("/", "");

    let mut name_sanitized = match sanitize_str(&DEFAULT, &name) {
        Ok(name_sanitized) => name_sanitized.replace("..", "").replace("/", ""),
        Err(e) => {
            warn!("Failed to sanitize name {} with error: {}", name, e);
            return make_json_response!(500, "Internal Server Error");
        }
    };

    if !valid_video_filename_ending(&name_sanitized) {
        info!("Invalid video filename {}", name_sanitized);
        return make_json_response!(400, "Bad Request");
    }

    // Safe unwrap, would have returned bad request if None
    let ending = get_filename_ending(&name_sanitized).unwrap();

    if name_sanitized.len() > 128 {
        info!("Name too long. Cutting off at 128 characters");
        name_sanitized.truncate(128);
        name_sanitized += &ending;
    }

    let video_file_stream = video.open(512i64.mebibytes());
    let folder = format!("videos/{}", user.id);
    if !std::path::Path::new(&folder).exists() {
        match rocket::tokio::fs::create_dir_all(&folder).await {
            Ok(_) => (),
            Err(e) => {
                warn!("Failed to create folder {} with error: {}", folder, e);
                return make_json_response!(500, "Internal Server Error");
            }
        }
    }
    let file_path = format!("{}/{}", folder, name_sanitized);
    let file_path_buf = PathBuf::from(file_path.clone());
    let file_out = match rocket::tokio::fs::File::create(file_path_buf.clone()).await {
        Ok(file_out) => file_out,
        Err(e) => {
            warn!(
                "Failed to create file {} with error: {}",
                file_path_buf.to_str().unwrap_or("\"Failed on unwrap\""),
                e
            );
            return make_json_response!(500, "Internal Server Error");
        }
    };
    match video_file_stream.stream_to(file_out).await {
        Ok(_) => {
            let video = VideoNoId {
                owner_id: 1,
                video_id: generate_new_video_id(),
                video_path: file_path.clone(),
                video_name: name_sanitized,
                // TODO : Optimize
                video_length: match ffprobe::ffprobe(&file_path) {
                    Ok(probe) => {
                        let duration = match &probe.streams[0].duration {
                            Some(duration) => duration.to_owned(),
                            None => {
                                warn!("Failed to get duration from video");
                                String::from("-1")
                            }
                        };
                        match duration.parse::<i32>() {
                            Ok(duration) => duration,
                            Err(e) => {
                                warn!("Failed to parse duration from video with error: {}", e);
                                -1
                            }
                        }
                    }
                    Err(_) => {
                        warn!("Failed to probe for video length");
                        -1
                    }
                },
                video_desc: String::default(),
                thumbnail_path: None,
            };
            match insert_new_video(&video) {
                Some(_) => make_json_response!(200, "Ok", video),
                None => make_json_response!(500, "Internal Server Error"),
            }
        }
        Err(e) => {
            warn!(
                "Failed to add video {} with error : {}",
                file_path_buf.to_str().unwrap_or("\"Failed on unwrap\""),
                e
            );
            match rocket::tokio::fs::remove_file(file_path_buf.clone()).await {
                Ok(_) => (),
                Err(e) => warn!(
                    "Failed to remove file {} with error : {}",
                    file_path_buf.to_str().unwrap_or("\"Failed on unwrap\""),
                    e
                ),
            }
            make_json_response!(500, "Internal Server Error")
        }
    }
}

#[post("/delete?<id>")]
pub async fn delete_video(id: String, cookies: &CookieJar<'_>) -> Json<String> {
    let user_id = match cookies.get("user_id") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            info!("No user_id cookie found");
            return make_json_response!(401, "Unauthorized");
        }
    };
    let oauth_type = match cookies.get("oauth") {
        Some(cookie) => cookie.value().to_string(),
        None => return make_json_response!(401, "Unauthorized"),
    };

    let token = match cookies.get_private("token") {
        Some(cookie) => cookie.value().to_string(),
        None => return make_json_response!(401, "Unauthorized"),
    };

    if !oauth_token_is_valid(oauth_type.clone(), token.clone(), user_id.clone()).await {
        return make_json_response!(401, "Unauthorized");
    }

    let user = match crate::auth::sql::get_user_by_user_id(&user_id) {
        Some(u) => u,
        None => {
            info!("Failed to get user with id {}", user_id);
            return make_json_response!(401, "Unauthorized");
        }
    };

    let video = match get_video_by_id(&id) {
        Some(video) => video,
        None => {
            info!("Could not find video with id {}", id);
            return make_json_response!(400, "Bad Request");
        }
    };

    if video.owner_id != user.id && !crate::auth::util::user_is_admin(&user.user_id) {
        info!("User did not own video that was attempted to be deleted.");
        return make_json_response!(401, "Unauthorized");
    }

    if !crate::video::sql::delete_video_with_id(video.id) {
        info!("Failed to remove video from database.");
        return make_json_response!(500, "Internal Server Error");
    }

    match rocket::tokio::fs::remove_file(video.video_path.clone()).await {
        Ok(_) => (),
        Err(e) => {
            warn!("Failed to delete video after removing video from database! (error {}) Please find it here: {}", e, video.video_path);
            return make_json_response!(500, "Internal Server Error");
        }
    }

    make_json_response!(200, "Ok")
}
