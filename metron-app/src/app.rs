use anyhow::{Context, Result};
use metron_config::*;
use metron_core::{Agent, Proxy, Runner};
use metron_grpc::{AgentClient, AgentServer};
use quanta::Clock;
use tower::discover::ServiceList;
use tracing::info;

const DEFAULT_AGENT_PORT: u16 = 9090;

pub async fn run_local_test(config: LocalTestConfig) -> Result<()> {
    info!("running local test");

    let runner = Runner::new("local".to_owned(), Clock::new(), config.sinks);
    runner.run_wait(config.plan).await?;
    // TODO: Still need to sleep here... What can I "hang" on?
    // tokio::time::sleep(std::time::Duration::from_secs(60)).await;

    Ok(())
}

pub async fn run_remote_test(config: RemoteTestConfig) -> Result<()> {
    info!("running remote test");

    let client = single_agent_client(&config.agents).await?;
    client.exec(config.plan).await?;

    Ok(())
}

pub async fn run_agent_server(config: AgentConfig) -> Result<()> {
    info!("running agent server");

    // Because here, the server doesn't need to wait, right?
    // Can I use the report "stream"? Perhaps the exec function is async non-blocking
    // but the runner exposes a wait function or is a Future for the local use case
    // which is the only use case, right? Could still marry up with the report stream
    // potentially.
    let runner = Runner::new("local".to_owned(), Clock::new(), config.sinks);
    let port = config.port.unwrap_or(DEFAULT_AGENT_PORT);
    let server = AgentServer::new(runner, port);
    server.run().await?;

    Ok(())
}

pub async fn run_proxy_server(config: ProxyConfig) -> Result<()> {
    info!("running proxy server");

    let discover = agent_discover(&config.agents).await?;
    let proxy = Proxy::new("proxy".to_owned(), discover);
    let port = config.port.unwrap_or(DEFAULT_AGENT_PORT);
    let server = AgentServer::new(proxy, port);
    server.run().await?;

    Ok(())
}

pub async fn stop_remote_test(config: StopConfig) -> Result<()> {
    info!("stopping any running test");

    let client = single_agent_client(&config.agents).await?;
    client.stop().await?;

    Ok(())
}

pub async fn report_remote_test(_config: ReportConfig) -> Result<()> {
    info!("requesting load test report");

    // let discover = agent_discover(&config.agents).await?;
    // let proxy = Proxy::new("local".to_owned(), discover);
    // let report = proxy.report().await?;
    // info!("received load test report: {:?}", report);

    Ok(())
}

#[allow(unused)]
async fn agent_discover(agents: &[RemoteAgentDiscovery]) -> Result<ServiceList<Vec<AgentClient>>> {
    let addrs = agents
        .iter()
        .filter_map(|d| {
            if let RemoteAgentDiscovery::Static(d) = d {
                return Some(d.endpoints.clone());
            }
            None
        })
        .flatten()
        .collect::<Vec<_>>();

    let mut agents = Vec::with_capacity(addrs.len());
    for addr in addrs {
        let agent = AgentClient::connect(addr).await?;
        agents.push(agent);
    }

    Ok(ServiceList::new(agents))
}

#[allow(unused)]
async fn single_agent_client(agents: &[RemoteAgentDiscovery]) -> Result<AgentClient> {
    let addr = agents
        .first()
        .and_then(|d| {
            if let RemoteAgentDiscovery::Static(d) = d {
                d.endpoints.first()
            } else {
                None
            }
        })
        .context("agent server address could not be discovered")?;

    AgentClient::connect(addr.clone()).await
}
