use rocket::{post, get};
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::State;
use sqlx::PgPool;
use uuid::Uuid;
use log::{error, info};

use crate::entity::models::{User, LoginUser, ProfileParams, UserWithProfile, AuthMethod};
use crate::validation::validators::validate_user;
use crate::services::auth::{hash_password, verify_password};
use crate::services::email::send_email;
use crate::api::responses::{ApiResponse, PostResponse, LoginResponse};
use crate::config::config::AppConfig;

#[post("/register/user", data = "<user>")]
pub async fn create_user(
    user: Json<User>,
    pool: &State<PgPool>,
    config: &State<AppConfig>,
) -> Json<ApiResponse<PostResponse>> {
    let user_data = user.into_inner();
    info!("Регистрация пользователя: {}", user_data.nickname);

    if let Err(e) = validate_user(&user_data.phone, &user_data.email, &user_data.auth, &user_data.nickname) {
        error!("Ошибка валидации для {}: {}", user_data.nickname, e);
        return ApiResponse::error(format!("Ошибка валидации: {}", e));
    }

    match &user_data.auth {
        AuthMethod::Yandex { provider_user_id } => {
            let existing_user: Option<(Uuid, String)> = sqlx::query_as(
                "SELECT user_id, nickname FROM Users WHERE yandex_id = $1"
            )
            .bind(provider_user_id)
            .fetch_optional(&**pool)
            .await
            .unwrap_or(None);

            if let Some((user_id, nickname)) = existing_user {
                info!("Пользователь с yandex_id {} уже существует: {}", provider_user_id, nickname);
                return ApiResponse::success(PostResponse {
                    user_id: user_id.to_string(),
                    message: format!("Пользователь уже зарегистрирован: {}", nickname),
                });
            }
        }
        AuthMethod::Password { .. } => {
            if check_exists(pool, "email", &user_data.email).await {
                return ApiResponse::error("Пользователь с таким email уже существует".into());
            }
            if check_exists(pool, "phone", &user_data.phone).await {
                return ApiResponse::error("Пользователь с таким номером уже существует".into());
            }
            if check_exists(pool, "nickname", &user_data.nickname).await {
                return ApiResponse::error("Пользователь с таким никнеймом уже существует".into());
            }
        }
    }

    let user_id = Uuid::new_v4();
    let (password, yandex_id) = match &user_data.auth {
        AuthMethod::Password { password } => {
            (Some(hash_password(password).unwrap_or_default()), None)
        }
        AuthMethod::Yandex { provider_user_id } => (None, Some(provider_user_id.clone())),
    };

    let result = sqlx::query(
        "INSERT INTO Users (user_id, email, phone, password, yandex_id, nickname)
         VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(user_id)
    .bind(&user_data.email)
    .bind(&user_data.phone)
    .bind(&password)
    .bind(&yandex_id)
    .bind(&user_data.nickname)
    .execute(&**pool)
    .await;

    if let Err(e) = result {
        error!("Ошибка вставки пользователя {}: {}", user_data.nickname, e);
        return ApiResponse::error(format!("Ошибка вставки пользователя: {}", e));
    }

    let subject = "Регистрация успешна!";
    let body = format!("Здравствуйте, {}! Ваш аккаунт успешно зарегистрирован.", user_data.nickname);
    if let Err(e) = send_email(config, &user_data.email, subject, &body) {
        error!("Не удалось отправить письмо для {}: {}", user_data.email, e);
        return ApiResponse::error(format!("Не удалось отправить письмо: {}", e));
    }

    info!("Пользователь {} успешно зарегистрирован", user_data.nickname);
    ApiResponse::success(PostResponse {
        user_id: user_id.to_string(),
        message: format!("Пользователь создан: {}", user_data.nickname),
    })
}

#[post("/profile", data = "<profile_params>")]
pub async fn create_profile(
    profile_params: Json<ProfileParams>,
    pool: &State<PgPool>,
) -> Json<ApiResponse<PostResponse>> {
    let params = profile_params.into_inner();
    let user_id = match Uuid::parse_str(&params.user_id) { // Исправлено: params вместо ¶ms
        Ok(uuid) => uuid,
        Err(_) => return ApiResponse::error("Неверный формат user_id".to_string()),
    };

    if !check_exists(pool, "user_id", &user_id.to_string()).await {
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
    .bind(&params.goal) // Исправлено: params вместо ¶ms
    .execute(&**pool)
    .await;

    match result {
        Ok(_) => {
            info!("Профиль для пользователя {} создан", user_id);
            ApiResponse::success(PostResponse {
                user_id: user_id.to_string(),
                message: "Профиль успешно создан".to_string(),
            })
        }
        Err(e) => {
            error!("Ошибка создания профиля для {}: {}", user_id, e);
            ApiResponse::error(format!("Ошибка добавления профиля: {}", e))
        }
    }
}

#[post("/auth/login", data = "<login_data>")]
pub async fn login(
    login_data: Json<LoginUser>,
    pool: &State<PgPool>,
    _jar: &CookieJar<'_>,
) -> Json<ApiResponse<LoginResponse>> {
    let data = login_data.into_inner();
    info!("Попытка входа для {}", data.login);

    let result = match data.ident.as_str() {
        "email" => sqlx::query_as::<_, (Uuid, String, String, Option<String>, String, Option<String>)>(
            "SELECT user_id, email, phone, password, nickname, yandex_id FROM Users WHERE email = $1"
        )
        .bind(&data.login)
        .fetch_optional(&**pool)
        .await,
        "phone" => sqlx::query_as::<_, (Uuid, String, String, Option<String>, String, Option<String>)>(
            "SELECT user_id, email, phone, password, nickname, yandex_id FROM Users WHERE phone = $1"
        )
        .bind(&data.login)
        .fetch_optional(&**pool)
        .await,
        _ => return ApiResponse::error("Неверный идентификатор".to_string()),
    };

    match result {
        Ok(Some((user_id, email, phone, password, nickname, yandex_id))) => {
            if yandex_id.is_some() {
                return ApiResponse::error("Вход через Yandex не поддерживается через этот маршрут".to_string());
            }
            match password {
                Some(hashed) => {
                    if verify_password(&data.password, &hashed) {
                        info!("Успешный вход для {}", nickname);
                        ApiResponse::success(LoginResponse {
                            phone,
                            email,
                            id: user_id.to_string(),
                            nickname,
                        })
                    } else {
                        error!("Неверный пароль для {}", data.login);
                        ApiResponse::error("Неверный пароль".to_string())
                    }
                }
                None => ApiResponse::error("Вход через Yandex не поддерживается через этот маршрут".to_string()),
            }
        }
        Ok(None) => ApiResponse::error("Пользователь не найден".to_string()),
        Err(e) => {
            error!("Ошибка поиска пользователя {}: {}", data.login, e);
            ApiResponse::error(format!("Ошибка поиска пользователя: {}", e))
        }
    }
}

#[get("/get_user_data/<id>")]
pub async fn get_user(id: String, pool: &State<PgPool>) -> Json<ApiResponse<UserWithProfile>> {
    let user_id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => return ApiResponse::error("Неверный формат ID".to_string()),
    };

    let result = sqlx::query_as::<_, (Uuid, String, String, String, Option<i32>, Option<f64>, Option<f64>, Option<String>)>(
        "SELECT u.user_id, u.email, u.phone, u.nickname, p.age, p.height, p.weight, p.goal 
         FROM Users u 
         LEFT JOIN profiles p ON u.user_id = p.user_id_new 
         WHERE u.user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(&**pool)
    .await;

    match result {
        Ok(Some((user_id, email, phone, nickname, age, height, weight, goal))) => {
            info!("Данные пользователя {} успешно получены", nickname);
            ApiResponse::success(UserWithProfile {
                id: user_id.to_string(),
                email,
                phone,
                nickname,
                age,
                height,
                weight,
                goal,
            })
        }
        Ok(None) => ApiResponse::error("Пользователь не найден".to_string()),
        Err(e) => {
            error!("Ошибка получения данных пользователя {}: {}", user_id, e);
            ApiResponse::error(format!("Ошибка получения данных пользователя: {}", e))
        }
    }
}

async fn check_exists(pool: &State<PgPool>, field: &str, value: &str) -> bool {
    let result = match field {
        "email" => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM Users WHERE email = $1)")
            .bind(value)
            .fetch_one(&**pool)
            .await,
        "phone" => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM Users WHERE phone = $1)")
            .bind(value)
            .fetch_one(&**pool)
            .await,
        "nickname" => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM Users WHERE nickname = $1)")
            .bind(value)
            .fetch_one(&**pool)
            .await,
        "user_id" => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM Users WHERE user_id = $1)")
            .bind(value)
            .fetch_one(&**pool)
            .await,
        _ => return false,
    };
    result.unwrap_or(false)
}