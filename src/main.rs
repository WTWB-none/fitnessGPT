use rocket::{get, routes};
use rocket::fs::FileServer;
use rocket::http::Method;
use rocket_cors::{AllowedOrigins, CorsOptions};
use sqlx::PgPool;
use dotenv::dotenv;
use std::env;
use log::info;

mod api;
mod entity;
mod validation;
mod services;
mod config;

use crate::api::handlers::{create_user, login, get_user, create_profile};
use crate::config::config::AppConfig;

#[get("/")]
fn index() -> &'static str {
    "Main page!"
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("Запуск приложения...");

    dotenv().ok();
    let config = AppConfig::from_env()?;
    let pool = PgPool::connect(&config.database_url).await?;

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true);

    info!("Сервер запускается на порту 8000...");
    let _rocket = rocket::build()
        .manage(pool)
        .manage(config)
        .mount(
            "/",
            routes![index, create_user, login, get_user, create_profile],
        )
        .mount("/static", FileServer::from("static"))
        .attach(cors.to_cors()?)
        .launch()
        .await?;

    Ok(())
}