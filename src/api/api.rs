use crate::{
    create_connection, make_json_response,
    models::{User, Video},
};
use diesel::prelude::*;
use rocket::response::content::Json;
use serde_json::json;

#[get("/users")]
pub async fn get_all_users() -> Json<String> {
    // TODO : Add authentication
    let connection = create_connection().expect("Failed to connect to database");
    let users = match crate::schema::users::table.load::<User>(&connection) {
        Ok(users) => users,
        Err(e) => {
            warn!("Failed to get users (error {})", e);
            return make_json_response!(500, format!("Failed to load users with error {}", e));
        }
    };
    make_json_response!(200, users)
}

#[get("/users?<id>")]
pub async fn get_user_by_id(id: i32) -> Json<String> {
    // TODO : Add authentication
    let connection = create_connection().expect("Failed to connect to database");
    let user = match crate::schema::users::table
        .filter(crate::schema::users::dsl::id.eq(id))
        .first::<User>(&connection)
    {
        Ok(user) => user,
        Err(e) => {
            warn!("Failed to get user with id : {} (error {})", id, e);
            return make_json_response!(
                500,
                format!("Failed to load user with id {} and error {}", id, e)
            );
        }
    };
    make_json_response!(200, user)
}

#[get("/videos")]
pub async fn get_all_videos() -> Json<String> {
    // TODO : Add authentication
    let connection = create_connection().expect("Failed to connect to database");
    let videos = match crate::schema::videos::table.load::<Video>(&connection) {
        Ok(videos) => videos,
        Err(e) => {
            warn!("Failed to get videos (error {})", e);
            return make_json_response!(500, format!("Failed to load videos with error {}", e));
        }
    };
    make_json_response!(200, videos)
}

#[get("/videos?<id>")]
pub async fn get_video_with_id(id: String) -> Json<String> {
    // TODO : Add authentication
    let connection = create_connection().expect("Failed to connect to database");
    let video = match crate::schema::videos::table
        .filter(crate::schema::videos::dsl::video_id.eq(id.clone()))
        .get_results::<Video>(&connection)
    {
        Ok(video) => video,
        Err(e) => {
            let error = format!("Failed to get video with id : {} (error {})", id, e);
            warn!("{}", error);
            return make_json_response!(500, error);
        }
    };
    make_json_response!(200, video)
}
