use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub email: String,
    pub phone: String,
    pub auth: AuthMethod,
    pub nickname: String,
}

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

#[derive(serde::Serialize)]
struct PostResponse {
    user_id: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AuthMethod {
    Password { password: String },
    Yandex { provider_user_id: String },
}

#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub login: String,
    pub ident: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecordUser {
    pub id: String,
    pub email: String,
    pub phone: String,
    pub nickname: String,
}

#[derive(Serialize, Deserialize)]
pub struct ProfileParams {
    pub user_id: String,
    pub age: u8,
    pub height: f32,
    pub weight: f32,
    pub goal: String,
}

impl User {
    pub fn create(email: String, phone: String, auth: AuthMethod, nickname: String) -> User {
        User {
            email,
            phone,
            auth,
            nickname,
        }
    }
}