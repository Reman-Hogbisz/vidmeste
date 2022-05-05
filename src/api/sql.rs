use crate::{create_connection, models::Video};
use diesel::prelude::*;

pub fn get_video_with_id(id: &String) -> Option<Video> {
    let connection = create_connection().expect("Failed to connect to database");
    match crate::schema::videos::table
        .filter(crate::schema::videos::dsl::video_id.eq(id.clone()))
        .get_result::<Video>(&connection)
    {
        Ok(video) => Some(video),
        Err(e) => {
            if e == diesel::NotFound {
                info!("Failed to get video with id {} with error {}", id, e);
            } else {
                warn!("Failed to get video with id : {} (error {})", id, e);
            }
            None
        }
    }
}

pub fn get_all_videos() -> Option<Vec<Video>> {
    let connection = create_connection().expect("Failed to connect to database");
    match crate::schema::videos::table.load::<Video>(&connection) {
        Ok(videos) => Some(videos),
        Err(e) => {
            if e == diesel::NotFound {
                info!("Failed to get videos with error {}", e);
            } else {
                warn!("Failed to get videos (error {})", e);
            }
            None
        }
    }
}
