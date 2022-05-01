#[macro_use]
extern crate rocket;

extern crate openssl;
#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

embed_migrations!("migrations");

mod auth;
mod util;

use diesel::prelude::*;
use dotenv::dotenv;
use rocket::{fs::NamedFile, routes};
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

#[rocket::main]
async fn main() {
    dotenv().ok();
    // openssl_probe::init_ssl_cert_env_vars();

    // let connection = create_connection().expect("Failed to connect to database");

    // embedded_migrations::run(&connection).expect("Failed to run embedded migrations");

    // std::mem::drop(connection);

    match rocket::build()
        .mount("/", routes![index, files, auth::get_auth, auth::do_work])
        .attach(crate::util::CORS)
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
    let database_url = unwrap_or_return!(env::var("DATABASE_URL"), "Database URL not set.");
    Some(unwrap_or_return!(
        PgConnection::establish(&database_url),
        "Error connecting to database!"
    ))
}
