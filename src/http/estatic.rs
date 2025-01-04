use axum::{
    body::Body,
    extract::{Path, Request, State},
    routing::get,
    middleware::{
        from_fn,
        Next,
    }, response::{self, IntoResponse, Html, Response}, routing::get_service, Router
};
use tower_http::services::ServeDir;
use std::env::var;
use tracing::{debug, error};
use super::super::models::Config;

const CONTENT_DIR: &str = "/public";

pub fn router() -> Router {
    let path = var("DESTINATION").unwrap_or(CONTENT_DIR.to_string());
    let config = Config::new(&path);
    debug!("Serving from: {}", &path);
    let serve_dir = ServeDir::new(path).append_index_html_on_directories(true);
    let serve_dir = get_service(serve_dir); //.handle_error(handle_error);
    Router::new()
        .route("/{*tail}", get(get_index))
        .fallback_service(serve_dir)
        //.layer(from_fn(content_type_middleware))
        .with_state(config)
}

pub async fn content_type_middleware(request: Request<Body>, next: Next) -> response::Response {
    let uri = request.uri().to_owned();
    let path = uri.path();

    let splited = path.split(".").collect::<Vec<_>>();

    let mut response = next.run(request).await;

    let content_type = if let Some(ext) = splited.last() {
        let extension = ext.to_owned().to_lowercase();

        match extension.as_str() {
            "html" => "text/html",
            "css" => "text/css",
            "js" => "text/javascript",
            "json" => "application/json",
            "png" => "image/png",
            "jpg" => "image/jpeg",
            "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "ico" => "image/x-icon",
            "ttf" => "font/ttf",
            "woff" => "font/woff",
            "woff2" => "font/woff2",
            "eot" => "application/vnd.ms-fontobject",
            "otf" => "font/otf",
            "txt" => "text/plain",
            "pdf" => "application/pdf",
            "doc" => "application/msword",
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "xls" => "application/vnd.ms-excel",
            "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "ppt" => "application/vnd.ms-powerpoint",
            "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "xml" => "application/xml",
            "zip" => "application/zip",
            "rar" => "application/x-rar-compressed",
            "7z" => "application/x-7z-compressed",
            "gz" => "application/gzip",
            "tar" => "application/x-tar",
            "swf" => "application/x-shockwave-flash",
            "flv" => "video/x-flv",
            "avi" => "video/x-msvideo",
            "mov" => "video/quicktime",
            "mp4" => "video/mp4",
            "mp3" => "audio/mpeg",
            "wav" => "audio/x-wav",
            "ogg" => "audio/ogg",
            "webm" => "video/webm",
            "mpg" => "video/mpeg",
            "mpeg" => "video/mpeg",
            "mpe" => "video/mpeg",
            "mp2" => "video/mpeg",
            "m4v" => "video/x-m4v",
            "3gp" => "video/3gpp",
            "3g2" => "video/3gpp2",
            "mkv" => "video/x-matroska",
            "amv" => "video/x-matroska",
            "m3u" => "audio/x-mpegurl",
            "m3u8" => "application/vnd.apple.mpegurl",
            "ts" => "video/mp2t",
            "f4v" => "video/mp4",
            "f4p" => "video/mp4",
            "f4a" => "video/mp4",
            "f4b" => "video/mp4",
            "webp" => "image/webp",
            "bmp" => "image/bmp",
            "tif" => "image/tiff",
            "tiff" => "image/tiff",
            "psd" => "image/vnd.adobe.photoshop",
            "ai" => "application/postscript",
            "eps" => "application/postscript",
            "ps" => "application/postscript",
            "dwg" => "image/vnd.dwg",
            "dxf" => "image/vnd.dxf",
            "rtf" => "application/rtf",
            "odt" => "application/vnd.oasis.opendocument.text",
            "ods" => "application/vnd.oasis.opendocument.spreadsheet",
            "wasm" => "application/wasm",
            _ => "application/octet-stream",
        }
    } else {
        "unknown"
    };

    if let Ok(content_type) = content_type.parse() {
        response.headers_mut().insert("Content-Type", content_type);
    }

    response
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
