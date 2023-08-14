mod config;
mod prom;

use std::{collections::HashMap, net::SocketAddr, time::Duration};

use anyhow::Result;
use axum::{
    extract::{ConnectInfo, Query},
    http::{header::HeaderMap, Method, Request, StatusCode, Uri},
    Json, Router,
};
use opentelemetry::global;
use serde_json::{json, Value};
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::{debug, info, info_span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

pub use self::config::Config;

/// Starts the echo server.
pub async fn serve(_config: &Config) -> Result<()> {
    init_tracing();

    let app =
        Router::new()
            .fallback(handle_request)
            .layer(
                TraceLayer::new_for_http().make_span_with(|req: &Request<_>| {
                    info_span!(
                        "http_request",
                        method = ?req.method(),
                        uri = ?req.uri(),
                    )
                }),
            );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    info!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown())
        .await?;

    Ok(())
}

/// Header that allows the HTTP status to be specified by the client.
const INJECT_STATUS_HEADER: &str = "x-metron-inject-status";

/// header that allows a server delay to be specified by the client.
const INJECT_DELAY_HEADER: &str = "x-metron-inject-delay";

/// Maximum allowed server delay.
const MAX_DELAY: Duration = Duration::from_secs(10);

/// Generic handler for all requests.
async fn handle_request(
    uri: Uri,
    method: Method,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> (StatusCode, Json<Value>) {
    // Collect the headers into something serializable.
    let headers: HashMap<String, String> = headers
        .iter()
        .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.to_string(), v.to_string())))
        .collect();

    // Allow the client to optionally specify the response status code.
    let status = headers
        .get(INJECT_STATUS_HEADER)
        .and_then(|v| v.parse().ok())
        .unwrap_or(StatusCode::OK);

    // Allow the client to optionally specify a delay.
    let delay = headers
        .get(INJECT_DELAY_HEADER)
        .and_then(|s| s.parse::<humantime::Duration>().ok())
        .map(|d| d.min(MAX_DELAY));

    // Pause if required.
    if let Some(delay) = delay {
        tokio::time::sleep(delay).await;
    }

    (
        status,
        Json(json!({
            "method": method.to_string(),
            "headers": headers,
            "parameters": params,
            "uri": uri.to_string(),
            "client_ip": addr.ip(),
        })),
    )
}

/// Returns a `Future` that completes when Ctrl-C is received.
async fn shutdown() {
    signal::ctrl_c().await.expect("failed to listen for ctrl-c");
    info!("bye");
}

fn init_tracing() {
    // Set the OTel propagation format (others W3C, B3, etc).
    global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());

    // Sets up the machinery needed to export data to Jaeger.
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("metron_echo")
        .install_simple()
        .expect("could not create Jaeger tracer");

    // Console layer for local debugging.
    let console_layer = console_subscriber::spawn();
    // console_subscriber::init();

    // OTel layer for exporting trace data to Jaeger.
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Console layer for local debugging.
    let stdout_layer = tracing_subscriber::fmt::layer();

    // Filter to allow using RUST_LOG to filter trace data.
    // TODO: How to get this working with the console layer?
    // See https://github.com/tokio-rs/console/issues/76
    // Which links to https://github.com/tokio-rs/tracing/pull/1523
    // let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
    //     .unwrap_or_else(|_| "user_service=debug,tower_http=debug".into());

    // Create and install the subscriber that processes all tracing events. The layers are invoked
    // from bottom to top.
    let filter_level = tracing_subscriber::filter::LevelFilter::DEBUG;
    tracing_subscriber::registry()
        .with(console_layer)
        .with(otel_layer.with_filter(filter_level))
        .with(stdout_layer.with_filter(filter_level))
        .try_init()
        .expect("could not install tracing subscriber");

    debug!("tracing has been initialized")
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
