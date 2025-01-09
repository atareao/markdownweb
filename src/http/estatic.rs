use axum::{
    extract::{Path, State},
    routing::get,
    response::Html,
    Router
};
use std::path::PathBuf;
use tower_http::services::ServeDir;
use tracing::{debug, error};
use crate::models::create_page_error;

use super::super::models::Config;

pub async fn router() -> Router {
    let config = Config::read_configuration().await;
    let path = config.destination.clone();
    
    debug!("Serving from: {}", &path);
    Router::new()
        .nest_service("/assets", ServeDir::new(&config.assets))
        .route("/", get(get_root))
        .route("/{*tail}", get(get_index))
        .with_state(config)
}

async fn get_root(state: State<Config>) -> Html<String>{
    debug!("=== root ===");
    get_index(state, Path("".to_string())).await
}
async fn get_index(State(config): State<Config>, Path(path): Path<String>) -> Html<String>{
    debug!("=== directory: {} ===", config.destination);
    debug!("Destination: {:?}", &config.destination);
    debug!("Path: {:?}", &path);
    let index_path = PathBuf::new()
        .join(&config.destination)
        .join(&path)
        .join("index.html");
    debug!("Index path: {:?}", index_path);
    if let Ok(true) = tokio::fs::try_exists(&index_path).await {
        match tokio::fs::read_to_string(&index_path).await{
            Ok(content) => {
                Html(content)
            },
            Err(e) => {
                error!("Error: {}", e);
                create_page_error(500, &e.to_string(), &config.site)
            },
        }
    }else{
        error!("Error. directory {:?} not exists", &index_path);
        create_page_error(404, "Page not found", &config.site)
    }
}
