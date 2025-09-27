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

        Ok(Self { env })
    }

    pub fn render(&self, name: &str, data: JinjaValue) -> Result<String> {
        let tmpl = self.env.get_template(name)?;
        Ok(tmpl.render(data)?)
    }
}
