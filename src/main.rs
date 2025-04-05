use rocket::{get, post, routes};
use rocket::fs::FileServer;
use rocket::http::{CookieJar, Method};
use rocket::serde::json::Json;
use rocket::State;
use rocket_cors::{AllowedOrigins, CorsOptions};
use sqlx::PgPool;
use uuid::Uuid;
use argon2::{Argon2, PasswordHasher, PasswordHash, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use lettre::{SmtpTransport, Transport, Message};
use lettre::message::SinglePart;
use lettre::transport::smtp::authentication::Credentials;
use dotenv::dotenv;
use std::env;

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
struct PostResponse {
    user_id: String,
    message: String,
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

fn send_email(recipient_email: &str, subject: &str, body: &str) -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let smtp_server = env::var("SMTP_SERVER")?;
    let smtp_user = env::var("SMTP_USER")?;
    let smtp_password = env::var("SMTP_PASSWORD")?;
    let mail_from = env::var("MAIL_FROM")?;

    let email = Message::builder()
        .from(mail_from.parse()?)
        .to(recipient_email.parse()?)
        .subject(subject)
        .singlepart(SinglePart::plain(body.to_string()))?;

    let credentials = Credentials::new(smtp_user, smtp_password);
    let mailer = SmtpTransport::relay(&smtp_server)?
        .credentials(credentials)
        .build();

    mailer.send(&email)?;

    Ok(())
}

#[post("/register/user", data = "<user>")]
async fn create_user(user: Json<entity::User>, pool: &State<PgPool>) -> Json<ApiResponse<PostResponse>> {
    let user_data = user.into_inner();
    println!("Данные: {:?}", user_data);

    if let Err(e) = validation::validate_user(&user_data.phone, &user_data.email, &user_data.auth, &user_data.nickname) {
        return ApiResponse::error(format!("Ошибка валидации: {}", e));
    }

    let email_exists: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM Users WHERE email = $1)")
        .bind(&user_data.email)
        .fetch_one(&**pool)
        .await {
            Ok(exists) => exists,
            Err(e) => return ApiResponse::error(format!("Ошибка проверки email: {}", e)),
        };
    if email_exists {
        return ApiResponse::error("Пользователь с таким email уже существует".into());
    }

    let phone_exists: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM Users WHERE phone = $1)")
        .bind(&user_data.phone)
        .fetch_one(&**pool)
        .await {
            Ok(exists) => exists,
            Err(e) => return ApiResponse::error(format!("Ошибка проверки телефона: {}", e)),
        };
    if phone_exists {
        return ApiResponse::error("Пользователь с таким номером уже существует".into());
    }

    let nickname_exists: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM Users WHERE nickname = $1)")
        .bind(&user_data.nickname)
        .fetch_one(&**pool)
        .await {
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

    let subject = "Регистрация успешна!";
    let body = format!("Здравствуйте, {}! Ваш аккаунт был успешно зарегистрирован.", user_data.nickname);
    if let Err(e) = send_email(&user_data.email, subject, &body) {
        return ApiResponse::error(format!("Не удалось отправить письмо: {}", e));
    }

    ApiResponse::success(PostResponse {
        user_id: user_id.to_string(),
        message: format!("Пользователь создан: {}", user_data.nickname),
    })
}

#[post("/profile", data = "<profile_params>")]
async fn create_profile(
    profile_params: Json<entity::ProfileParams>,
    pool: &State<PgPool>,
) -> Json<ApiResponse<PostResponse>> {
    let params = profile_params.into_inner();
    
    let user_id = match Uuid::parse_str(&params.user_id) {
        Ok(uuid) => uuid,
        Err(_) => return ApiResponse::error("Неверный формат user_id".to_string()),
    };

    let user_exists: bool = match sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM Users WHERE user_id = $1)")
        .bind(&user_id)
        .fetch_one(&**pool)
        .await {
            Ok(exists) => exists,
            Err(e) => return ApiResponse::error(format!("Ошибка проверки пользователя: {}", e)),
        };

    if !user_exists {
        return ApiResponse::error("Пользователь не найден".to_string());
    }

    let result = sqlx::query(
        "INSERT INTO profiles (user_id_new, age, height, weight, goal)
         VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(user_id)
    .bind(params.age as i32)
    .bind(params.height as f64)
    .bind(params.weight as f64)
    .bind(&params.goal)
    .execute(&**pool)
    .await;

    match result {
        Ok(_) => ApiResponse::success(PostResponse {
            user_id: user_id.to_string(),
            message: "Профиль успешно создан".to_string(),
        }),
        Err(e) => ApiResponse::error(format!("Ошибка добавления профиля: {}", e)),
    }
}

#[post("/auth/login", data = "<login_data>")]
async fn login(login_data: Json<entity::LoginUser>, pool: &State<PgPool>, _jar: &CookieJar<'_>) -> Json<ApiResponse<LoginResponse>> {
    let data = login_data.into_inner();
    let ident_field = match data.ident.as_str() {
        "email" => "email",
        "phone" => "phone",
        _ => return ApiResponse::error("Неверный идентификатор".to_string()),
    };

    let query = format!(
        "SELECT user_id, email, phone, password, nickname, yandex_id FROM Users WHERE {} = $1",
        ident_field
    );
    let row: Option<(Uuid, String, String, Option<String>, String, Option<String>)> = match sqlx::query_as(&query)
        .bind(&data.login)
        .fetch_optional(&**pool)
        .await {
            Ok(row) => row,
            Err(e) => return ApiResponse::error(format!("Ошибка поиска пользователя: {}", e)),
        };

    match row {
        Some((user_id, email, phone, password, nickname, yandex_id)) => {
            if yandex_id.is_some() {
                return ApiResponse::error("Пользователь зарегистрирован через Yandex. Войдите через Yandex.".to_string());
            }
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
        None => ApiResponse::error("Пользователь не найден".to_string()),
    }
}

#[get("/get_user_data/<id>")]
async fn get_user(id: String, pool: &State<PgPool>) -> Json<ApiResponse<entity::UserWithProfile>> {
    let user_id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => return ApiResponse::error("Неверный формат ID".to_string()),
    };

    let row: Option<(Uuid, String, String, String, Option<i32>, Option<f64>, Option<f64>, Option<String>)> = 
        match sqlx::query_as(
            "SELECT 
                u.user_id, 
                u.email, 
                u.phone, 
                u.nickname, 
                p.age, 
                p.height, 
                p.weight, 
                p.goal 
            FROM Users u 
            LEFT JOIN profiles p ON u.user_id = p.user_id_new 
            WHERE u.user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&**pool)
        .await {
            Ok(row) => row,
            Err(e) => return ApiResponse::error(format!("Ошибка получения данных пользователя: {}", e)),
        };

    match row {
        Some((user_id, email, phone, nickname, age, height, weight, goal)) => {
            let user = entity::UserWithProfile {
                id: user_id.to_string(),
                email,
                phone,
                nickname,
                age,
                height,
                weight,
                goal,
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

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
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
        .mount("/", routes![index, create_user, get_user, login, create_profile])
        .mount("/static", FileServer::from("static"))
        .attach(cors.to_cors().unwrap())
        .launch()
        .await?;
    Ok(())
}