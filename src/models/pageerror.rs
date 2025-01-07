use axum::response::Html;
use minijinja::context;
use super::{Config, ENV};

pub fn create_page_error(code: u16, message: &str, config: &Config) -> Html<String> {
        let ctx = context! {
            config => config,
            code => code,
            message => message,
        };
        if let Ok(template) = ENV.get_template("error.html") {
            if let Ok(rendered) = template.render(&ctx) {
                return Html(rendered);
            }
        }
        Html("Can not render template 'error.html'".to_string())
}
