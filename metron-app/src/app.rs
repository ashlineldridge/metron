use anyhow::{Context, Result};
use metron_config::*;
use metron_core::{Agent, AgentRequest, Proxy, Runner};
use metron_grpc::{AgentClient, AgentServer};
use quanta::Instant;
use tower::discover::ServiceList;
use tracing::info;

const DEFAULT_AGENT_PORT: u16 = 9090;

pub async fn run_local_test(config: LocalTestConfig) -> Result<()> {
    info!("running local test");

    let runner = Runner::run("local".to_owned(), config.sinks);
    runner
        .execute(AgentRequest {
            plan: config.plan,
            start: Instant::now(),
        })
        .await?;

    Ok(())
}

pub async fn run_remote_test(config: RemoteTestConfig) -> Result<()> {
    info!("running remote test");

    let agent_client = single_agent_client(&config.agents).await?;
    agent_client
        .execute(AgentRequest {
            plan: config.plan,
            start: Instant::now(),
        })
        .await?;

    Ok(())
}

pub async fn run_agent_server(config: AgentConfig) -> Result<()> {
    info!("running agent server");

    let runner = Runner::run("agent".to_owned(), config.sinks);
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

pub async fn cancel_remote_test(_config: CancelConfig) -> Result<()> {
    info!("cancelling any running test");

    // let discover = agent_discover(&config.agents).await?;
    // let proxy = Proxy::new("local".to_owned(), discover);
    // proxy
    //     .execute(AgentRequest {
    //         plan: Plan::empty(),
    //         start: Instant::now(),
    //     })
    //     .await?;

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
