use regex::Regex;
use crate::entity::models::AuthMethod;

const PHONE_ERROR: &str = "Неверный формат телефона. Пример: +79991234567";
const EMAIL_ERROR: &str = "Неверный формат email";
const NICKNAME_EMPTY_ERROR: &str = "Никнейм не может быть пустым";
const NICKNAME_LENGTH_ERROR: &str = "Никнейм должен быть от 3 до 30 символов";
const NICKNAME_FORMAT_ERROR: &str = "Никнейм может содержать только буквы, цифры и подчеркивания";
const PASSWORD_LENGTH_ERROR: &str = "Пароль должен быть не короче 8 символов";
const PASSWORD_FORMAT_ERROR: &str = "Пароль должен содержать буквы и цифры";
const YANDEX_ID_EMPTY_ERROR: &str = "ID пользователя Yandex не может быть пустым";

fn validate_phone(phone: &str) -> Result<(), String> {
    let phone_regex = Regex::new(r"^\+[1-9][0-9]{9,14}$").unwrap();
    if !phone_regex.is_match(phone) {
        return Err(PHONE_ERROR.to_string());
    }
    Ok(())
}

fn validate_email(email: &str) -> Result<(), String> {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if !email_regex.is_match(email) {
        return Err(EMAIL_ERROR.to_string());
    }
    Ok(())
}

fn validate_nickname(nickname: &str) -> Result<(), String> {
    if nickname.is_empty() {
        return Err(NICKNAME_EMPTY_ERROR.to_string());
    }
    if nickname.len() < 3 || nickname.len() > 30 {
        return Err(NICKNAME_LENGTH_ERROR.to_string());
    }
    let nickname_regex = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
    if !nickname_regex.is_match(nickname) {
        return Err(NICKNAME_FORMAT_ERROR.to_string());
    }
    Ok(())
}

fn validate_auth(auth: &AuthMethod) -> Result<(), String> {
    match auth {
        AuthMethod::Password { password } => {
            if password.len() < 8 {
                return Err(PASSWORD_LENGTH_ERROR.to_string());
            }
            if !password.chars().any(|c| c.is_digit(10)) || !password.chars().any(|c| c.is_alphabetic()) {
                return Err(PASSWORD_FORMAT_ERROR.to_string());
            }
        }
        AuthMethod::Yandex { provider_user_id } => {
            if provider_user_id.is_empty() {
                return Err(YANDEX_ID_EMPTY_ERROR.to_string());
            }
        }
    }
    Ok(())
}

pub fn validate_user(phone: &str, email: &str, auth: &AuthMethod, nickname: &str) -> Result<(), String> {
    validate_phone(phone)?;
    validate_email(email)?;
    validate_nickname(nickname)?;
    validate_auth(auth)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_user() {
        let auth = AuthMethod::Password { password: "Password123".to_string() };
        let result = validate_user("+79991234567", "test@example.com", &auth, "testuser");
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_phone() {
        let auth = AuthMethod::Password { password: "Password123".to_string() };
        let result = validate_user("123", "test@example.com", &auth, "testuser");
        assert!(result.is_err());
    }
}