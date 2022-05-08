#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel_migrations;
embed_migrations!("migrations");

#[macro_use]
extern crate diesel;

extern crate openssl;

pub mod api;
pub mod auth;
pub mod models;
pub mod schema;
pub mod util;
pub mod video;

use diesel::prelude::*;
use dotenv::dotenv;
use rocket::{fs::NamedFile, response::Redirect, routes};
use rocket_oauth2::OAuth2;
use std::{
    env,
    path::{Path, PathBuf},
};

#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("static/index.html")).await.ok()
}

#[get("/<file..>")]
async fn files(file: PathBuf) -> Option<NamedFile> {
    match NamedFile::open(Path::new("static/").join(file)).await {
        Ok(file) => Some(file),
        Err(_) => NamedFile::open(Path::new("static/index.html")).await.ok(),
    }
}

#[catch(404)]
async fn not_found_catcher() -> Redirect {
    Redirect::to("/404")
}

#[rocket::main]
async fn main() {
    dotenv().ok();
    openssl_probe::init_ssl_cert_env_vars();

    let connection = create_connection().expect("Failed to connect to database");

    embedded_migrations::run(&connection).expect("Failed to run embedded migrations");

    std::mem::drop(connection);

    match rocket::build()
        .mount("/", routes![index, files,])
        .mount(
            "/api",
            routes![
                crate::api::api::get_all_users,
                crate::auth::auth::create_user,
                crate::api::api::get_user_by_id,
                crate::api::api::get_all_videos,
                crate::api::api::get_video_with_id,
                crate::auth::auth::me,
                crate::auth::auth::google_login,
                crate::auth::auth::google_callback,
                crate::auth::auth::hogbisz_login,
                crate::auth::auth::hogbisz_callback,
                crate::auth::auth::discord_login,
                crate::auth::auth::discord_callback,
                crate::auth::auth::logout,
            ],
        )
        .mount(
            "/api/video",
            routes![
                crate::video::public::get_video,
                crate::video::public::add_video,
                crate::video::public::delete_video,
                crate::video::public::get_video_info,
            ],
        )
        .register("/", catchers![not_found_catcher])
        .attach(crate::util::CORS)
        .attach(OAuth2::<crate::auth::auth::Hogbisz>::fairing("hogbisz"))
        .launch()
        .await
    {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

fn create_connection() -> Option<PgConnection> {
    let database_url = unwrap_or_return_result!(env::var("DATABASE_URL"), "Database URL not set.");
    Some(unwrap_or_return_result!(
        PgConnection::establish(&database_url),
        "Error connecting to database!"
    ))
}
