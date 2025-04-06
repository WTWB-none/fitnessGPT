use lettre::{SmtpTransport, Transport, Message};
use lettre::message::SinglePart;
use lettre::transport::smtp::authentication::Credentials;
use crate::config::config::AppConfig;

pub fn send_email(config: &AppConfig, recipient_email: &str, subject: &str, body: &str) -> Result<(), Box<dyn std::error::Error>> {
    let email = Message::builder()
        .from(config.mail_from.parse()?)
        .to(recipient_email.parse()?)
        .subject(subject)
        .singlepart(SinglePart::plain(body.to_string()))?;

    let credentials = Credentials::new(config.smtp_user.clone(), config.smtp_password.clone());
    let mailer = SmtpTransport::relay(&config.smtp_server)?
        .credentials(credentials)
        .build();

    mailer.send(&email)?;
    Ok(())
}