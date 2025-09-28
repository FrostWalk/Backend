use confirm_email::generate_token;
use lettre::message::{header::ContentType, Mailbox, Message, MultiPart, SinglePart};
use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport,
    Tokio1Executor,
};
use url::Url;

use super::template::TemplateEngine;
use crate::config::Config;
use minijinja::Value as JinjaValue;

type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;
type Result<T> = std::result::Result<T, DynError>;

const CONFIRMATION_URL: &str = "/v1/students/auth/confirm";

pub struct Mailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
    base_url: Url,
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
            config.app_base_url(),
        )
    }

    pub fn new(
        smtp_host: &str, port: u16, username: &str, password: &str, from_name: &str,
        app_base_url: &str,
    ) -> Result<Self> {
        let creds = Credentials::new(username.to_owned(), password.to_owned());

        let mut builder = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(smtp_host)?;
        builder = builder.port(port);

        let transport = builder.credentials(creds).build();

        let from = Mailbox::new(Some(from_name.to_owned()), username.parse()?);
        let base_url = Url::parse(app_base_url)?;

        Ok(Self {
            transport,
            from,
            base_url,
            templates: TemplateEngine::new()?,
        })
    }

    fn confirmation_link(&self, email: String, key: String) -> Result<Url> {
        let token = generate_token(email, key)?;

        let mut url = self.base_url.join(CONFIRMATION_URL)?;
        url.query_pairs_mut().append_pair("t", token.as_str());
        Ok(url)
    }

    async fn send_templated(
        &self, to_email: String, to_name: String, subject: &str, html_template_name: &str,
        text_template_name: &str, ctx: JinjaValue,
    ) -> Result<()> {
        let to = Mailbox::new(Some(to_name), to_email.parse()?);

        let html_body = self.templates.render(html_template_name, ctx.clone())?;
        let text_body = self.templates.render(text_template_name, ctx)?;

        let email = Message::builder()
            .from(self.from.clone())
            .to(to)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::parse("text/plain; charset=utf-8").unwrap())
                            .body(text_body),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::parse("text/html; charset=utf-8").unwrap())
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
}
