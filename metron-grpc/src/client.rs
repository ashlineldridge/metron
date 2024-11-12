use std::{future::Future, pin::Pin, task::Poll, time::Duration};

use anyhow::{Context, Result};
use metron_core::{Agent, AgentRequest};
use tokio::sync::{mpsc, oneshot};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tower::Service;
use tracing::info;

use crate::proto;

struct ControlRequest {
    inner: proto::ControlRequest,
    done_tx: oneshot::Sender<()>,
}

#[allow(unused)]
pub struct AgentClient {
    client: proto::agent_client::AgentClient<tonic::transport::Channel>,
    req_tx: mpsc::Sender<ControlRequest>,
}

impl AgentClient {
    pub async fn connect(server_addr: String) -> Result<Self> {
        info!(server_addr, "agent client connecting to server");

        let mut client = proto::agent_client::AgentClient::connect(server_addr).await?;

        let (req_tx, req_rx) = mpsc::channel(PROXY_CHAN_SIZE);
        let req_stream = ReceiverStream::<ControlRequest>::new(req_rx).map(|req| req.inner);

        let res = client.control(req_stream).await?;
        let mut res_stream = res.into_inner();
        let (res_tx, mut res_rx) = mpsc::channel(PROXY_CHAN_SIZE);

        let _h1 = tokio::spawn(async move {
            info!("agent_client: spawned result stream processor");
            while let Some(res) = res_stream.next().await {
                info!("agent_client: got result stream response: {:?}", res);
                let res = res?;
                // let
                res_tx.send(res).await?;
            }

            info!("agent_client: closing result stream processor");
            Result::<()>::Ok(())
        });

        let _h2 = tokio::spawn(async move {
            info!("agent_client: spawned result channel processor");
            while let Some(res) = res_rx.recv().await {
                info!("agent_client: got result channel response: {:?}", res);
            }

            info!("agent_client: closing result channel processor");
            Result::<()>::Ok(())
        });

        /*
        Problem:
        What is the "result"? The task above is pulling on res_rx but what is it going to do with it?
        What is the common interface(s) across Agent types?
        */

        Ok(Self { client, req_tx })
    }
}

const PROXY_CHAN_SIZE: usize = 1024;
const SERVER_ACCEPT_TIMEOUT: Duration = Duration::from_secs(1);

impl Agent for AgentClient {
    async fn execute(&self, req: AgentRequest) -> Result<()> {
        let (done_tx, done_rx) = oneshot::channel();
        let req = ControlRequest {
            inner: proto::ControlRequest {
                plan: Some(req.plan.try_into()?),
                start: Some(req.start.into()),
            },
            done_tx,
        };

        self.req_tx.send(req).await.context("could not send")?;
        tokio::time::timeout(SERVER_ACCEPT_TIMEOUT, done_rx).await??;

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
