use std::{pin::Pin, sync::Arc};

use anyhow::Result;
use metron_core::Agent;
use tokio::sync::Semaphore;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};
use tracing::info;

use crate::proto;

#[derive(Clone)]
pub struct AgentServer<T> {
    agent: Arc<T>,
    port: u16,
    lock: Arc<Semaphore>,
}

impl<T> AgentServer<T>
where
    T: Agent + Send + Sync + 'static,
{
    pub fn new(agent: T, port: u16) -> Self {
        Self {
            agent: Arc::new(agent),
            port,
            lock: Arc::new(Semaphore::new(1)),
        }
    }

    pub async fn run(self) -> Result<()> {
        let address = format!("[::1]:{}", self.port).parse()?;
        let server = proto::agent_server::AgentServer::new(self);

        info!("agent_server: metron server listening on {}", address);
        tonic::transport::Server::builder()
            .add_service(server)
            .serve(address)
            .await?;

        Ok(())
    }
}

#[tonic::async_trait]
impl<T> proto::agent_server::Agent for AgentServer<T>
where
    T: Agent + Send + Sync + 'static,
{
    type ControlStream =
        Pin<Box<dyn Stream<Item = Result<proto::ControlResponse, Status>> + Send + 'static>>;

    async fn control(
        &self,
        request: Request<Streaming<proto::ControlRequest>>,
    ) -> Result<Response<Self::ControlStream>, Status> {
        info!("agent_server: received control stream RPC");

        // Acquire permit to ensure that the server is controlled by a single client.
        let permit = self
            .lock
            .clone()
            .try_acquire_owned()
            .map_err(|_| Status::failed_precondition("control stream in progress"))?;

        let agent = self.agent.clone();
        let mut stream = request.into_inner();
        let output = async_stream::try_stream! {
            info!("agent_server: building stream: {:?}", permit);
            while let Some(req) = stream.next().await {
                info!("agent_server: received control message via stream");
                let req = req?;
                let req = req.try_into().map_err(|e| Status::invalid_argument(format!("invalid control request: {}", e)))?;
                agent.execute(req).await.map_err(|e| Status::internal(format!("agent server error: {}", e)))?;
                yield proto::ControlResponse { };
            }

            info!("agent_server: control stream is closing");

            // Drop the permit so that a subsequent control RPC can be executed.
            drop(permit);
        };

        info!("agent_server: returning control stream");
        Ok(Response::new(Box::pin(output) as Self::ControlStream))
    }
}
