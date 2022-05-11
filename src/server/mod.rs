mod config;

use std::convert::Infallible;

use anyhow::Result;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};

pub use self::config::Config;

pub fn run(config: &Config) -> Result<()> {
    let runtime = if let Some(worker_threads) = config.worker_threads {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(worker_threads)
            .enable_all()
            .build()?
    } else {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
    };

    runtime.block_on(run_server(config))?;

    Ok(())
}

async fn run_server(config: &Config) -> Result<()> {
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
