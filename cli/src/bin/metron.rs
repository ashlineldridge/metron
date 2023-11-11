//! Entry point for the main `metron` binary.

use std::env;

use anyhow::Result;
use cli::ParsedCli;
use grpc::{MetronClient, MetronServer};
use metron::{Controller, ControllerConfig, LoadTestConfig, Runner, RunnerConfig, RunnerDiscovery};

#[tokio::main]
async fn main() -> Result<()> {
    let parsed_config = cli::parse(env::args_os())?;
    match parsed_config {
        ParsedCli::LoadTest(config) => run_test(config).await?,
        ParsedCli::Runner(config) => run_runner(config).await?,
        ParsedCli::Controller(config) => run_controller(config).await?,
        ParsedCli::Help(message) => println!("{message}"),
    }

    Ok(())
}

async fn run_test(config: LoadTestConfig) -> Result<()> {
    if let Some(runner_discovery) = config.external_runners {
        let runners = external_runners(&runner_discovery).await?;
        let controller = Controller::new(runners);
        controller.run(&config.plan).await?;
    } else {
        let controller = Controller::new(vec![Runner::new()]);
        controller.run(&config.plan).await?;
    }

    Ok(())
}

async fn run_runner(config: RunnerConfig) -> Result<()> {
    let port = config.server_port;
    let runner = Runner::new();
    let metron_server = MetronServer::new(runner, port);

    metron_server.listen().await?;

    Ok(())
}

// Runner addresses need to be of the form: http://[::1]:9090
async fn run_controller(config: ControllerConfig) -> Result<()> {
    let port = config.server_port;
    let runners = external_runners(&config.external_runners).await?;
    let controller = Controller::new(runners);
    let metron_server = MetronServer::new(controller, port);

    metron_server.listen().await?;

    Ok(())
}

async fn external_runners(config: &RunnerDiscovery) -> Result<Vec<MetronClient>> {
    let mut runners = Vec::with_capacity(config.static_runners.len());
    for endpoint in &config.static_runners {
        let runner = MetronClient::connect(endpoint.clone().into()).await?;
        runners.push(runner);
    }

    Ok(runners)
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

// fn dump_config() {
//     let plan = metron::core::Plan {
//         segments: vec![
//             metron::core::PlanSegment::Fixed {
//                 rate: metron::core::Rate::per_second(100),
//                 duration: Some(std::time::Duration::from_secs(120)),
//             },
//             metron::core::PlanSegment::Linear {
//                 rate_start: metron::core::Rate::per_second(100),
//                 rate_end: metron::core::Rate::per_second(200),
//                 duration: std::time::Duration::from_secs(60),
//             },
//         ],
//         connections: 8,
//         http_method: metron::core::HttpMethod::Get,
//         targets: vec![
//             "https://foo.com".parse().unwrap(),
//             "https://bar.com".parse().unwrap(),
//         ],
//         headers: vec![
//             metron::core::Header {
//                 name: "X-Metron-Foo".to_owned(),
//                 value: "foo".to_owned(),
//             },
//             metron::core::Header {
//                 name: "X-Metron-Bar".to_owned(),
//                 value: "bar".to_owned(),
//             },
//         ],
//         payload: Some("foobar".to_owned()),
//         worker_threads: 8,
//         latency_correction: true,
//     };

//     let plan_text = serde_yaml::to_string(&plan).unwrap();
//     println!("{}", plan_text);
// }

// fn test_logging() {
//     tracing_subscriber::fmt()
//         .with_max_level(tracing::Level::TRACE)
//         .init();

//     error!("ayo, we got an error here");
// }
