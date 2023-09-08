use agent::{GrpcServer, Server};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let server = GrpcServer::new(8080);
    server.run().await?;

    Ok(())
}
