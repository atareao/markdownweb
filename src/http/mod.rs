mod estatic;

use axum::{
    extract::Request,
    ServiceExt
};
use tower_http::{
    normalize_path::NormalizePathLayer,
    trace::TraceLayer,
};
use core::error;
use std::env::var;
use tracing::debug;
use tower::{Layer, ServiceBuilder};

pub async fn serve() -> Result<(), Box<dyn error::Error>>{
    let port: u16 = var("PORT").ok().and_then(|port| port.parse().ok()).unwrap_or(8080);
    debug!("Starting server on port: {}", port);
    let router = estatic::router().await.layer(
        ServiceBuilder::new()
            // Enables logging. Use `RUST_LOG=tower_http=debug`
            .layer(TraceLayer::new_for_http())
    );
    let app = NormalizePathLayer::trim_trailing_slash().layer(router);
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, ServiceExt::<Request>::into_make_service(app))
        .await
        .map_err(|err| err.into())
}
