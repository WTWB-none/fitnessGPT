use serde::Serialize;
use rocket::serde::json::Json;

/// Универсальная структура ответа API
#[derive(Serialize)]
pub struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Json<Self> {
        Json(ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        })
    }

    pub fn error(error: String) -> Json<Self> {
        Json(ApiResponse {
            success: false,
            data: None,
            error: Some(error),
        })
    }
}

/// Ответ на создание сущности
#[derive(Serialize)]
pub struct PostResponse {
    pub user_id: String,
    pub message: String,
}

/// Ответ на успешный вход
#[derive(Serialize)]
pub struct LoginResponse {
    pub phone: String,
    pub email: String,
    pub id: String,
    pub nickname: String,
}