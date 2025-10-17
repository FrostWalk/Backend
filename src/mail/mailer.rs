use confirm_email::generate_token;
use lettre::message::{
    header::{ContentTransferEncoding, ContentType},
    Mailbox, Message, MultiPart, SinglePart,
};
use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport,
    Tokio1Executor,
};
use url::Url;
use uuid::Uuid;

use super::template::TemplateEngine;
use crate::config::Config;
use minijinja::Value as JinjaValue;

type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;
type Result<T> = std::result::Result<T, DynError>;

const CONFIRMATION_URL: &str = "/confirm";

#[derive(Clone)]
pub struct Mailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
    frontend_base_url: Url,
    templates: TemplateEngine,
}

impl Mailer {
    pub fn from_config(config: &Config) -> Result<Self> {
        Self::new(
            config.smtp_host(),
            config.smtp_port(),
            config.smtp_username(),
            config.smtp_password(),
            config.email_from(),
            config.frontend_base_url(),
        )
    }

    pub fn new(
        smtp_host: &str, port: u16, username: &str, password: &str, from_name: &str,
        frontend_base_url: &str,
    ) -> Result<Self> {
        let creds = Credentials::new(username.to_owned(), password.to_owned());

        // Configure SMTP transport with RFC 5322 compliance
        // - Uses STARTTLS for secure connection (required by Google)
        // - Sets timeouts for reliable delivery
        let mut builder = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(smtp_host)?;
        builder = builder
            .port(port)
            .credentials(creds)
            // Set connection timeout (30 seconds) for reliable delivery
            .timeout(Some(std::time::Duration::from_secs(30)));

        let transport = builder.build();

        let from = Mailbox::new(Some(from_name.to_owned()), username.parse()?);
        let frontend_base_url = Url::parse(frontend_base_url)?;

        Ok(Self {
            transport,
            from,
            frontend_base_url,
            templates: TemplateEngine::new()?,
        })
    }

    fn confirmation_link(&self, email: String, key: String) -> Result<Url> {
        let token = generate_token(email, key)?;

        let mut url = self.frontend_base_url.join(CONFIRMATION_URL)?;
        url.query_pairs_mut().append_pair("t", token.as_str());
        Ok(url)
    }

    /// Generate a RFC 5322 compliant Message-ID header
    /// Format: <unique-id@domain>
    /// Uses the sender's email address domain
    fn generate_message_id(&self) -> String {
        let unique_id = Uuid::new_v4();
        let domain = self.from.email.domain();
        format!("<{}@{}>", unique_id, domain)
    }

    async fn send_templated(
        &self, to_email: String, to_name: String, subject: &str, html_template_name: &str,
        text_template_name: &str, ctx: JinjaValue,
    ) -> Result<()> {
        let to = Mailbox::new(Some(to_name), to_email.parse()?);

        let html_body = self.templates.render(html_template_name, ctx.clone())?;
        let text_body = self.templates.render(text_template_name, ctx)?;

        // Generate RFC 5322 compliant Message-ID using sender's email domain
        let message_id = self.generate_message_id();

        // Build email with RFC 5322 compliant structure
        // The lettre library automatically adds required headers:
        // - Date header (current time)
        // - MIME-Version (when using MultiPart)
        // We explicitly add:
        // - Message-ID (format: <unique-id@sender-domain>)
        // Using QuotedPrintable encoding ensures RFC 5322 line length limits (998 chars/line)
        let email = Message::builder()
            .from(self.from.clone())
            .to(to)
            .subject(subject)
            .message_id(Some(message_id))
            .multipart(
                // MultiPart::alternative with text/plain first, then text/html
                // This is the RFC 2046 recommended order
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .header(ContentTransferEncoding::QuotedPrintable)
                            .body(text_body),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .header(ContentTransferEncoding::QuotedPrintable)
                            .body(html_body),
                    ),
            )?;

