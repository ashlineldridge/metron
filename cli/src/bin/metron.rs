//! Entry point for the main `metron` binary.

use std::env;

use anyhow::Result;
use cli::ParsedCli;
use metron::RunConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // dump_config();
    let parsed_config = cli::parse(env::args_os())?;
    match parsed_config {
        ParsedCli::Run(config) => run(&config).await?,
        ParsedCli::Help(text) => println!("{text}"),
    }

    Ok(())
}

async fn run(_config: &RunConfig) -> Result<()> {
    // if let Some(runner) = &config.local_runner {}
    // let mut remote_runners = Vec::with_capacity(config.remote_runners.len());
    // for r in &config.remote_runners {
    //     match r {
    //         RunnerRef::Static { address } => todo!(),
    //         RunnerRef::Kubernetes {
    //             namespace,
    //             selector,
    //             port,
    //         } => todo!(),
    //     }
    //     // let runner = Runner::new(r.name.clone(), r.signaller, r.worker_threads);
    // }

    // let registry = RunnerRegistry::new(runners);
    // for r in &config.runner_discovery {
    //     match r.address.scheme() {
    //         "local" =>
    //     }
    // }

    // let target_runners = config.runner_discovery.iter().map(|r| match (r.remote, r.local) {
    //     (Some(remote), None) => todo!(),
    //     (None, Some(local)) => todo!(),
    //     _ => bail!("invalid runner discovery"),
    // })
    // let controller = Controller::new(target_runners);

    // let local_runners = config.runners.iter().map(|r| Runner::new(r.name.clone(), r.signaller, r.worker_threads)).collect();

    // if let Some(port) = config.port {
    // } else {
    // }

    // Ok(())
    todo!()
}

// async fn run_test(config: TestConfig) -> Result<()> {
//     if let Some(runner_discovery) = config.runners {
//         let runners = external_runners(&runner_discovery).await?;
//         let controller = Controller::new(runners);
//         controller.run(&config.plan).await?;
//     } else {
//         let controller = Controller::new(vec![Runner::new()]);
//         controller.run(&config.plan).await?;
//     }

//     Ok(())
// }

// async fn run_runner(config: RunnerConfig) -> Result<()> {
//     let port = config.port;
//     let runner = Runner::new();
//     let metron_server = MetronServer::new(runner, port);

//     metron_server.listen().await?;

//     Ok(())
// }

// // Runner addresses need to be of the form: http://[::1]:9090
// async fn run_controller(config: ControllerConfig) -> Result<()> {
//     let port = config.port;
//     let runners = external_runners(&config.runners).await?;
//     let controller = Controller::new(runners);
//     let metron_server = MetronServer::new(controller, port);

//     metron_server.listen().await?;

//     Ok(())
// }

// async fn external_runners(config: &RunnerDiscoveryConfig) -> Result<Vec<MetronClient>> {
//     let mut runners = Vec::with_capacity(config.static_runners.len());
//     for endpoint in &config.static_runners {
//         let runner = MetronClient::connect(endpoint.clone()).await?;
//         runners.push(runner);
//     }

//     Ok(runners)
// }

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
//     let plan = metron::Plan {
//         segments: vec![
//             metron::RateSegment::Fixed {
//                 rate: 100.0,
//                 duration: Some(std::time::Duration::from_secs(120)),
//             },
//             metron::RateSegment::Linear {
//                 rate_start: 100.0,
//                 rate_end: 200.0,
//                 duration: std::time::Duration::from_secs(60),
//             },
//         ],
//         actions: vec![metron::Action::Http {
//             method: metron::HttpMethod::Get,
//             headers: [("foo".to_owned(), "bar".to_owned())].into_iter().collect(),
//             payload: "foobar".to_owned(),
//             target: "https://foobar.com".try_into().unwrap(),
//         }],
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

// async fn do_your_thing(&self) -> anyhow::Result<()> {
//     match &self.action {
//         Action::HttpRequest {
//             method,
//             headers,
//             payload,
//         } => {
//             let target = self.targets.first().unwrap();
//             let url: hyper::Uri = target.try_into()?;
//             let host = url.host().context("target has no host")?;
//             let port = url.port_u16().unwrap_or(80);
//             let addr = format!("{}:{}", host, port);

//             let stream = tokio::net::TcpStream::connect(addr).await?;
//             let io = hyper_util::rt::TokioIo::new(stream);

//             let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
//             tokio::task::spawn(async move {
//                 if let Err(err) = conn.await {
//                     println!("Connection failed: {:?}", err);
//                 }
//             });

//             let authority = url.authority().unwrap().clone();

//             let mut req = hyper::Request::builder()
//                 .uri(url)
//                 .method(method.to_string().as_str())
//                 .header(hyper::header::HOST, authority.as_str());

//             for header in headers {
//                 req = req.header(&header.name, &header.value);
//             }

//             let req = req.body(payload.clone())?;

//             let _res = sender.send_request(req).await?;
//         }
//         Action::UdpDatagram { payload } => {
//             let client = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
//             let target = self.targets.first().unwrap();
//             client.connect(target).await?;
//             client.send(payload.as_bytes()).await?;
//         }
//     }

//     Ok(())
// }
