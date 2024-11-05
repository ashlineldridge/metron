use std::{net::AddrParseError, pin::Pin};

use metron_core::Agent;
use thiserror::Error;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Streaming};

use crate::proto;

#[derive(Error, Debug)]
pub enum AgentServerError {
    #[error(transparent)]
    TransportError(#[from] tonic::transport::Error),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[derive(Clone)]
pub struct AgentServer<T> {
    agent: T,
    port: u16,
}

impl<T> AgentServer<T>
where
    T: Agent + Clone + Send + Sync + 'static,
{
    pub fn new(agent: T, port: u16) -> Self {
        Self { agent, port }
    }

    pub async fn run(self) -> Result<(), AgentServerError> {
        let address = format!("[::1]:{}", self.port)
            .parse()
            .map_err(|e: AddrParseError| AgentServerError::Unexpected(e.into()))?;

        let server = proto::agent_server::AgentServer::new(self);

        println!("metron server listening on {}", address);
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
    T: Agent + Clone + Send + Sync + 'static,
{
    type ControlStream =
        Pin<Box<dyn Stream<Item = Result<proto::ControlResponse, tonic::Status>> + Send + 'static>>;

    async fn control(
        &self,
        request: Request<Streaming<proto::ControlRequest>>,
    ) -> Result<Response<Self::ControlStream>, tonic::Status> {
        let mut stream = request.into_inner();

        let mut agent = self.agent.clone();
        let output = async_stream::try_stream! {
            while let Some(req) = stream.next().await {
                let req = req?;
                let plan = req.plan.ok_or_else(|| tonic::Status::invalid_argument("missing plan"))?;
                let plan = plan.try_into().unwrap();
                // let start = req.start_time.ok_or_else(|| tonic::Status::invalid_argument("missing start time"))?;
                // let start = start.try_into().unwrap();

                agent.test(&plan).await.expect("service call failed");

                yield proto::ControlResponse { };
            }
        };

        Ok(Response::new(Box::pin(output) as Self::ControlStream))
    }
}
