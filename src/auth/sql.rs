use crate::{
    create_connection,
    models::{User, UserNoId},
    util::make_random_string,
};
use diesel::prelude::*;

pub fn generate_new_user_id() -> String {
    let mut user_id = make_random_string(32);
    while let Some(_) = get_user_by_user_id_no_error(&user_id) {
        user_id = make_random_string(32);
    }
    user_id
}

pub fn insert_user(email: String) -> Option<User> {
    let connection = create_connection().expect("Failed to connect to database");
    match diesel::insert_into(crate::schema::users::table)
        .values(&UserNoId {
            email: email.clone(),
            displayname: email.clone(),
            user_id: generate_new_user_id(),
        })
        .get_result::<User>(&connection)
    {
        Ok(user) => Some(user),
        Err(e) => {
            info!("Failed to insert user (error {})", e);
            None
        }
    }
}

fn get_user_by_user_id_no_error(user_id: &String) -> Option<User> {
    let connection = create_connection().expect("Failed to connect to database");
    match crate::schema::users::table
        .filter(crate::schema::users::dsl::user_id.eq(user_id.to_owned()))
        .first::<User>(&connection)
    {
        Ok(user) => Some(user),
        Err(_) => None,
    }
}

pub fn get_user_by_email(email: String) -> Option<User> {
    let connection = match crate::create_connection() {
        Some(connection) => connection,
        None => {
            warn!("Failed to get connection to database");
            return None;
        }
    };
    match crate::schema::users::table
        .filter(crate::schema::users::dsl::email.eq(email.to_owned()))
        .get_result::<User>(&connection)
    {
        Ok(user) => Some(user),
        Err(e) => {
            if e != diesel::NotFound {
                warn!("Failed to get user {} with error {}", email, e);
            }
            None
        }
    }
}

pub fn get_user_by_user_id(user_id: &String) -> Option<User> {
    let connection = match crate::create_connection() {
        Some(connection) => connection,
        None => {
            warn!("Failed to get connection to database");
            return None;
        }
    };
    match crate::schema::users::table
        .filter(crate::schema::users::dsl::user_id.eq(user_id.to_owned()))
        .get_result::<User>(&connection)
    {
        Ok(user) => Some(user),
        Err(e) => {
            if e != diesel::NotFound {
                warn!("Failed to get user {} with error {}", user_id, e);
            }
            None
        }
    }
}

pub fn get_user_by_id(id: i32) -> Option<User> {
    let connection = match crate::create_connection() {
        Some(connection) => connection,
        None => {
            warn!("Failed to get connection to database");
            return None;
        }
    };
    match crate::schema::users::table
        .filter(crate::schema::users::dsl::id.eq(id))
        .get_result::<User>(&connection)
    {
        Ok(user) => Some(user),
        Err(e) => {
            if e != diesel::NotFound {
                warn!("Failed to get user {} with error {}", id, e);
            }
            None
        }
    }
}

pub fn dump_user_table() -> Option<Vec<User>> {
    let connection = match crate::create_connection() {
        Some(connection) => connection,
        None => {
            warn!("Failed to get connection to database");
            return None;
        }
    };
    match crate::schema::users::table.load::<User>(&connection) {
        Ok(users) => Some(users),
        Err(e) => {
            if e != diesel::NotFound {
                warn!("Failed to get users with error {}", e);
            }
            None
        }
    }
}
