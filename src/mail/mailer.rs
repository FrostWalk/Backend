use lettre::message::{header::ContentType, Mailbox, Message, MultiPart, SinglePart};
use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport,
    Tokio1Executor,
};
use url::Url;

use minijinja::Value as JinjaValue;

use super::template::TemplateEngine;

type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;
type Result<T> = std::result::Result<T, DynError>;

pub struct Mailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
    base_url: Url,
    templates: TemplateEngine,
}

impl Mailer {
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

    fn confirmation_link(&self, token: &str) -> Result<Url> {
        let mut url = self.base_url.join("/auth/confirm")?;
        url.query_pairs_mut().append_pair("token", token);
        Ok(url)
    }

    async fn send_templated(
        &self, to_email: &str, to_name: String, subject: &str, html_template_name: &str,
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
        &self, to_email: &str, to_name: String, token: &str,
    ) -> Result<()> {
        let confirm_url = self.confirmation_link(token)?;

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
        &self, to_email: &str, to_name: String, reset_url: &str,
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
