extern crate diesel;

use crate::schema::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, Identifiable, Debug)]
#[table_name = "user_permissions"]
pub struct UserPermissions {
    pub id: i32,
    pub permission: String,
}

#[derive(Identifiable, Queryable, Serialize, Deserialize, Debug, Default)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub user_id: String,
    pub email: String,
    pub displayname: String,
    pub permissions: Vec<i32>,
}
impl TryFrom<&String> for User {
    type Error = ();
    fn try_from(user_id: &String) -> Result<Self, ()> {
        match crate::auth::sql::get_user_by_user_id(user_id) {
            Some(user) => Ok(user),
            None => {
                info!(
                    "Failed to get user with user_id {} when converting from string",
                    user_id
                );
                Err(())
            }
        }
    }
}
impl TryFrom<i32> for User {
    type Error = ();
    fn try_from(user_id: i32) -> Result<Self, ()> {
        match crate::auth::sql::get_user_by_id(user_id) {
            Some(user) => Ok(user),
            None => {
                info!(
                    "Failed to get user with id {} when converting from i32",
                    user_id
                );
                Err(())
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Insertable)]
#[table_name = "users"]
pub struct UserNoId {
    pub user_id: String,
    pub email: String,
    pub displayname: String,
}

#[derive(Identifiable, Queryable, Associations, Debug, Serialize, Deserialize)]
#[belongs_to(User, foreign_key = "owner_id")]
#[table_name = "videos"]
pub struct Video {
    pub id: i32,
    pub video_id: String,
    pub video_path: String,
    pub video_url: String,
    pub video_name: String,
    pub video_length: f64,
    pub video_desc: String,
    pub owner_id: i32,
    pub thumbnail_path: Option<String>,
}

#[derive(Insertable, Debug, Serialize, Deserialize)]
#[table_name = "videos"]
pub struct VideoNoId {
    pub video_id: String,
    pub video_path: String,
    pub video_url: String,
    pub video_name: String,
    pub video_length: f64,
    pub video_desc: String,
    pub owner_id: i32,
    pub thumbnail_path: Option<String>,
}
