use std::{
    error::Error,
    future::{self, Future},
    pin::Pin,
    task::Poll,
    time::Instant,
};

use anyhow::Result;
use hyper::{Body, Request, Response};
use metrics::{
    describe_counter, describe_gauge, describe_histogram, gauge, histogram, increment_counter,
};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use tower::{Layer, Service};

// TODO: Introduce known/allowed paths
// TODO: Rename PromHttpServerLayer to PromHttpLayer if there is nothing server specific about it.

const LATENCY_HISTOGRAM_BUCKETS: [f64; 12] = [
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0,
];

#[derive(Clone)]
pub struct PromHttpServerLayer {
    metrics_path: Option<String>,
}

impl PromHttpServerLayer {
    pub fn new(metrics_path: Option<String>) -> Self {
        Self { metrics_path }
    }
}

impl Default for PromHttpServerLayer {
    fn default() -> Self {
        Self::new(Some("/metrics".to_owned()))
    }
}

impl<S> Layer<S> for PromHttpServerLayer {
    type Service = PromHttpServerService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        PromHttpServerService::new(inner, self.metrics_path.clone())
    }
}

#[derive(Clone)]
pub struct PromHttpServerService<S> {
    inner: S,
    metrics_path: Option<String>,
    handle: PrometheusHandle,
}

impl<S> PromHttpServerService<S> {
    pub fn new(inner: S, metrics_path: Option<String>) -> Self {
        let builder = PrometheusBuilder::new()
            .set_buckets_for_metric(
                Matcher::Full("http_request_duration_seconds".to_owned()),
                &LATENCY_HISTOGRAM_BUCKETS,
            )
            .unwrap();
        let handle = builder.install_recorder().unwrap();

        describe_gauge!("build_info", "Software build and version information");
        describe_counter!("http_requests_total", "Total number of HTTP requests");
        describe_histogram!(
            "http_request_duration_seconds",
            "HTTP request latency distribution"
        );

        gauge!(
            "build_info", 1.0,
            "version" => "1.2.3",
            "revision" => "abcdefg",
            "branch" => "main",
            "rust_version" => "1.62.0"
        );

        Self {
            inner,
            metrics_path,
            handle,
        }
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
        increment_counter!("http_requests_total");

        // If a metrics path has been set and it is equal to the requested path, then we return
        // the standard Prometheus metrics text as the response body.
        if let Some(path) = &self.metrics_path && path == req.uri().path() {
            let resp = Response::builder()
                .status(200)
                .header(hyper::header::CONTENT_TYPE, "UTF-8")
                .body(Body::from(self.handle.render()))
                .map_err(Into::into);

            return Box::pin(future::ready(resp));
        }

        let mut inner = self.inner.clone();
        Box::pin(async move {
            let method = req.method().clone();
            let uri = req.uri().clone();
            let start = Instant::now();
            let resp = inner.call(req).await.map_err(Into::into)?;

            histogram!(
                "http_request_duration_seconds",
                start.elapsed(),
                "status" => resp.status().as_u16().to_string(),
                "method" => method.to_string(),
                "path" => uri.path().to_owned(),
            );

            Ok(resp)
        })
    }
}
