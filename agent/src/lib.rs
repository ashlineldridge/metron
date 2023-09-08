use anyhow::Result;
use async_trait::async_trait;
use proto::metron::{agent_client::AgentClient, agent_server::Agent, TestPlan, TestReport};
use tonic::transport::Channel;

// TODO:
// Make two simple binaries that exercise the interface below
// Then adjust the types to take advantage of Service trait and other Tonic stuff
// See: https://github.com/hyperium/tonic/tree/master/examples/src

#[async_trait]
pub trait Client {
    async fn run(&mut self, plan: TestPlan) -> Result<TestReport>;
}

pub struct GrpcClient(AgentClient<Channel>);

impl GrpcClient {
    pub async fn connect(server_address: &str) -> Result<Self> {
        let client = AgentClient::connect(server_address.to_owned()).await?;
        Ok(Self(client))
    }
}

#[async_trait]
impl Client for GrpcClient {
    async fn run(&mut self, plan: TestPlan) -> Result<TestReport> {
        let request = tonic::Request::new(plan);
        let response = self.0.run(request).await?;
        let report = response.into_inner();

        Ok(report)
    }
}

#[async_trait]
pub trait Server {
    async fn run(&self) -> Result<()>;
}

#[derive(Clone)]
pub struct GrpcServer {
    port: u16,
}

impl GrpcServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }
}

#[async_trait]
impl Agent for GrpcServer {
    async fn run(
        &self,
        request: tonic::Request<TestPlan>,
    ) -> std::result::Result<tonic::Response<TestReport>, tonic::Status> {
        let plan = request.into_inner();
        println!("Got plan: {:?}", plan);

        Ok(tonic::Response::new(TestReport {
            target: plan.target,
            total_requests: 100,
            total_duration: None,
            response_latency: vec![],
            error_latency: vec![],
            request_delay: vec![],
        }))
    }
}

#[async_trait]
impl Server for GrpcServer {
    async fn run(&self) -> Result<()> {
        let address = format!("[::1]:{}", self.port).parse()?;
        let service = proto::metron::agent_server::AgentServer::new(self.clone());

        tonic::transport::Server::builder()
            .add_service(service)
            .serve(address)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {}
}
