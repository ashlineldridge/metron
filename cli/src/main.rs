//! Entry point for the main `metron` binary.

use std::env;

use anyhow::Result;
use cli::Spec;

#[tokio::main]
async fn main() -> Result<()> {
    let spec = cli::parse(env::args_os())?;
    match spec {
        Spec::Agent(config) => println!("agent: {:?}", config),
        Spec::Controller(config) => println!("controller: {:?}", config),
        Spec::Run(config) => println!("run: {:?}", config),
        Spec::Help(message) => println!("{message}"),
    }

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
