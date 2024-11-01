use metron_core::{Plan, Report};
use thiserror::Error;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

use crate::proto;

#[derive(Clone)]
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
    pub async fn test(&mut self, plan: &Plan) -> Result<(), AgentClientError> {
        let plan = plan.try_into()?;
        self.inner
            .test(proto::TestRequest { plan: Some(plan) })
            .await?;
        Ok(())
    }

    pub async fn cancel(&mut self) -> Result<(), AgentClientError> {
        self.inner.cancel(proto::CancelRequest {}).await?;
        Ok(())
    }

    pub async fn report(&mut self) -> Result<Report, AgentClientError> {
        let res = self
            .inner
            .report(proto::ReportRequest { duration: None })
            .await?;

        // TODO: Convert proto report into domain object.
        let _report = res.into_inner();

        Ok(Report {})
    }

    // TODO: Don't expose the proto from here. Use a domain type.
    pub async fn proxy(
        &mut self,
    ) -> Result<(Sender<proto::ProxyRequest>, Receiver<proto::ProxyResponse>), AgentClientError>
    {
        let (req_tx, req_rx) = mpsc::channel(PROXY_CHAN_SIZE);
        let req_stream = ReceiverStream::new(req_rx);

        let res = self.inner.proxy(req_stream).await?;
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

#[derive(Error, Debug)]
pub enum AgentClientError {
    #[error(transparent)]
    TransportError(#[from] tonic::transport::Error),

    #[error(transparent)]
    StatusError(#[from] tonic::Status),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}
