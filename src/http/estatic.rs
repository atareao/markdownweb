use axum::{
    Router,
    body::Body,
    routing::get,
    http::{
        Request,
        Response,
        StatusCode,
        Uri,
    },
    response::IntoResponse,
};
use tower_http::services::ServeDir;
use tower::ServiceExt;
use std::env::var;

const CONTENT_DIR: &str = "/public";

pub fn router() -> Router {
    Router::new()
        .nest_service("/", get(handler))
}

async fn handler(uri: Uri) -> Result<Response<Body>, (StatusCode, String)>{
    tracing::debug!("Uri: {}", uri);
    let res = get_static_file(&uri).await.unwrap().into_response();
    Ok(res)
}

async fn get_static_file(uri: &Uri) -> Result<Response<Body>, (StatusCode, String)> {
    tracing::debug!("Uri: {}", uri);
    let public_path = var("PUBLIC").unwrap_or(CONTENT_DIR.to_string());
    let filename = if uri.path() == "/" {
        format!("{}/{}", public_path, "index.html")
    }else{
        format!("{}{}", public_path, uri.path())
    };
    tracing::debug!("Filename: {}", filename);
    let def_uri = match tokio::fs::metadata(&filename).await {
        Ok(file) => if file.is_file(){
                uri.clone()
            }else{
                Uri::from_static("/404.html")
            }
        Err(_) => Uri::from_static("/404.html"),
    };
    let req = Request::builder().uri(def_uri).body(Body::empty()).unwrap();
    // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
    match ServeDir::new(CONTENT_DIR).oneshot(req).await {
        Ok(res) => {
            Ok(res.into_response())
        },
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", err),
        )),
    }
}
