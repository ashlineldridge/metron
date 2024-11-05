use std::{future::Future, pin::Pin, task::Poll};

use metron_core::{Agent, AgentError, Plan, Report};
use thiserror::Error;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tower::Service;

use crate::proto;

#[derive(Error, Debug)]
pub enum AgentClientError {
    #[error(transparent)]
    TransportError(#[from] tonic::transport::Error),

    #[error(transparent)]
    StatusError(#[from] tonic::Status),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

// #[derive(Clone)]
pub struct AgentClient {
    inner: proto::agent_client::AgentClient<tonic::transport::Channel>,
}

impl AgentClient {
    pub async fn connect(server_addr: String) -> Result<Self, AgentClientError> {
        let inner = proto::agent_client::AgentClient::connect(server_addr).await?;
        Ok(Self { inner })
    }
}

const PROXY_CHAN_SIZE: usize = 1024;

impl AgentClient {
    pub async fn test(&self, plan: &Plan) -> Result<(), AgentClientError> {
        // self.inner.control
        // let plan = plan.try_into()?;
        // self.inner
        //     .clone()
        //     .test(proto::TestRequest { plan: Some(plan) })
        //     .await?;
        Ok(())
    }

    pub async fn cancel(&self) -> Result<(), AgentClientError> {
        // self.inner.clone().cancel(proto::CancelRequest {}).await?;
        Ok(())
    }

    pub async fn report(&self) -> Result<Report, AgentClientError> {
        // let res = self
        //     .inner
        //     .clone()
        //     .report(proto::ReportRequest { duration: None })
        //     .await?;

        // TODO: Convert proto report into domain object.
        // let _report = res.into_inner();

        Ok(Report {})
    }

    // TODO: Don't expose the proto from here. Use a domain type.
    pub async fn control(
        &self,
    ) -> Result<
        (
            Sender<proto::ControlRequest>,
            Receiver<proto::ControlResponse>,
        ),
        AgentClientError,
    > {
        let (req_tx, req_rx) = mpsc::channel(PROXY_CHAN_SIZE);
        let req_stream = ReceiverStream::new(req_rx);

        let res = self.inner.clone().control(req_stream).await?;
        let mut res_stream = res.into_inner();
        let (res_tx, res_rx) = mpsc::channel(PROXY_CHAN_SIZE);

        // TODO: Potentially need (and then don't need mut?):
        // tokio::pin!(res_stream);

        tokio::spawn(async move {
            while let Some(res) = res_stream.next().await {
                // TODO: Error handling? Won't this be silent?
                let res = res?;
                res_tx.send(res).await?;
            }
            anyhow::Result::<()>::Ok(())
        });

        Ok((req_tx, res_rx))
    }
}

impl Agent for AgentClient {
    async fn test(&mut self, _plan: &Plan) -> Result<(), AgentError> {
        // TODO: Create a control connection and send a oneshot plan message.
        // self.inner.c
        // self.inner.test(plan).await?;
        Ok(())
    }

    async fn cancel(&mut self) -> Result<(), AgentError> {
        // TODO: Create a control connection and send a oneshot empty plan message.
        // self.cancel().await?;
        Ok(())
    }
}

impl From<AgentClientError> for AgentError {
    fn from(value: AgentClientError) -> Self {
        AgentError::Unexpected(value.into())
    }
}

impl Service<()> for AgentClient {
    type Response = ();
    type Error = AgentClientError;
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
