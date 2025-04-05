use rocket::{get, post, routes};
use rocket::fs::FileServer;
use rocket::http::{Cookie, CookieJar, Method};
use rocket::serde::json::Json;
use rocket_cors::{AllowedOrigins, CorsOptions};
use sqlx::PgPool;
use uuid::Uuid;
use argon2::{Argon2, PasswordHasher, PasswordHash, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::OsRng};

mod entity;
mod validation;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T: serde::Serialize> ApiResponse<T> {
    fn success(data: T) -> Json<Self> {
        Json(ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        })
    }

    fn error(error: String) -> Json<Self> {
        Json(ApiResponse {
            success: false,
            data: None,
            error: Some(error),
        })
    }
}

#[derive(serde::Serialize)]
struct LoginResponse {
    phone: String,
    email: String,
    id: String,
    nickname: String,
}

fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();
    Ok(password_hash)
}

#[post("/register/user", data = "<user>")]
async fn create_user(user: Json<entity::User>, pool: &rocket::State<PgPool>) -> Json<ApiResponse<String>> {
    let user_data = user.into_inner();
    println!("Данные: {:?}", user_data);

    if let Err(e) = validation::validate_user(&user_data.phone, &user_data.email, &user_data.auth, &user_data.nickname) {
        return ApiResponse::error(format!("Ошибка валидации: {}", e));
    }

    let email_exists: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM Users WHERE email = $1)")
        .bind(&user_data.email)
        .fetch_one(&**pool)
        .await
    {
        Ok(exists) => exists,
        Err(e) => return ApiResponse::error(format!("Ошибка проверки email: {}", e)),
    };

    if email_exists {
        return ApiResponse::error("Пользователь с таким email уже существует".into());
    }

    let phone_exists: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM Users WHERE phone = $1)")
        .bind(&user_data.phone)
        .fetch_one(&**pool)
        .await
    {
        Ok(exists) => exists,
        Err(e) => return ApiResponse::error(format!("Ошибка проверки телефона: {}", e)),
    };

    if phone_exists {
        return ApiResponse::error("Пользователь с таким номером уже существует".into());
    }

    let nickname_exists: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM Users WHERE nickname = $1)")
        .bind(&user_data.nickname)
        .fetch_one(&**pool)
        .await
    {
        Ok(exists) => exists,
        Err(e) => return ApiResponse::error(format!("Ошибка проверки никнейма: {}", e)),
    };

    if nickname_exists {
        return ApiResponse::error("Пользователь с таким никнеймом уже существует".into());
    }

    let user_id = Uuid::new_v4();
    let (password, yandex_id) = match user_data.auth {
        entity::AuthMethod::Password { password } => {
            let hashed = match hash_password(&password) {
                Ok(h) => h,
                Err(e) => return ApiResponse::error(format!("Ошибка хеширования пароля: {}", e)),
            };
            (Some(hashed), None)
        }
        entity::AuthMethod::Yandex { provider_user_id } => (None, Some(provider_user_id)),
    };

    let result = sqlx::query(
        "INSERT INTO Users (user_id, email, phone, password, yandex_id, nickname)
         VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(user_id)
    .bind(&user_data.email)
    .bind(&user_data.phone)
    .bind(password)
    .bind(yandex_id)
    .bind(&user_data.nickname)
    .execute(&**pool)
    .await;

    if let Err(e) = result {
        return ApiResponse::error(format!("Ошибка вставки пользователя: {}", e));
    }

    ApiResponse::success(format!("Пользователь создан: {}", user_data.nickname))
}

#[post("/auth/login", data = "<login_data>")]
async fn login(login_data: Json<entity::LoginUser>, pool: &rocket::State<PgPool>, _jar: &CookieJar<'_>) -> Json<ApiResponse<LoginResponse>> {
    let data = login_data.into_inner();
    let ident_field = match data.ident.as_str() {
        "email" => "email",
        "phone" => "phone",
        _ => return ApiResponse::error("Неверный идентификатор".to_string()),
    };

    // Проверяем пользователя и его yandex_id
    let query = format!("SELECT user_id, email, phone, password, nickname, yandex_id FROM Users WHERE {} = $1", ident_field);
    let row: Option<(Uuid, String, String, Option<String>, String, Option<String>)> = match sqlx::query_as(&query)
        .bind(&data.login)
        .fetch_optional(&**pool)
        .await
    {
        Ok(row) => row,
        Err(e) => return ApiResponse::error(format!("Ошибка поиска пользователя: {}", e)),
    };

    match row {
        Some((user_id, email, phone, password, nickname, yandex_id)) => {
            // Если yandex_id не пустой, пользователь зарегистрирован через Yandex, вход только через Yandex
            if yandex_id.is_some() {
                return ApiResponse::error("Пользователь зарегистрирован через Yandex. Войдите через Yandex.".to_string());
            }

            // Если yandex_id пустой, проверяем пароль
            match password {
                Some(hashed_password) => {
                    let parsed_hash = match PasswordHash::new(&hashed_password) {
                        Ok(hash) => hash,
                        Err(_) => return ApiResponse::error("Ошибка обработки пароля".to_string()),
                    };
                    if Argon2::default().verify_password(data.password.as_bytes(), &parsed_hash).is_ok() {
                        let response = LoginResponse {
                            phone,
                            email,
                            id: user_id.to_string(),
                            nickname,
                        };
                        ApiResponse::success(response)
                    } else {
                        ApiResponse::error("Неверный пароль".to_string())
                    }
                }
                None => ApiResponse::error("Пользователь не зарегистрирован через пароль. Войдите через Yandex.".to_string()),
            }
        }
        None => ApiResponse::error("Пользователь не зарегистрирован".to_string()),
    }
}

#[get("/get_user_data/<id>")]
async fn get_user(id: String, pool: &rocket::State<PgPool>) -> Json<ApiResponse<entity::RecordUser>> {
    let user_id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => return ApiResponse::error("Неверный формат ID".to_string()),
    };

    let row: Option<(Uuid, String, String, String)> = match sqlx::query_as(
        "SELECT user_id, email, phone, nickname FROM Users WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(&**pool)
    .await
    {
        Ok(row) => row,
        Err(e) => return ApiResponse::error(format!("Ошибка получения пользователя: {}", e)),
    };

    match row {
        Some((user_id, email, phone, nickname)) => {
            let user = entity::RecordUser {
                id: user_id.to_string(),
                email,
                phone,
                nickname,
            };
            ApiResponse::success(user)
        }
        None => ApiResponse::error("Пользователь не найден".to_string()),
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
    let pool = PgPool::connect("postgres://exerted:Topparol754@localhost/fitness_assistant").await?;

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
        .manage(pool)
        .mount("/", routes![index, set_cookie, get_cookie, create_user, get_user, login])
        .mount("/static", FileServer::from("static"))
        .attach(cors.to_cors().unwrap())
        .launch()
        .await?;
    Ok(())
}