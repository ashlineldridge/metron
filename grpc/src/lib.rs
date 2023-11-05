mod proto {
    tonic::include_proto!("proto");
}

use std::{
    future::Future,
    net::AddrParseError,
    pin::Pin,
    task::Poll,
    time::{Duration, Instant},
};

use metron::core::Plan;
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
pub struct MetronServer<S> {
    inner: S,
    port: u16,
}

impl<S> MetronServer<S>
where
    S: Service<Plan> + Send + Sync + 'static,
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
    S: Service<Plan> + Send + Sync + 'static,
{
    type RunStream =
        Pin<Box<dyn Stream<Item = Result<proto::MetronResponse, tonic::Status>> + Send + 'static>>;

    async fn run(
        &self,
        request: Request<Streaming<proto::MetronRequest>>,
    ) -> Result<Response<Self::RunStream>, tonic::Status> {
        let mut stream = request.into_inner();

        let output = async_stream::try_stream! {
            while let Some(req) = stream.next().await {
                // Just bounce responses back for now.
                let req = req?;
                let plan = req.plan.ok_or_else(|| tonic::Status::invalid_argument("missing plan"))?;
                let num_segments = plan.segments.len() as u64;
                yield proto::MetronResponse { num_segments };
            }
        };

        Ok(Response::new(Box::pin(output) as Self::RunStream))
    }
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
        let outbound = async_stream::stream! {
            let mut interval = tokio::time::interval(Duration::from_secs(1));

            let start = Instant::now();
            loop {
                let time = interval.tick().await;
                let elapsed = time.duration_since(start.into());
                let request = proto::MetronRequest {
                    plan: Some(proto::Plan {
                        segments: vec![],
                        target: format!("http://foo-{}", elapsed.as_secs()).to_owned(),
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
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

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

// TODO: Make AgentClient a Service and feed it into the Controller to make
// the Controller be able to talk to remote agents.

// TODO: GrpcServerAgent is also a Service

// TODO: GrpcServerAgent potentially doesn't need to be a Service.
// Have a look at Tonic as it will have preferences around  middlewear
// already. Important thing is that the GrpcServerAgent wraps a
// Service.
/// `Service` implementation that runs a gRPC server.

// TODO:
// Make two simple binaries that exercise the interface below
// Then adjust the types to take advantage of Service trait and other Tonic stuff
// See: https://github.com/hyperium/tonic/tree/master/examples/src

// #[async_trait]
// pub trait Client {
//     async fn run(&mut self, plan: TestPlan) -> Result<TestReport>;
// }

// pub struct GrpcClient(AgentClient<Channel>);

// impl GrpcClient {
//     pub async fn connect(server_address: &str) -> Result<Self> {
//         let client = AgentClient::connect(server_address.to_owned()).await?;
//         Ok(Self(client))
//     }
// }

// #[async_trait]
// impl Client for GrpcClient {
//     async fn run(&mut self, plan: TestPlan) -> Result<TestReport> {
//         let request = tonic::Request::new(plan);
//         let response = self.0.run(request).await?;
//         let report = response.into_inner();

//         Ok(report)
//     }
// }

// #[async_trait]
// pub trait Server {
//     async fn run(&self) -> Result<()>;
// }

// #[derive(Clone)]
// pub struct GrpcServer {
//     port: u16,
// }

// impl GrpcServer {
//     pub fn new(port: u16) -> Self {
//         Self { port }
//     }
// }

// #[async_trait]
// impl Agent for GrpcServer {
//     async fn run(
//         &self,
//         request: tonic::Request<TestPlan>,
//     ) -> std::result::Result<tonic::Response<TestReport>, tonic::Status> {
//         let plan = request.into_inner();
//         println!("Got plan: {:?}", plan);

//         Ok(tonic::Response::new(TestReport {
//             target: plan.target,
//             total_requests: 100,
//             total_duration: None,
//             response_latency: vec![],
//             error_latency: vec![],
//             request_delay: vec![],
//         }))
//     }
// }

// #[async_trait]
// impl Server for GrpcServer {
//     async fn run(&self) -> Result<()> {
//         let address = format!("[::1]:{}", self.port).parse()?;
//         let service = metron::agent_server::AgentServer::new(self.clone());

//         tonic::transport::Server::builder()
//             .add_service(service)
//             .serve(address)
//             .await?;

//         Ok(())
//     }
// }

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
