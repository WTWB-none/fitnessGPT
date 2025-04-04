use rocket::{get, post, routes};
use rocket::fs::FileServer;
use rocket::http::{Cookie, CookieJar};
use rocket::serde::json::Json;

use surrealdb::Surreal;
use surrealdb::engine::local::{RocksDb, Db};

mod entity;

async fn init_db() -> surrealdb::Result<Surreal<Db>> {
    let db = Surreal::new::<RocksDb>("data/db").await?;
    db.use_ns("data").use_db("data").await?;
    Ok(db)
}

#[post("/create_user", data = "<user>")]
async fn create_user(user: Json<entity::User>) -> Json<String> {
    let user_data = user.into_inner();
    println!("Данные: {:?}", user_data);

    let db = init_db().await.unwrap();
    let query = format!(
        "CREATE user SET name = '{}', email = '{}', password = '{}', phone = {}",
        user_data.name, user_data.email, user_data.password, user_data.phone
    );
    db.query(query).await.unwrap();

    Json(format!("Пользователь создан: {}", user_data.name))
}

#[get("/get_user")]
async fn get_user() -> Json<()> {
    Json(())
}

#[get("/")]
fn index() -> &'static str {
    "Main page!"
}

#[get("/set_cookie")]
fn set_cookie(jar: &CookieJar<'_>) -> &'static str {
    jar.add(Cookie::new("user", "Alex"));
    "Кука установлена!"
}

#[get("/get_cookie")]
fn get_cookie(jar: &CookieJar<'_>) -> String {
    match jar.get("user") {
        Some(cookie) => format!("Кука: {}", cookie.value()),
        None => "Кука не найдена".to_string(),
    }
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _rocket = rocket::build()
        .mount("/", routes![index, set_cookie, get_cookie, create_user, get_user])
        .mount("/static", FileServer::from("static"))
        .launch()
        .await?;
    Ok(())
}