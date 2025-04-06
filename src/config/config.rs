use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Переменная окружения {0} не найдена")]
    MissingEnvVar(String),
    #[error("Ошибка парсинга: {0}")]
    ParseError(#[from] std::env::VarError),
}

/// Конфигурация приложения, включающая настройки базы данных и SMTP
#[derive(Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub smtp_server: String,
    pub smtp_user: String,
    pub smtp_password: String,
    pub mail_from: String,
}

impl AppConfig {
    /// Загружает конфигурацию из переменных окружения
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(AppConfig {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL".to_string()))?,
            smtp_server: env::var("SMTP_SERVER")
                .map_err(|_| ConfigError::MissingEnvVar("SMTP_SERVER".to_string()))?,
            smtp_user: env::var("SMTP_USER")
                .map_err(|_| ConfigError::MissingEnvVar("SMTP_USER".to_string()))?,
            smtp_password: env::var("SMTP_PASSWORD")
                .map_err(|_| ConfigError::MissingEnvVar("SMTP_PASSWORD".to_string()))?,
            mail_from: env::var("MAIL_FROM")
                .map_err(|_| ConfigError::MissingEnvVar("MAIL_FROM".to_string()))?,
        })
    }
}