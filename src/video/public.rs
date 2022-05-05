use crate::{
    models::VideoNoId,
    unwrap_or_return_option,
    video::sql::{generate_new_video_id, get_video_by_id, insert_new_video},
};
use rocket::http::Status;
use rocket::{
    data::{Data, ToByteUnit},
    http::CookieJar,
};
use rocket_seek_stream::SeekStream;
use sanitize_html::rules::predefined::DEFAULT;
use sanitize_html::sanitize_str;
use std::path::PathBuf;

#[get("/get?<id>")]
pub async fn get_video<'a>(id: String) -> Option<SeekStream<'a>> {
    // TODO : Implement authentication
    let video_path: PathBuf = unwrap_or_return_option!(get_video_by_id(&id), "Video not found");

    SeekStream::from_path(video_path).ok()
}

#[post("/add?<name>", data = "<video>")]
pub async fn add_video(name: String, video: Data<'_>, cookies: &CookieJar<'_>) -> Status {
    let name_sanitized = match sanitize_str(&DEFAULT, &name) {
        Ok(name_sanitized) => name_sanitized.replace("..", "").replace(".", "_"),
        Err(e) => {
            warn!("Failed to sanitize name {} with error: {}", name, e);
            return Status::InternalServerError;
        }
    };
    let video_file_stream = video.open(512i64.mebibytes());
    let file_path = format!("videos/{}/{}.mp4", user_id, name);
    let file_path_buf = PathBuf::from(file_path.clone());
    let file_out = match rocket::tokio::fs::File::create(file_path_buf.clone()).await {
        Ok(file_out) => file_out,
        Err(e) => {
            warn!(
                "Failed to create file {} with error: {}",
                file_path_buf.to_str().unwrap_or("\"Failed on unwrap\""),
                e
            );
            return Status::InternalServerError;
        }
    };
    match video_file_stream.stream_to(file_out).await {
        Ok(_) => {
            let video = VideoNoId {
                owner_id: 1,
                video_id: generate_new_video_id(),
                video_path: file_path.clone(),
                video_name: name_sanitized + ".mp4",
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
                Some(_) => Status::Ok,
                None => Status::InternalServerError,
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
            Status::InternalServerError
        }
    }
}
