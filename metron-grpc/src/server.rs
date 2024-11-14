use std::sync::Arc;

use anyhow::Result;
use metron_core::{Agent, ReportKind};
use tonic::{Request, Response, Status};
use tracing::info;

use crate::proto;

#[derive(Clone)]
pub struct AgentServer<T> {
    agent: Arc<T>,
    port: u16,
}

impl<T> AgentServer<T>
where
    T: Agent + Send + Sync + 'static,
{
    pub fn new(agent: T, port: u16) -> Self {
        Self {
            agent: Arc::new(agent),
            port,
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
    async fn exec(
        &self,
        request: Request<proto::ExecRequest>,
    ) -> std::result::Result<Response<proto::ExecResponse>, Status> {
        let plan = request
            .get_ref()
            .plan
            .as_ref()
            .ok_or(Status::invalid_argument("missing plan"))?
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("invalid plan: {}", e)))?;
        self.agent
            .exec(plan)
            .await
            .map_err(|e| Status::internal(format!("agent server error: {}", e)))?;

        Ok(Response::new(proto::ExecResponse {}))
    }

    async fn report(
        &self,
        _request: Request<proto::ReportRequest>,
    ) -> std::result::Result<Response<proto::ReportResponse>, Status> {
        self.agent
            .report(ReportKind::DelayLatency)
            .await
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?;
        Ok(Response::new(proto::ReportResponse { histogram: vec![] }))
    }

    async fn stop(
        &self,
        _request: Request<proto::StopRequest>,
    ) -> std::result::Result<Response<proto::StopResponse>, Status> {
        self.agent
            .stop()
            .await
            .map_err(|e| Status::internal(format!("internal error: {}", e)))?;
        Ok(Response::new(proto::StopResponse {}))
    }
}
