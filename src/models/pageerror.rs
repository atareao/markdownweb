use super::{Site, ENV};
use axum::response::Html;
use minijinja::context;

pub fn create_page_error(code: u16, message: &str, site: &Site) -> Html<String> {
    let ctx = context! {
        site => site,
        code => code,
        message => message,
    };
    match ENV.get_template("error.html") {
        Ok(template) => match template.render(&ctx) {
            Ok(rendered) => {
                Html(rendered)
            }
            Err(e) => {
                Html(format!("Error: {}", e))
            }
        },
        Err(e) => {
            Html(format!("Error: {}", e))
        }
    }
}