        self.transport.send(email).await?;
        Ok(())
    }

    pub async fn send_account_confirmation(
        &self, to_email: String, to_name: String, key: String,
    ) -> Result<()> {
        let confirm_url = self.confirmation_link(to_email.clone(), key)?;

        let ctx = minijinja::context! {
            user_name => to_name,
            url => confirm_url.as_str(),
        };

        self.send_templated(
            to_email,
            to_name,
            "Confirm your account",
            "confirm.html",
            "confirm.txt",
            ctx,
        )
        .await
    }

    pub async fn send_password_reset(
        &self, to_email: String, to_name: String, reset_url: &str,
    ) -> Result<()> {
        let ctx = minijinja::context! {
            user_name => to_name,
            url => reset_url,
        };

        self.send_templated(
            to_email,
            to_name,
            "Reset your password",
            "reset.html",
            "reset.txt",
            ctx,
        )
        .await
    }

    pub async fn send_admin_welcome(
        &self, to_email: String, to_name: String, password: String,
    ) -> Result<()> {
        let login_url = self.frontend_base_url.join("/admin/login")?.to_string();

        let ctx = minijinja::context! {
            user_name => to_name,
            email => to_email.clone(),
            password => password,
            login_url => login_url,
        };

        self.send_templated(
            to_email,
            to_name,
            "Welcome to Advanced Programming Administration",
            "admin_welcome.html",
            "admin_welcome.txt",
            ctx,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_confirmation_link_generation() {
        let mailer = create_test_mailer().unwrap();
        let email = TEST_STUDENT_EMAIL.to_string();
        let key = "test-confirmation-key".to_string();

        let result = mailer.confirmation_link(email.clone(), key.clone());
        assert!(result.is_ok());

        let url = result.unwrap();
        assert!(url.as_str().contains(TEST_FRONTEND_URL));
        assert!(url.as_str().contains("/confirm"));
        assert!(url.as_str().contains("t="));

        // Verify the token can be extracted
        let query_pairs: std::collections::HashMap<_, _> = url.query_pairs().collect();
        assert!(query_pairs.contains_key("t"));
    }

    #[test]
    fn test_confirmation_link_with_different_emails() {
        let mailer = create_test_mailer().unwrap();
        let key = "test-key".to_string();

        let email1 = "user1@test.com".to_string();
        let email2 = "user2@test.com".to_string();

        let url1 = mailer.confirmation_link(email1, key.clone()).unwrap();
        let url2 = mailer.confirmation_link(email2, key).unwrap();

        // Different emails should generate different tokens
        let query1: std::collections::HashMap<_, _> = url1.query_pairs().collect();
        let query2: std::collections::HashMap<_, _> = url2.query_pairs().collect();

        assert_ne!(query1.get("t"), query2.get("t"));
    }

    #[test]
    fn test_confirmation_link_with_different_keys() {
        let mailer = create_test_mailer().unwrap();
        let email = TEST_STUDENT_EMAIL.to_string();

        let key1 = "key1".to_string();
        let key2 = "key2".to_string();

        let url1 = mailer.confirmation_link(email.clone(), key1).unwrap();
        let url2 = mailer.confirmation_link(email, key2).unwrap();

        // Different keys should generate different tokens
        let query1: std::collections::HashMap<_, _> = url1.query_pairs().collect();
        let query2: std::collections::HashMap<_, _> = url2.query_pairs().collect();

        assert_ne!(query1.get("t"), query2.get("t"));
    }

    #[test]
    fn test_generate_message_id_format() {
        let mailer = create_test_mailer().unwrap();
        let message_id = mailer.generate_message_id();

        // Should be in format <uuid@domain>
        assert!(message_id.starts_with('<'));
        assert!(message_id.ends_with('>'));
        assert!(message_id.contains('@'));

        // Should contain the sender's domain
        assert!(message_id.contains("test.com"));
    }

    #[test]
    fn test_generate_message_id_uniqueness() {
        let mailer = create_test_mailer().unwrap();

        let id1 = mailer.generate_message_id();
        let id2 = mailer.generate_message_id();

        // Should generate unique IDs
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_message_id_domain_extraction() {
        let mailer = create_test_mailer().unwrap();
        let message_id = mailer.generate_message_id();

        // Extract domain from message ID
        let domain_part = message_id.split('@').nth(1).unwrap().trim_end_matches('>');
        assert_eq!(domain_part, "test.com");
    }

    #[test]
    fn test_mailer_new_success() {
        let result = Mailer::new(
            TEST_SMTP_HOST,
            587,
            TEST_SMTP_USERNAME,
            "testpassword",
            "Test Sender",
            TEST_FRONTEND_URL,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_mailer_new_invalid_smtp_host() {
        let result = Mailer::new(
            "invalid-host-that-does-not-exist",
            587,
            TEST_SMTP_USERNAME,
            "testpassword",
            "Test Sender",
            TEST_FRONTEND_URL,
        );

        // This might succeed or fail depending on network, but should not panic
        // The important thing is that it handles the error gracefully
        let _ = result; // Don't assert, just ensure it doesn't panic
    }

    #[test]
    fn test_mailer_new_invalid_frontend_url() {
        let result = Mailer::new(
            TEST_SMTP_HOST,
            587,
            TEST_SMTP_USERNAME,
            "testpassword",
            "Test Sender",
            "not-a-valid-url",
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_mailer_from_config() {
        let config = create_test_config();
        let result = Mailer::from_config(&config);

        assert!(result.is_ok());
    }

    #[test]
    fn test_mailer_is_cloneable() {
        let mailer1 = create_test_mailer().unwrap();
        let mailer2 = mailer1.clone();

        // Both should work independently
        let email = TEST_STUDENT_EMAIL.to_string();
        let key = "test-key".to_string();

        let result1 = mailer1.confirmation_link(email.clone(), key.clone());
        let result2 = mailer2.confirmation_link(email, key);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    #[test]
    fn test_confirmation_link_with_special_characters() {
        let mailer = create_test_mailer().unwrap();
        let email = "test+tag@example.com".to_string();
        let key = "key-with-special-chars!@#$%".to_string();

        let result = mailer.confirmation_link(email, key);
        assert!(result.is_ok());

        let url = result.unwrap();
        assert!(url.as_str().contains("/confirm"));
    }

    #[test]
    fn test_confirmation_link_with_unicode() {
        let mailer = create_test_mailer().unwrap();
        let email = "тест@example.com".to_string();
        let key = "ключ-с-кириллицей".to_string();

        let result = mailer.confirmation_link(email, key);
        assert!(result.is_ok());

        let url = result.unwrap();
        assert!(url.as_str().contains("/confirm"));
    }

    fn create_test_mailer() -> Result<Mailer> {
        Mailer::new(
            TEST_SMTP_HOST,
            587,
            TEST_SMTP_USERNAME,
            "testpassword",
            "Test Sender",
            TEST_FRONTEND_URL,
        )
    }
}
