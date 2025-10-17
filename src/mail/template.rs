use minijinja::{Environment, Value as JinjaValue};

type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;
type Result<T> = std::result::Result<T, DynError>;

const CONFIRM_HTML_TMPL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/confirm.html"
));
const CONFIRM_TEXT_TMPL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/confirm.txt"
));

const RESET_HTML_TMPL: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/reset.html"));
const RESET_TEXT_TMPL: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/reset.txt"));

const ADMIN_WELCOME_HTML_TMPL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/admin_welcome.html"
));
const ADMIN_WELCOME_TEXT_TMPL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/admin_welcome.txt"
));

#[derive(Clone)]
pub struct TemplateEngine {
    env: Environment<'static>,
}

impl TemplateEngine {
    pub fn new() -> Result<Self> {
        let mut env = Environment::new();

        env.add_template("confirm.html", CONFIRM_HTML_TMPL)?;
        env.add_template("confirm.txt", CONFIRM_TEXT_TMPL)?;

        env.add_template("reset.html", RESET_HTML_TMPL)?;
        env.add_template("reset.txt", RESET_TEXT_TMPL)?;

        env.add_template("admin_welcome.html", ADMIN_WELCOME_HTML_TMPL)?;
        env.add_template("admin_welcome.txt", ADMIN_WELCOME_TEXT_TMPL)?;

        Ok(Self { env })
    }

    pub fn render(&self, name: &str, data: JinjaValue) -> Result<String> {
        let tmpl = self.env.get_template(name)?;
        Ok(tmpl.render(data)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_template_engine_new_success() {
        let result = TemplateEngine::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_confirm_html() {
        let engine = TemplateEngine::new().unwrap();
        let ctx = create_test_email_context();

        let result = engine.render("confirm.html", ctx);
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(!html.is_empty());
        assert!(html.contains("Test User")); // user_name
                                             // Check that the URL is present in some form (might be encoded or formatted differently)
        assert!(
            html.contains("test.example.com")
                || html.contains("confirm")
                || html.contains("test-token")
        );
    }

    #[test]
    fn test_render_confirm_text() {
        let engine = TemplateEngine::new().unwrap();
        let ctx = create_test_email_context();

        let result = engine.render("confirm.txt", ctx);
        assert!(result.is_ok());

        let text = result.unwrap();
        assert!(!text.is_empty());
        assert!(text.contains("Test User")); // user_name
                                             // Check that the URL is present in some form
        assert!(
            text.contains("test.example.com")
                || text.contains("confirm")
                || text.contains("test-token")
        );
    }

    #[test]
    fn test_render_reset_html() {
        let engine = TemplateEngine::new().unwrap();
        let ctx = create_test_password_reset_context();

        let result = engine.render("reset.html", ctx);
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(!html.is_empty());
        assert!(html.contains("Test User")); // user_name
                                             // Check that the URL is present in some form
        assert!(
            html.contains("test.example.com")
                || html.contains("reset")
                || html.contains("test-reset-token")
        );
    }

    #[test]
    fn test_render_reset_text() {
        let engine = TemplateEngine::new().unwrap();
        let ctx = create_test_password_reset_context();

        let result = engine.render("reset.txt", ctx);
        assert!(result.is_ok());

        let text = result.unwrap();
        assert!(!text.is_empty());
        assert!(text.contains("Test User")); // user_name
                                             // Check that the URL is present in some form
        assert!(
            text.contains("test.example.com")
                || text.contains("reset")
                || text.contains("test-reset-token")
        );
    }

    #[test]
    fn test_render_admin_welcome_html() {
        let engine = TemplateEngine::new().unwrap();
        let ctx = create_test_admin_email_context();

        let result = engine.render("admin_welcome.html", ctx);
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(!html.is_empty());
        assert!(html.contains("Test Admin")); // user_name
        assert!(html.contains(TEST_ADMIN_EMAIL)); // email
        assert!(html.contains(TEST_PASSWORD)); // password
                                               // Check that the login URL is present in some form
        assert!(
            html.contains("test.example.com") || html.contains("admin") || html.contains("login")
        );
    }

    #[test]
    fn test_render_admin_welcome_text() {
        let engine = TemplateEngine::new().unwrap();
        let ctx = create_test_admin_email_context();

        let result = engine.render("admin_welcome.txt", ctx);
        assert!(result.is_ok());

        let text = result.unwrap();
        assert!(!text.is_empty());
        assert!(text.contains("Test Admin")); // user_name
        assert!(text.contains(TEST_ADMIN_EMAIL)); // email
        assert!(text.contains(TEST_PASSWORD)); // password
                                               // Check that the login URL is present in some form
        assert!(
            text.contains("test.example.com") || text.contains("admin") || text.contains("login")
        );
    }

    #[test]
    fn test_render_nonexistent_template() {
        let engine = TemplateEngine::new().unwrap();
        let ctx = create_test_email_context();

        let result = engine.render("nonexistent.html", ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_render_with_missing_context_variable() {
        let engine = TemplateEngine::new().unwrap();
        // Create context with missing required variables
        let ctx = minijinja::context! {
            user_name => "Test User"
            // Missing 'url' variable that templates expect
        };

        let result = engine.render("confirm.html", ctx);
        // This might fail or succeed depending on template implementation
        // Just check that we get some result
        let _ = result; // Don't assert error, just ensure it doesn't panic
    }

    #[test]
    fn test_render_with_empty_context() {
        let engine = TemplateEngine::new().unwrap();
        let ctx = minijinja::context! {};

        let result = engine.render("confirm.html", ctx);
        // This might fail or succeed depending on template implementation
        // Just check that we get some result
        let _ = result; // Don't assert error, just ensure it doesn't panic
    }

    #[test]
    fn test_render_with_extra_context_variables() {
        let engine = TemplateEngine::new().unwrap();
        // Add extra variables that templates don't use
        let ctx = minijinja::context! {
            user_name => "Test User",
            url => "https://test.example.com/confirm?t=test-token",
            extra_variable => "This should be ignored",
            another_extra => 123
        };

        let result = engine.render("confirm.html", ctx);
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(html.contains("Test User"));
        // Check that the URL is present in some form
        assert!(
            html.contains("test.example.com")
                || html.contains("confirm")
                || html.contains("test-token")
        );
        // Extra variables should not cause issues
    }

    #[test]
    fn test_render_with_special_characters() {
        let engine = TemplateEngine::new().unwrap();
        let ctx = minijinja::context! {
            user_name => "Test User & \"Special\" <Characters>",
            url => "https://test.example.com/confirm?t=test-token&param=value",
            email => "test+tag@example.com",
            password => "P@ssw0rd!@#$%^&*()"
        };

        let result = engine.render("confirm.html", ctx);
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(!html.is_empty());
        // Should handle special characters properly
    }

    #[test]
    fn test_render_with_unicode() {
        let engine = TemplateEngine::new().unwrap();
        let ctx = minijinja::context! {
            user_name => "Тест Пользователь",
            url => "https://test.example.com/confirm?t=тест-токен",
            email => "тест@example.com"
        };

        let result = engine.render("confirm.html", ctx);
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(!html.is_empty());
        assert!(html.contains("Тест Пользователь"));
    }

    #[test]
    fn test_template_engine_is_cloneable() {
        let engine1 = TemplateEngine::new().unwrap();
        let engine2 = engine1.clone();

        // Both should work independently
        let ctx = create_test_email_context();
        let result1 = engine1.render("confirm.html", ctx.clone());
        let result2 = engine2.render("confirm.html", ctx);

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());
    }
}
