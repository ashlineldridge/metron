mod config;
mod prom;

use std::{
    future::{self, Future},
    net::SocketAddr,
    pin::Pin,
    task::Poll,
};

use anyhow::Result;
use hyper::{http, Body, Request, Response};
use log::info;
use tower::{make::Shared, Service, ServiceBuilder};

pub use self::config::Config;
use crate::echo::prom::PromHttpServerLayer;

pub async fn serve(config: &Config) -> Result<()> {
    let server = Server::new(config.clone());
    let service = ServiceBuilder::new()
        .layer(PromHttpServerLayer::default())
        .service(server);

    info!("Server listening on :{}", config.port);

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    hyper::Server::bind(&addr)
        .serve(Shared::new(service))
        .await?;

    Ok(())
}

#[derive(Clone)]
struct Server {
    #[allow(dead_code)]
    config: Config,
}

impl Server {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl Service<Request<Body>> for Server {
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
            .body(Body::from("This server sees you.\n"));

        let fut = future::ready(result);
        Box::pin(fut)
    }
}
