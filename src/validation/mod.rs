use crate::entity::AuthMethod;
use regex::Regex;
use email_address::EmailAddress;
use lazy_static::lazy_static;

pub fn validate_user(name: &str, phone: &str, email: &str, auth: &AuthMethod) -> Result<(), String> {
    validate_name(name)?;
    validate_phone(phone)?;
    validate_email(email)?;

    if let AuthMethod::Password { password } = auth {
        validate_password(password)?;
    }

    Ok(())
}

fn validate_name(username: &str) -> Result<(), &'static str> {
    lazy_static! {
        static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_-]{3,20}$").unwrap();
    }
    if USERNAME_REGEX.is_match(username) {
        Ok(())
    } else {
        Err("Имя пользователя должно содержать от 3 до 20 символов, включая буквы русского или английского алфавита, цифры, подчеркивания или дефисы.")
    }
}

fn validate_phone(phone: &str) -> Result<(), &'static str> {
    lazy_static! {
        static ref PHONE_REGEX: Regex = Regex::new(r"^(?:\+7|8)\d{10}$").unwrap();
    }
    if PHONE_REGEX.is_match(phone) {
        Ok(())
    } else {
        Err("Номер телефона должен начинаться с '+7' или '8' и содержать 11 цифр.")
    }
}

fn validate_email(email: &str) -> Result<(), &'static str> {
    if EmailAddress::is_valid(email) {
        Ok(())
    } else {
        Err("Некорректный формат адреса электронной почты.")
    }
}

fn validate_password(password: &str) -> Result<(), &'static str> {
    if password.len() < 8 {
        return Err("Пароль должен содержать минимум 8 символов.");
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err("Пароль должен содержать хотя бы одну заглавную букву.");
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err("Пароль должен содержать хотя бы одну строчную букву.");
    }
    if !password.chars().any(|c| c.is_digit(10)) {
        return Err("Пароль должен содержать хотя бы одну цифру.");
    }
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err("Пароль должен содержать хотя бы один специальный символ.");
    }
    Ok(())
}
