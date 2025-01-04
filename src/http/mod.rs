mod estatic;

use tower_http::trace::TraceLayer;
use core::error;
use std::env::var;
use tracing::debug;

use tower::ServiceBuilder;

pub async fn serve() -> Result<(), Box<dyn error::Error>>{
    let port: u16 = var("PORT").ok().and_then(|port| port.parse().ok()).unwrap_or(8080);
    debug!("Starting server on port: {}", port);
    let router = estatic::router().layer(
        ServiceBuilder::new()
            // Enables logging. Use `RUST_LOG=tower_http=debug`
            .layer(TraceLayer::new_for_http())
    );
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router)
        .await
        .map_err(|err| err.into())
}
