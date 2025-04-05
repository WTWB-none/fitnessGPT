use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub email: String,
    pub phone: String,
    pub auth: AuthMethod,
    pub nickname: String,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Params {
    age: u8,
    height: u16,
    weight: u128,
    user_id: String,
    target: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RecordParams {
    id: String,
    date: String,
    title: String,
    content: String,
    user_id: String,
    addition: String,
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

impl Params {
    pub fn add_params(age: u8, height: u16, weight: u128, user_id: String, target: String) -> Params {
        Params {
            age,
            height,
            weight,
            user_id,
            target,
        }
    }
}