use axum::{
    extract::{Path, State},
    routing::get,
    response::Html,
    Router
};
use std::env::var;
use tracing::debug;
use super::super::models::Config;

const CONTENT_DIR: &str = "/public";

pub fn router() -> Router {
    let path = var("DESTINATION").unwrap_or(CONTENT_DIR.to_string());
    let config = Config::new(&path);
    debug!("Serving from: {}", &path);
    Router::new()
        .route("/{*tail}", get(get_index))
        .with_state(config)
}

async fn get_index(State(config): State<Config>, Path(path): Path<String>) -> Html<String>{
    debug!("directory: {}", config.directory);
    let index = format!("{}/{}/index.html", config.directory, path);
    if let Ok(true) = tokio::fs::try_exists(&index).await {
        match tokio::fs::read_to_string(index).await{
            Ok(content) => {
                debug!("Content: {}", content);
                Html(content)

            },
            Err(e) => {
                Html(e.to_string())
            },
        }
    }else{
        Html("error".to_string())
    }
}
