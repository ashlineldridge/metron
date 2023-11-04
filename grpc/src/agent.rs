mod proto {
    tonic::include_proto!("proto");
}

use std::{
    future::Future,
    pin::Pin,
    task::Poll,
    time::{Duration, Instant},
};

use proto::{agent_server, AgentRequest, AgentResponse, Plan};
use thiserror::Error;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Streaming};
use tower::Service;

use self::proto::agent_client;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    TransportError(#[from] tonic::transport::Error),

    #[error(transparent)]
    StatusError(#[from] tonic::Status),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

pub struct AgentServer<S> {
    inner: S,
    port: u16,
}

impl<S> AgentServer<S> {
    pub fn new(inner: S, port: u16) -> Self {
        Self { inner, port }
    }

    // pub async fn run(self) -> anyhow::Result<()> {
    //     // Should I be passing self or should I create an instance of the thing
    //     // that implements agent_server::Agent?

    //     let address = format!("[::1]:{}", self.port).parse()?;
    //     tonic::transport::Server::builder()
    //         .add_service(self)
    //         .serve(address)
    //         .await?;

    //     Ok(())
    // }
}

#[tonic::async_trait]
impl<S> agent_server::Agent for AgentServer<S>
where
    S: Service<Plan> + Send + Sync + 'static,
{
    type RunStream =
        Pin<Box<dyn Stream<Item = Result<AgentResponse, tonic::Status>> + Send + 'static>>;

    async fn run(
        &self,
        request: Request<Streaming<AgentRequest>>,
    ) -> Result<Response<Self::RunStream>, tonic::Status> {
        let mut stream = request.into_inner();

        let output = async_stream::try_stream! {
            while let Some(req) = stream.next().await {
                // Just bounce responses back for now.
                let req = req?;
                let plan = req.plan.ok_or_else(|| tonic::Status::invalid_argument("missing plan"))?;
                let num_segments = plan.segments.len() as u64;
                yield AgentResponse { num_segments };
            }
        };

        Ok(Response::new(Box::pin(output) as Self::RunStream))
    }
}

#[derive(Clone)]
pub struct AgentClient {
    inner: agent_client::AgentClient<tonic::transport::Channel>,
}

impl AgentClient {
    // TODO: This module prob needs its own error type?
    pub async fn connect(server_addr: String) -> Result<Self, Error> {
        let inner = agent_client::AgentClient::connect(server_addr).await?;

        Ok(Self { inner })
    }
}

impl AgentClient {
    async fn run(&mut self, plan: &Plan) -> Result<(), Error> {
        let outbound = async_stream::stream! {
            let mut interval = tokio::time::interval(Duration::from_secs(1));

            let start = Instant::now();
            loop {
                let time = interval.tick().await;
                let elapsed = time.duration_since(start.into());
                let request = AgentRequest {
                    plan: Some(Plan {
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
            println!("GOT AGENT RESPONSE = {:?}", res);
        }

        Ok(())
    }
}

impl Service<Plan> for AgentClient {
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
        let mut agent = self.clone();
        Box::pin(async move { agent.run(&req).await })
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
// impl<R, S> tower::Service<R> for GrpcServerAgent<S>
// where
//     S: tower::Service<R> + Clone + 'static,
//     R: 'static,
// {
//     type Response = S::Response;
//     type Error = S::Error;
//     type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

//     fn poll_ready(
//         &mut self,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
//         self.inner.poll_ready(cx)
//     }

//     fn call(&mut self, req: R) -> Self::Future {
//         // See https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services.
//         let clone = self.inner.clone();
//         let mut inner = std::mem::replace(&mut self.inner, clone);
//         Box::pin(async move { inner.call(req).await })
//     }
// }

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
