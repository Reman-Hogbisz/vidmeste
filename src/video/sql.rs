extern crate diesel;

use std::path::PathBuf;

use crate::{create_connection, models::*, util::make_random_string};
use diesel::prelude::*;

pub fn get_video_by_id(id: i32) -> Option<Video> {
    let connection = create_connection().expect("Failed to connect to database");
    match crate::schema::videos::table
        .filter(crate::schema::videos::dsl::id.eq(id.to_owned()))
        .first::<Video>(&connection)
    {
        Ok(video) => Some(video),
        Err(e) => {
            info!("Failed to get video with id : {} (error {})", id, e);
            None
        }
    }
}

pub fn get_video_by_video_id(id: &String) -> Option<Video> {
    let connection = create_connection().expect("Failed to connect to database");
    match crate::schema::videos::table
        .filter(crate::schema::videos::dsl::video_id.eq(id.to_owned()))
        .first::<Video>(&connection)
    {
        Ok(video) => Some(video),
        Err(e) => {
            info!(
                "Failed to get video with video id : {} (error {})",
                id.to_owned(),
                e
            );
            None
        }
    }
}

fn get_video_by_video_id_no_error(id: &String) -> Option<PathBuf> {
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

pub fn delete_video_with_video_id<T: Into<String>>(id: T) -> bool {
    let connection = match crate::create_connection() {
        Some(c) => c,
        None => return false,
    };
    match diesel::delete(
        crate::schema::videos::table.filter(crate::schema::videos::dsl::video_id.eq(id.into())),
    )
    .execute(&connection)
    {
        Ok(_) => true,
        Err(e) => {
            info!("Failed to delete video from database with error {}", e);
            false
        }
    }
}

pub fn delete_video_with_id<T: Into<i32>>(id: T) -> bool {
    let connection = match crate::create_connection() {
        Some(c) => c,
        None => return false,
    };
    match diesel::delete(
        crate::schema::videos::table.filter(crate::schema::videos::dsl::id.eq(id.into())),
    )
    .execute(&connection)
    {
        Ok(_) => true,
        Err(e) => {
            info!("Failed to delete video from database with error {}", e);
            false
        }
    }
}

/// Generates a new video id that does not exist in the database
pub fn generate_new_video_id() -> String {
    let mut video_id = make_random_string(32);
    while let Some(_) = get_video_by_video_id_no_error(&video_id) {
        video_id = make_random_string(32);
    }
    video_id
}
