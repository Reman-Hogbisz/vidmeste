extern crate diesel;

use std::path::PathBuf;

use crate::{create_connection, models::*};
use diesel::prelude::*;
use rand::Rng;

pub fn get_video_by_id(id: &String) -> Option<PathBuf> {
    let connection = create_connection().expect("Failed to connect to database");
    match crate::schema::videos::table
        .filter(crate::schema::videos::dsl::video_id.eq(id.to_owned()))
        .first::<Video>(&connection)
    {
        Ok(video) => Some(PathBuf::from(video.video_path)),
        Err(e) => {
            info!(
                "Failed to get video with id : {} (error {})",
                id.to_owned(),
                e
            );
            None
        }
    }
}

fn get_video_by_id_no_error(id: &String) -> Option<PathBuf> {
    let connection = create_connection().expect("Failed to connect to database");
    match crate::schema::videos::table
        .filter(crate::schema::videos::dsl::video_id.eq(id.to_owned()))
        .first::<Video>(&connection)
    {
        Ok(video) => Some(PathBuf::from(video.video_path)),
        Err(_) => None,
    }
}

pub fn insert_new_video(video: &VideoNoId) -> Option<Video> {
    let connection = create_connection().expect("Failed to connect to database");
    match diesel::insert_into(crate::schema::videos::table)
        .values(video)
        .get_result::<Video>(&connection)
    {
        Ok(video) => Some(video),
        Err(e) => {
            info!(
                "Failed to insert video with id : {} (error {})",
                video.video_id, e
            );
            None
        }
    }
}

/// Generates a new video id that does not exist in the database
pub fn generate_new_video_id() -> String {
    let mut video_id = make_random_string(32);
    while let Some(_) = get_video_by_id_no_error(&video_id) {
        video_id = make_random_string(32);
    }
    video_id
}

fn make_random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
    let mut thread_rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = thread_rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
