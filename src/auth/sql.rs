use crate::{
    create_connection,
    models::{User, UserNoId},
};
use diesel::prelude::*;

pub fn insert_user(email: String) -> Option<User> {
    let connection = create_connection().expect("Failed to connect to database");
    match diesel::insert_into(crate::schema::users::table)
        .values(&UserNoId { email })
        .get_result::<User>(&connection)
    {
        Ok(user) => Some(user),
        Err(e) => {
            info!("Failed to insert user (error {})", e);
            None
        }
    }
}
