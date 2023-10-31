mod proto {
    tonic::include_proto!("proto");
}

use metron::core;
use proto::{
    agent_server, AgentStateRequest, AgentStateResponse, LoadTestRequest, LoadTestResponse,
};
use tonic::{Request, Response, Status};

pub struct GrpcServerAgent<S> {
    inner: S,
    port: u16,
}

impl<S> GrpcServerAgent<S> {
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
impl<S> agent_server::Agent for GrpcServerAgent<S>
where
    S: tower::Service<core::TestPlan> + Send + Sync + 'static,
{
    async fn run_load_test(
        &self,
        request: Request<LoadTestRequest>,
    ) -> Result<Response<LoadTestResponse>, Status> {
        // TODO: Convert LoadTestRequest into a core::LoadTest.
        // Can we define an Into?
        // let load_test = core::LoadTest {};
        // TODO: Look
        // self.inner.call(load_test);

        let res = LoadTestResponse {};
        Ok(Response::new(res))
    }

    async fn get_agent_state(
        &self,
        request: Request<AgentStateRequest>,
    ) -> Result<Response<AgentStateResponse>, Status> {
        todo!()
    }
}

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
