mod config;
mod prom;

use std::{collections::HashMap, net::SocketAddr};

use anyhow::Result;
use axum::{
    extract::{ConnectInfo, Path},
    headers::UserAgent,
    http::{header::HeaderMap, StatusCode, Uri},
    routing::{delete, get, patch, post, put},
    Json, Router, TypedHeader,
};
use log::info;
use serde_json::{json, Value};
use tokio::signal;

pub use self::config::Config;

pub async fn serve(_config: &Config) -> Result<()> {
    // Implement basic httpbin spec: https://httpbin.org/legacy.
    let app = Router::new()
        .fallback(not_found)
        .route("/", get(get_root))
        .route("/ip", get(get_ip))
        .route("/headers", get(get_headers))
        .route("/get", get(get_headers))
        .route("/post", post(get_headers))
        .route("/patch", patch(get_headers))
        .route("/put", put(get_headers))
        .route("/delete", delete(get_headers))
        .route("/user-agent", get(get_user_agent))
        .route("/status/:code", get(get_status))
        .route("/delay/:duration", get(get_delay));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
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
    (StatusCode::OK, "Hello".to_owned())
}

async fn get_ip(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({ "origin": addr.ip() })))
}

async fn get_headers(headers: HeaderMap) -> (StatusCode, Json<Value>) {
    let mut m = HashMap::new();
    for (k, v) in headers.iter() {
        if let Ok(v) = v.to_str() {
            m.insert(k.to_string(), v.to_string());
        }
    }

    (StatusCode::OK, Json(json!({ "headers": m })))
}

async fn get_user_agent(TypedHeader(user_agent): TypedHeader<UserAgent>) -> Json<Value> {
    Json(json!({ "user-agent": user_agent.to_string() }))
}

async fn get_status(Path(status): Path<u16>) -> StatusCode {
    StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_REQUEST)
}

async fn get_delay(Path(_delay): Path<String>) -> (StatusCode, String) {
    // TODO next
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
