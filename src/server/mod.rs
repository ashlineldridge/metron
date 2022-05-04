mod config;

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use std::convert::Infallible;
use tokio::runtime::Builder;

pub use self::config::Config;

pub fn run(config: &Config) -> Result<(), anyhow::Error> {
    let runtime = Builder::new_multi_thread()
        .worker_threads(config.worker_threads)
        .enable_all()
        .build()?;

    runtime.block_on(run_server(config))?;

    Ok(())
}

async fn run_server(config: &Config) -> Result<(), anyhow::Error> {
    let service =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });
    let addr = ([127, 0, 0, 1], config.port).into();
    let server = Server::bind(&addr).serve(service);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}

async fn handle_request(_: Request<Body>) -> std::result::Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello")))
}