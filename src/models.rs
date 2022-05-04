extern crate diesel;

use crate::schema::*;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Identifiable, Queryable, Serialize, Deserialize, Debug)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Insertable)]
#[table_name = "users"]
pub struct UserNoId {
    pub email: String,
}

#[derive(Identifiable, Queryable, Associations, Debug, Serialize, Deserialize)]
#[belongs_to(User, foreign_key = "owner_id")]
#[table_name = "videos"]
pub struct Video {
    pub id: i32,
    pub video_id: String,
    pub video_path: String,
    pub video_name: String,
    pub video_length: i32,
    pub video_desc: String,
    pub owner_id: i32,
    pub thumbnail_path: Option<String>,
}

#[derive(Insertable, Debug, Serialize, Deserialize)]
#[table_name = "videos"]
pub struct VideoNoId {
    pub video_id: String,
    pub video_path: String,
    pub video_name: String,
    pub video_length: i32,
    pub video_desc: String,
    pub owner_id: i32,
    pub thumbnail_path: Option<String>,
}
