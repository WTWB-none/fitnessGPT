use serde::{Deserialize, Serialize};

/// Перечисление методов аутентификации
#[derive(Debug, Serialize, Deserialize)]
pub enum AuthMethod {
    Password { password: String },
    Yandex { provider_user_id: String },
}

/// Пользователь для регистрации
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub email: String,
    pub phone: String,
    pub auth: AuthMethod,
    pub nickname: String,
}

/// Пользователь с профилем
#[derive(Debug, Serialize, Deserialize)]
pub struct UserWithProfile {
    pub id: String,
    pub email: String,
    pub phone: String,
    pub nickname: String,
    pub age: Option<i32>,
    pub height: Option<f64>,
    pub weight: Option<f64>,
    pub goal: Option<String>,
}

/// Данные для входа
#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub login: String,
    pub ident: String,
    pub password: String,
}

/// Параметры профиля
#[derive(Serialize, Deserialize)]
pub struct ProfileParams {
    pub user_id: String,
    pub age: u8,
    pub height: f32,
    pub weight: f32,
    pub goal: String,
}