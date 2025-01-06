use axum::{
    extract::{Path, State},
    routing::get,
    response::Html,
    Router
};
use tracing::debug;
use super::super::models::Config;

pub async fn router() -> Router {
    let config = Config::read_configuration().await;
    let path = config.destination.clone();
    
    debug!("Serving from: {}", &path);
    Router::new()
        .route("/{*tail}", get(get_index))
        .with_state(config)
}

async fn get_index(State(config): State<Config>, Path(path): Path<String>) -> Html<String>{
    debug!("directory: {}", config.destination);
    let index = format!("{}/{}/index.html", config.destination, path);
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
