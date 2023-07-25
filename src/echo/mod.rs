mod config;
mod prom;

use std::net::SocketAddr;

use anyhow::Result;
use axum::{
    http::{StatusCode, Uri},
    routing::{delete, get, patch, post, put},
    Router,
};
use log::info;
use tokio::signal;

pub use self::config::Config;

pub async fn serve(_config: &Config) -> Result<()> {
    // Implement basic httpbin spec: https://httpbin.org/legacy.
    let app = Router::new()
        .fallback(not_found)
        .route("/", get(get_root))
        .route("/ip", get(get_ip))
        .route("/uuid", get(get_uuid))
        .route("/headers", get(get_headers))
        .route("/get", get(get_headers))
        .route("/post", post(get_headers))
        .route("/patch", patch(get_headers))
        .route("/put", put(get_headers))
        .route("/delete", delete(get_headers))
        .route("/status/:code", get(get_status))
        .route("/delay/:duration", get(get_delay));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown())
        .await?;

    Ok(())
}

async fn shutdown() {
    signal::ctrl_c().await.expect("failed to listen for event");
    info!("bye");
}

async fn not_found(uri: Uri) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("No route for {}", uri))
}

async fn get_root() -> (StatusCode, String) {
    (StatusCode::OK, "Hello\n".to_owned())
}

async fn get_ip() -> (StatusCode, String) {
    (StatusCode::OK, "Hello\n".to_owned())
}

async fn get_uuid() -> (StatusCode, String) {
    (StatusCode::OK, "Hello\n".to_owned())
}

async fn get_headers() -> (StatusCode, String) {
    (StatusCode::OK, "Hello\n".to_owned())
}

async fn get_status() -> (StatusCode, String) {
    (StatusCode::OK, "Hello\n".to_owned())
}

async fn get_delay() -> (StatusCode, String) {
    (StatusCode::OK, "Hello\n".to_owned())
}

// See above:
// Starting point idea for getting back into the project:
// Wire up this server so that it has a basic decent config
// file (e.g. imagine using it to load test a proxy) and can
// introduce jitter. Then instrument it with tracing and
// Prom crate. Think about names, project structure, modules,
// and testing while you're doing this.
// Re: CLI structure, it makes sense to have echo / run
//
// Could you also make it configurable as a generic GPRC endpoint?
// E.g. to test something like the throughput of a GRPC proxy?
