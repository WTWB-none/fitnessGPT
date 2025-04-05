use core::hash;
use std::collections::HashMap;
use std::vec;

use rocket::http::uri::Query;
use rocket::{get, post, routes};
use rocket::fs::FileServer;
use rocket::http::{Cookie, CookieJar, Method};
use rocket::serde::json::Json;
use rocket_cors::{AllowedOrigins, CorsOptions};

use surrealdb::{RecordId, Surreal, Value};
use surrealdb::engine::local::{RocksDb, Db};

use sha2::{Sha256, Digest};


use uuid::Uuid;

use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{PasswordHash, PasswordVerifier};


mod entity;
mod validation;fn verify_password(password: &str, hashed_password: &str) -> Result<(), argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hashed_password)?; // Парсим сохраненный хеш
    let argon2 = Argon2::default();
    argon2.verify_password(password.as_bytes(), &parsed_hash) // Проверяем
}

async fn init_db() -> surrealdb::Result<Surreal<Db>> {
    let db = Surreal::new::<RocksDb>("data/db").await?;
    db.use_ns("data").use_db("data").await?;
    Ok(db)
}


fn hash_password_sha256(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result) // Хеш в hex-формате
}
#[post("/register/user", data = "<user>")]
async fn create_user(user: Json<entity::User>) -> Json<String> {
    let user_data = user.into_inner();
    println!("Данные: {:?}", user_data);

    if let Err(e) = validation::validate_user(&user_data.name, &user_data.phone, &user_data.email, &user_data.auth) {
        return Json(format!("Ошибка валидации: {}", e));
    }

    let db = init_db().await.unwrap();

    let check_email_query = "SELECT email FROM user WHERE email = $email";
    let mut response = match db.query(check_email_query)
        .bind(("email", user_data.email.clone()))
        .await
    {
        Ok(res) => res,
        Err(e) => return Json(format!("Ошибка запроса почты: {}", e)),
    };

    let emails: Vec<String> = match response.take("email") {
        Ok(emails) => emails,
        Err(_) => vec![],
    };

    if !emails.is_empty() {
        return Json("Пользователь с таким email уже существует".into());
    }

    let check_phone_query = "SELECT phone FROM user WHERE phone = $phone";
    let mut response = match db.query(check_phone_query)
        .bind(("phone", user_data.phone.clone()))
        .await
    {
        Ok(res) => res,
        Err(e) => return Json(format!("Ошибка запроса телефона: {}", e)),
    };

    let phones: Vec<String> = match response.take("phone") {
        Ok(phones) => phones,
        Err(_) => vec![],
    };

    if !phones.is_empty() {
        return Json("Пользователь с таким номером уже существует".into());
    }

    let user_id = format!("user:({})", Uuid::new_v4());
    println!("{}", user_id);    

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

#[post("/auth/login", data = "<login_data>")]
async fn  login(login_data: Json<entity::LoginUser>, jar: &CookieJar<'_>) -> Json<String> {

    let data = login_data.into_inner();
    let b = data.ident == "phone";

    let query = format!("SELECT * FROM user WHERE {} = $login", data.ident);

    let db = init_db().await.unwrap();

    let mut response = match db.query(query)
        .bind(("login", data.login.clone()))
        .await
    {
        Ok(res) => res,
        Err(e) => return Json(format!("Ошибка запроса почты: {}", e)),
    };

    let res: Vec<String> = match response.take("phone") {
        Ok(phones) => phones,
        Err(_) => vec![],
    };

    let query = format!("SELECT VALUE id FROM user WHERE {} = '{}'", data.ident, res[0]);
    
    let mut id = match db.query(query)
        .bind(("login", data.login.clone()))
        .await
    {
        Ok(res) => res,
        Err(e) => return Json(format!("Ошибка запроса почты: {}", e)),
    };
       
    let id = id.take::<Option<RecordId>>(0).unwrap().unwrap();
    let id = id.key().to_string();

    let vec: Vec<char> = id.chars().collect::<Vec<char>>();
    let r = vec.clone().into_iter().skip(1).take(vec.len() - 2).collect::<String>();

    let query = format!("SELECT hashed_password FROM `{}`", format!("auth_password_{}", r));

    let mut response = match db.query(query)
        .await
    {
        Ok(res) => {println!("{:?}", res);res},
        Err(e) => return Json(format!("Ошибка запроса почты: {}", e)),
    };

    let hash_pass: Vec<String> = match response.take("hashed_password") {
        Ok(pass) => pass,
        Err(_) => vec![],
    };


    
    println!("{:?} {}",  hash_pass[0], hash_password_sha256(&data.password));
    if hash_pass[0] == hash_password_sha256(&data.password) {
        return Json(format!("name:Тест, phone:тест, email:тест, id:тест"))
    }

    Json("Пользователь не зарегистрирован!".to_string())

}

#[get("/get_user_data/<id>")]
async fn get_user(id: String) -> Json<Option<entity::RecordUser>> {
    let db = init_db().await.unwrap();
    let user_id = RecordId::from(("user", id.as_str()));
    
    match db.select(user_id).await {
        Ok(user) => Json(user),
        Err(e) => {
            eprintln!("Ошибка получения пользователя: {}", e);
            Json(None)
        }
    }
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
        .mount("/", routes![index, set_cookie, get_cookie, create_user, get_user, login])
        .mount("/static", FileServer::from("static"))
        .attach(cors.to_cors().unwrap())
        .launch()
        .await?;
    Ok(())

}