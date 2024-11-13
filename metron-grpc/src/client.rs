use std::{future::Future, pin::Pin, task::Poll};

use anyhow::Result;
use metron_core::{Agent, Plan, Report, ReportKind};
use tower::Service;
use tracing::info;

use crate::proto;

#[allow(unused)]
pub struct AgentClient {
    client: proto::agent_client::AgentClient<tonic::transport::Channel>,
}

impl AgentClient {
    pub async fn connect(server_addr: String) -> Result<Self> {
        info!(server_addr, "agent client connecting to server");
        let client = proto::agent_client::AgentClient::connect(server_addr).await?;
        Ok(Self { client })
    }
}

// const PROXY_CHAN_SIZE: usize = 1024;
// const SERVER_ACCEPT_TIMEOUT: Duration = Duration::from_secs(1);

impl Agent for AgentClient {
    async fn exec(&self, plan: Plan) -> Result<()> {
        let req = proto::ExecRequest {
            plan: Some(plan.try_into()?),
        };

        let mut client = self.client.clone();
        client.exec(req).await?;

        Ok(())
    }

    async fn report(&self, kind: ReportKind) -> Result<Report> {
        let req = proto::ReportRequest {
            report_kind: TryInto::<proto::ReportKind>::try_into(kind)? as i32,
        };

        let mut client = self.client.clone();
        let res = client.report(req).await?;
        let res = res.into_inner();

        res.try_into()
    }

    async fn stop(&self) -> Result<()> {
        let req = proto::StopRequest {};
        let mut client = self.client.clone();
        client.stop(req).await?;

        Ok(())
    }
}

impl Service<()> for AgentClient {
    type Response = ();
    type Error = anyhow::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        // TODO: Actually query the self.inner.
        // "Readiness" can probably just mean that the agent is running and can be communicated with.
        // Because the proxy request could lower, e.g., the rate.
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: ()) -> Self::Future {
        Box::pin(async { Ok(()) })
    }
}
