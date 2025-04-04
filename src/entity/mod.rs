use serde::{Deserialize, Serialize};
use surrealdb::RecordId;
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub email: String,
    pub phone: String,
    pub auth: AuthMethod,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AuthMethod {
    Password { password: String },
    Yandex { provider_user_id: String }
}

impl AuthMethod {
    pub async fn save(&self, user_id: &str, db: &Surreal<Db>) -> Result<(), String> {
        match self {
            AuthMethod::Password { password } => {
                let hashed = super::hash_password(password);
                let auth_id = format!("auth:password:{}", Uuid::new_v4());
                let query = format!(
                    "CREATE `{}` SET user = '{}', provider = 'password', hashed_password = '{}'",
                    auth_id, user_id, hashed
                );
                db.query(&query).await.map_err(|e| e.to_string())?;
            }

            AuthMethod::Yandex { provider_user_id } => {
                let auth_id = format!("auth:yandex:{}", provider_user_id);
                let query = format!(
                    "CREATE `{}` SET user = '{}', provider = 'yandex', provider_user_id = '{}'",
                    auth_id, user_id, provider_user_id
                );
                db.query(&query).await.map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
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
    pub fn create(name: String, email: String, phone: String, auth: AuthMethod) -> User {
        User {
            name,
            email,
            phone,
            auth,
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