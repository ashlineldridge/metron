mod config;

use std::{
    error::Error,
    future::{self, Future},
    net::SocketAddr,
    pin::Pin,
    task::Poll, time::Instant,
};

use anyhow::Result;
use hyper::http;
use hyper::{Body, Request, Response, Server};
use log::info;
use prometheus::{HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry, TextEncoder};
use tower::{make::Shared, Layer, Service, ServiceBuilder};

pub use self::config::Config;

// TODO: Introduce known/allowed paths
// TODO: Create appropriate bins for histograms (lowest res seems to be 5ms?)

pub async fn serve(config: &Config) -> Result<()> {
    let server = EchoServer::new(config.clone());
    let service = ServiceBuilder::new()
        .layer(PromHttpServerLayer::new(Some("/metrics".to_owned())))
        .service(server);

    info!("Server listening on :{}", config.port);

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    Server::bind(&addr).serve(Shared::new(service)).await?;

    Ok(())
}

#[derive(Clone)]
struct EchoServer {
    #[allow(dead_code)]
    config: Config,
}

impl EchoServer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl Service<Request<Body>> for EchoServer {
    type Response = Response<Body>;
    type Error = http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        // Always ready for now. In the future, if we want to want to provide the ability to
        // manipulate server latency, we can do that here (or in a dedicated layer).
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Request<Body>) -> Self::Future {
        let result = Response::builder()
            .status(200)
            .header(hyper::header::CONTENT_TYPE, "utf-8")
            .body(Body::from("This server sees you."));

        let fut = future::ready(result);
        Box::pin(fut)
    }
}

// TODO: Rename to PromHttpLayer if there is nothing server specific about it.
#[derive(Clone)]
struct PromHttpServerLayer {
    metrics_path: Option<String>,
}

impl PromHttpServerLayer {
    pub fn new(metrics_path: Option<String>) -> Self {
        Self { metrics_path }
    }
}

impl<S> Layer<S> for PromHttpServerLayer {
    type Service = PromHttpServerService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        PromHttpServerService::new(inner, self.metrics_path.clone())
    }
}

#[derive(Clone)]
struct PromHttpServerService<S> {
    inner: S,
    metrics_path: Option<String>,
    registry: Registry,
    http_requests_total: IntCounterVec,
    http_request_duration_seconds: HistogramVec,
}

impl<S> PromHttpServerService<S> {
    pub fn new(inner: S, metrics_path: Option<String>) -> Self {
        // Create metric collectors and then register them with the registry created below.
        // These calls only fail if the input arguments are bad; since we're specifying
        // them statically it's fine to unwrap.

        let build_info = prometheus::IntGaugeVec::new(
            Opts::new("build_info", "Software build and version information"),
            &["version", "revision", "branch", "rust_version"],
        )
        .unwrap();

        build_info
            .with_label_values(&["1.2.3", "abcdefg", "main", "1.62.0"])
            .set(1);

        let http_requests_total = IntCounterVec::new(
            Opts::new("http_requests_total", "Total number of HTTP requests"),
            &["method", "path"],
        )
        .unwrap();
        let http_request_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request latency distribution",
            ),
            &["status", "method", "path"],
        )
        .unwrap();

        let registry = Registry::default();
        registry.register(Box::new(build_info)).unwrap();
        registry
            .register(Box::new(http_requests_total.clone()))
            .unwrap();
        registry
            .register(Box::new(http_request_duration_seconds.clone()))
            .unwrap();

        Self {
            inner,
            metrics_path,
            registry,
            http_requests_total,
            http_request_duration_seconds,
        }
    }

    fn metrics_response(&self) -> Result<Response<Body>> {
        let mut buf = String::new();
        let encoder = TextEncoder::new();
        encoder.encode_utf8(&self.registry.gather(), &mut buf)?;

        let resp = Response::builder()
            .status(200)
            .header(hyper::header::CONTENT_TYPE, "UTF-8")
            .body(Body::from(buf))?;

        Ok(resp)
    }
}

impl<S> Service<Request<Body>> for PromHttpServerService<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn Error + Send + Sync>> + 'static,
{
    type Response = S::Response;
    type Error = Box<dyn Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        self.http_requests_total
            .with_label_values(&[req.method().as_str(), req.uri().path()])
            .inc();

        // If a metrics path has been set and it is equal to the requested path, then we return
        // the standard Prometheus metrics text as the response body.
        if let Some(path) = &self.metrics_path && path == req.uri().path() {
            match self.metrics_response() {
                Ok(resp) => return Box::pin(future::ready(Ok(resp))),
                Err(err) => return Box::pin(future::ready(Err(err.into()))),
            }
        }

        let mut inner = self.inner.clone();
        let http_request_duration_seconds = self.http_request_duration_seconds.clone();
        Box::pin(async move {
            let method = req.method().clone();
            let uri = req.uri().clone();
            let start = Instant::now();
            let resp = inner.call(req).await.map_err(Into::into)?;
            let elapsed = start.elapsed().as_secs_f64();

            http_request_duration_seconds
                .with_label_values(&[resp.status().as_str(), method.as_str(), uri.path()])
                .observe(elapsed);

            Ok(resp)
        })
    }
}
