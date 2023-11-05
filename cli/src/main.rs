//! Entry point for the main `metron` binary.

use std::env;

use anyhow::Result;
use cli::Spec;
use grpc::{MetronClient, MetronServer};
use metron::core::{Agent, AgentConfig, Controller, ControllerConfig, Runner, RunnerConfig};

#[tokio::main]
async fn main() -> Result<()> {
    let spec = cli::parse(env::args_os())?;
    match spec {
        Spec::Run(config) => run_runner(config).await?,
        Spec::Agent(config) => run_agent(config).await?,
        Spec::Controller(config) => run_controller(config).await?,
        Spec::Help(message) => println!("{message}"),
    }

    Ok(())
}

async fn run_runner(config: RunnerConfig) -> Result<()> {
    // TODO: Need to grab the agents/agent-discovery from somewhere.
    // Perhaps rather than giving the Controller a list of agents,
    // I give it a mechanism to obtain the agents. In the simple case,
    // it's a static list. But it also provides a means for agent discovery.

    // Need a conditional on whether to create a MetronClient that is given
    // the address of a remote agent or a local Agent (both implement Service).
    let agent_addr = "http://[::1]:9090".to_owned();
    let metron_client = MetronClient::connect(agent_addr).await?;
    let agents = vec![metron_client];
    let controller_config = ControllerConfig::default();
    let controller = Controller::new(controller_config, agents);

    // let agent_config = AgentConfig::default();
    // let agents = vec![Agent::new(agent_config, Runner::new(config.clone()))];
    // let controller_config = ControllerConfig::default();
    // let controller = Controller::new(controller_config, agents);

    controller.run(&config.plan).await?;

    Ok(())
}

// TODO: See, here the AgentConfig is being given to Agent but port is actually
// required by AgentServer...
async fn run_agent(config: AgentConfig) -> Result<()> {
    let runner_config = RunnerConfig::default();
    let agent = Agent::new(config.clone(), Runner::new(runner_config));
    let metron_server = MetronServer::new(agent, config.port);

    metron_server.listen().await?;

    Ok(())
}

async fn run_controller(config: ControllerConfig) -> Result<()> {
    let agent_addr = "http://[::1]:9090".to_owned();
    let metron_client = MetronClient::connect(agent_addr).await?;
    let agents = vec![metron_client];
    let controller = Controller::new(config, agents);
    let metron_server = MetronServer::new(controller, 9191);

    metron_server.listen().await?;

    Ok(())
}

// How CLI influences the composition of Metron components:
//
// 1. metron run --rate 500 --duration 5m --target http://localhost:8080
//    - Run Metron as an all-in-one unit
//    - Entry point will build a Controller that controls an Agent that drives a Runner
//    - Entry point will build a Plan and tell the Controller to run it
//    - What about "runtime" config (e.g. thread settings, connections, etc)?
//
// 2. metron agent --port 9090
//    - Run Metron as a gRPC server agent
//    - Entry point will build an AgentServer that wraps an Agent that drives a Runner
//    - AgentServer will wait for instructions on port 9090
//
// 3. metron run --rate 500 --duration 5m --target http://localhost:8080 --agent localhost:9090
//    - Run Metron as a local controller talking to a remote agent
//    - Multiple agents can be specified
//    - Also supports service discovery of agents (like Prom)
//    - Entry point will build a Controller that controls an AgentClient configured to talk to localhost:9090
//    - Entry point will build a Plan and tell the Controller to run it
//    - What about "runtime" config (e.g. thread settings, connections, etc)?
//
// 4. metron controller --port 9191 --agent localhost:9090
//    - Run Metron as a gRPC server controller
//    - Multiple agents can be specified
//    - Also supports service discovery of agents (like Prom)
//    - Entry point will build an *AgentServer* that wraps a Controller that drives an AgentClient configured to talk to localhost:9090
//    - What about "runtime" config (e.g. thread settings, connections, etc)?
//
// 5. metron run --rate 500 --duration 5m --target http://localhost:8080 --agent localhost:9191
//    - Run Metron as a local controller talking to a remote controller (see previous command running controller on 9191)
//    - From the client's perspective there is no difference between a remote agent and a remote controller
//    - Entry point will build a Controller that controls an AgentClient configured to talk to localhost:9191
//    - Entry point will build a Plan and tell the Controller to run it
//    - What about "runtime" config (e.g. thread settings, connections, etc)?
