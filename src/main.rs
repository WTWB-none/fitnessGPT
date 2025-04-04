use rocket::{get, post, routes};
use rocket::fs::FileServer;
use rocket::http::{Cookie, CookieJar, Method};
use rocket::serde::json::Json;
use rocket_cors::{AllowedOrigins, CorsOptions};

use surrealdb::Surreal;
use surrealdb::engine::local::{RocksDb, Db};

use uuid::Uuid;

use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{SaltString, rand_core::OsRng};


mod entity;
mod validation;

async fn init_db() -> surrealdb::Result<Surreal<Db>> {
    let db = Surreal::new::<RocksDb>("data/db").await?;
    db.use_ns("data").use_db("data").await?;
    Ok(db)
}

fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    hash
}

#[post("/auth/user", data = "<user>")]
async fn create_user(user: Json<entity::User>) -> Json<String> {
    let user_data = user.into_inner();
    println!("Данные: {:?}", user_data);

    if let Err(e) = validation::validate_user(&user_data.name, &user_data.phone, &user_data.email, &user_data.auth) {
        return Json(format!("Ошибка валидации: {}", e));
    }

    let db = init_db().await.unwrap();
    let user_id = format!("user:{}", Uuid::new_v4());

    let query = format!(
        "CREATE user SET id = '{}', name = '{}', email = '{}', phone = '{}'",
        user_id, user_data.name, user_data.email, user_data.phone
    );
    db.query(&query).await.unwrap();

    if let Err(e) = user_data.auth.save(&user_id, &db).await {
        return Json(format!("Ошибка сохранения авторизации: {}", e));
    }

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

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true);

    let _rocket = rocket::build()
        .mount("/", routes![index, set_cookie, get_cookie, create_user, get_user])
        .mount("/static", FileServer::from("static"))
        .attach(cors.to_cors().unwrap())
        .launch()
        .await?;
    Ok(())

}