mod proto {
    tonic::include_proto!("proto");
}

use std::{future::Future, net::AddrParseError, pin::Pin, task::Poll, time::Duration};

use metron::core::{HttpMethod, Plan};
use thiserror::Error;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Streaming};
use tower::Service;

use self::proto::metron_client;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    TransportError(#[from] tonic::transport::Error),

    #[error(transparent)]
    StatusError(#[from] tonic::Status),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[derive(Clone)]
pub struct MetronClient {
    inner: metron_client::MetronClient<tonic::transport::Channel>,
}

impl MetronClient {
    pub async fn connect(server_addr: String) -> Result<Self, Error> {
        let inner = metron_client::MetronClient::connect(server_addr).await?;

        Ok(Self { inner })
    }
}

impl MetronClient {
    async fn run(&mut self, plan: &Plan) -> Result<(), Error> {
        let target = plan
            .targets
            .first()
            .expect("where the target at?")
            .to_string();
        let outbound = async_stream::stream! {
            let mut interval = tokio::time::interval(Duration::from_secs(1));

            loop {
                let target = target.clone();
                interval.tick().await;
                println!("MetronClient sending plan for {target}");
                let request = proto::MetronRequest {
                    plan: Some(proto::Plan {
                        segments: vec![],
                        target,
                    }),
                };

                yield request;
            }
        };

        // TODO: Remove unwraps.
        let response = self.inner.run(Request::new(outbound)).await?;
        let mut inbound = response.into_inner();

        while let Some(res) = inbound.message().await? {
            println!("GOT METRON RESPONSE = {:?}", res);
        }

        Ok(())
    }
}

impl Service<Plan> for MetronClient {
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Plan) -> Self::Future {
        let mut metron = self.clone();
        Box::pin(async move { metron.run(&req).await })
    }
}

#[derive(Clone)]
pub struct MetronServer<S> {
    inner: S,
    port: u16,
}

impl<S> MetronServer<S>
where
    S: Service<Plan> + Send + Sync + Clone + 'static,
    S::Error: std::fmt::Debug, // This can be removed once proper error handling is in place
    S::Future: Send + 'static,
{
    pub fn new(inner: S, port: u16) -> Self {
        Self { inner, port }
    }

    pub async fn listen(self) -> Result<(), Error> {
        let address = format!("[::1]:{}", self.port)
            .parse()
            .map_err(|e: AddrParseError| Error::Unexpected(e.into()))?;

        let server = proto::metron_server::MetronServer::new(self);

        println!("metron server listening on {}", address);
        tonic::transport::Server::builder()
            .add_service(server)
            .serve(address)
            .await?;

        Ok(())
    }
}

#[tonic::async_trait]
impl<S> proto::metron_server::Metron for MetronServer<S>
where
    S: Service<Plan> + Send + Sync + Clone + 'static,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    type RunStream =
        Pin<Box<dyn Stream<Item = Result<proto::MetronResponse, tonic::Status>> + Send + 'static>>;

    async fn run(
        &self,
        request: Request<Streaming<proto::MetronRequest>>,
    ) -> Result<Response<Self::RunStream>, tonic::Status> {
        let mut stream = request.into_inner();

        let mut inner = self.inner.clone();
        let output = async_stream::try_stream! {
            while let Some(req) = stream.next().await {
                let req = req?;
                let plan = req.plan.ok_or_else(|| tonic::Status::invalid_argument("missing plan"))?;
                let plan: Plan = plan.try_into().unwrap();
                let target = plan.targets.first().unwrap().to_string();

                inner.call(plan).await.expect("service call failed");

                yield proto::MetronResponse { target };
            }
        };

        Ok(Response::new(Box::pin(output) as Self::RunStream))
    }
}

impl TryFrom<proto::Plan> for Plan {
    type Error = anyhow::Error;

    fn try_from(value: proto::Plan) -> Result<Self, Self::Error> {
        Ok(Self {
            segments: vec![],
            connections: 3,
            http_method: HttpMethod::Get,
            targets: vec![value.target.parse()?],
            headers: vec![],
            payload: None,
            worker_threads: 100,
            latency_correction: true,
        })
    }
}
