use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct User {
    pub name: String,
    pub email: String,
    pub password: String,
    pub phone: String,
}

#[derive(Debug, Deserialize)]
pub struct RecordUser {
    #[allow(dead_code)]
    id: RecordId,
    name: String,
    email: String,
    password: String,
    phone: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Params {
    age: u8,
    height: u16,
    weight: u128,
    user_id: RecordId,
    target: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RecordParams {
    #[allow(dead_code)]
    id: RecordId,
    date: String,
    title: String,
    content: String,
    user_id: RecordId,
    addition: String,
}

impl User {
    pub fn create(name: String, email: String, password: String, phone: String) -> User {
        User {
            name,
            email,
            password,
            phone,
        }
    }
}

impl Params {
    pub fn add_params(
        age: u8,
        height: u16,
        weight: u128,
        user_id: RecordId,
        target: String,
    ) -> Params {
        Params {
            age,
            height,
            weight,
            user_id,
            target,
        }
    }
}

impl RecordUser {
    pub fn get_uid(&self) -> &RecordId {
        &self.id
    }
}