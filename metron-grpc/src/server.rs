use std::{net::AddrParseError, pin::Pin};

use metron_core::Agent;
use thiserror::Error;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};

use crate::proto;

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

    pub async fn listen(self) -> Result<(), AgentServerError> {
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
    type ProxyStream =
        Pin<Box<dyn Stream<Item = Result<proto::ProxyResponse, tonic::Status>> + Send + 'static>>;

    async fn test(
        &self,
        request: Request<proto::TestRequest>,
    ) -> Result<Response<proto::TestResponse>, Status> {
        let req = request.into_inner();
        let plan = req.plan.ok_or(Status::invalid_argument("missing plan"))?;
        let plan = plan
            .try_into()
            .map_err(|_| Status::invalid_argument("invalid plan"))?;

        // TODO: Should I be cloning this?
        let mut agent = self.agent.clone();
        agent
            .test(&plan)
            .await
            .map_err(|_| Status::internal("agent error"))?;

        Ok(Response::new(proto::TestResponse {}))
    }

    async fn cancel(
        &self,
        _request: Request<proto::CancelRequest>,
    ) -> Result<Response<proto::CancelResponse>, Status> {
        let mut agent = self.agent.clone();
        agent
            .cancel()
            .await
            .map_err(|_| Status::internal("agent error"))?;
        Ok(Response::new(proto::CancelResponse {}))
    }

    async fn report(
        &self,
        _request: Request<proto::ReportRequest>,
    ) -> Result<Response<proto::ReportResponse>, tonic::Status> {
        let mut agent = self.agent.clone();
        let _report = agent
            .report()
            .await
            .map_err(|_| Status::internal("agent error"))?;
        Ok(Response::new(proto::ReportResponse { stats: None }))
    }

    async fn proxy(
        &self,
        request: Request<Streaming<proto::ProxyRequest>>,
    ) -> Result<Response<Self::ProxyStream>, tonic::Status> {
        let mut stream = request.into_inner();

        // let mut inner = self.agent.clone();
        let output = async_stream::try_stream! {
            while let Some(_req) = stream.next().await {
                // let req = req?;
                // let plan = req.plan.ok_or_else(|| tonic::Status::invalid_argument("missing plan"))?;
                // let plan: Plan = plan.try_into().unwrap();
                // let _target = "TODO".to_string();

                // inner.call(plan).await.expect("service call failed");

                yield proto::ProxyResponse { };
            }
        };

        Ok(Response::new(Box::pin(output) as Self::ProxyStream))
    }
}

// TODO: Can write a From to convert to a gRPC status used above

#[derive(Error, Debug)]
pub enum AgentServerError {
    #[error(transparent)]
    TransportError(#[from] tonic::transport::Error),

    #[error(transparent)]
    StatusError(#[from] tonic::Status),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}
