//! Entry point for the main `metron` binary.

use anyhow::{bail, Result};
use grpc::server::GrpcServerAgent;
use metron::core::{
    agent::{Agent, SimpleSink},
    LoadTest,
};

pub enum Config {
    RunSpec(),
    AgentSpec(),
    ControlSpec(),
}

// Different ways to run this thing
// 1. metron run --rate 500 --duration 5m --target http://localhost:8080
//    - Run Metron as an all-in-one unit
// 2. metron agent --port 9090
//    - Run Metron as a gRPC server agent
// 3. metron run --rate 500 --duration 5m --target http://localhost:8080 --agent localhost:9090
//    - Run Metron as a local controller talking to a remote agent
//    - Multiple agents can be specified
//    - Also supports service discovery of agents (like Prom)
//    - Advanced config may require config file
// 4. metron control --port 9191 --agent localhost:9090
//    - Run Metron as a gRPC server controller
//    - Multiple agents can be specified
//    - Also supports service discovery of agents (like Prom)
//    - Advanced config may require config file
// 5. metron run --rate 500 --duration 5m --target http://localhost:8080 --agent localhost:9191
//    - Run Metron as a local controller talking to a remote controller
//    - From the client's perspective there is no difference between a remote agent and a remote controller
// 6. metron provision --platform fargate --cluster-name foo --num-agents 100 --provision-controller true
//    - Provision a Fargate pool of 100 agents managed by a controller (controller is given service discovery configuration to find the agents)
//    - Alternatively, can be provisioned without a controller
//    - In either case, `metron run` can be called and told about the agents or the controller
// 7. metron destroy --platform fargate --cluster-name foo
//    - Destroy a Fargate cluster
// 8. metrond
//    - Run Metron as a Kubernetes operator
#[tokio::main]
async fn main() -> Result<()> {
    let run_method = "grpc-server";
    match run_method {
        "standalone" => {
            // Standalone mode would require a load test to be supplied.
            let test = LoadTest {};
            run_standalone_agent(&test).await?
        }
        "grpc-server" => run_grpc_server_agent().await?,
        _ => bail!("unknown run method"),
    };

    Ok(())
}

async fn run_standalone_agent(test: &LoadTest) -> Result<()> {
    let sink = SimpleSink::new();
    // TODO: Avoid mut?
    let mut agent = Agent::new(sink);

    agent.run(test).await?;

    Ok(())
}

async fn run_grpc_server_agent() -> Result<()> {
    let sink = SimpleSink::new();
    let agent = Agent::new(sink);
    let agent = GrpcServerAgent::new(agent, 8181);
    // agent.run();

    todo!()
}
