use regex::Regex;
use crate::entity::AuthMethod;

pub fn validate_user(phone: &str, email: &str, auth: &AuthMethod, nickname: &str) -> Result<(), String> {
    let phone_regex = Regex::new(r"^\+?[0-9]{10,15}$").unwrap();
    if !phone_regex.is_match(phone) {
        return Err("Неверный формат телефона. Должен быть в формате +77777777777".to_string());
    }

    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if !email_regex.is_match(email) {
        return Err("Неверный формат email".to_string());
    }

    if nickname.is_empty() {
        return Err("Никнейм не может быть пустым".to_string());
    }
    if nickname.len() < 3 || nickname.len() > 30 {
        return Err("Никнейм должен быть от 3 до 30 символов".to_string());
    }
    let nickname_regex = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
    if !nickname_regex.is_match(nickname) {
        return Err("Никнейм может содержать только буквы, цифры и подчеркивания".to_string());
    }

    match auth {
        AuthMethod::Password { password } => {
            if password.len() < 8 {
                return Err("Пароль должен быть не короче 8 символов".to_string());
            }
            if !password.chars().any(|c| c.is_digit(10)) || !password.chars().any(|c| c.is_alphabetic()) {
                return Err("Пароль должен содержать буквы и цифры".to_string());
            }
        }
        AuthMethod::Yandex { provider_user_id } => {
            if provider_user_id.is_empty() {
                return Err("ID пользователя Yandex не может быть пустым".to_string());
            }
        }
    }

    Ok(())
}