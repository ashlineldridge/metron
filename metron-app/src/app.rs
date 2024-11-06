use anyhow::Result;
use metron_config::*;
use metron_core::{Agent, Proxy, Runner};
use metron_grpc::{AgentClient, AgentServer};
use tower::discover::ServiceList;
use tracing::info;

const DEFAULT_AGENT_PORT: u16 = 9090;

pub async fn run_local_test(config: LocalTestConfig) -> Result<()> {
    info!("running local test");

    let runner = Runner::run_dedicated("local".to_owned(), config.sinks);
    runner.test(&config.plan).await?;

    Ok(())
}

pub async fn run_remote_test(config: RemoteTestConfig) -> Result<()> {
    info!("running remote test");

    let discover = agent_discover(&config.agents).await?;
    let proxy = Proxy::new("local".to_owned(), discover);
    proxy.test(&config.plan).await?;

    Ok(())
}

pub async fn run_agent_server(config: AgentConfig) -> Result<()> {
    info!("running agent server");

    let runner = Runner::run_dedicated("agent".to_owned(), config.sinks);
    let port = config.port.unwrap_or(DEFAULT_AGENT_PORT);
    let server = AgentServer::new(runner, port);
    server.run().await?;

    Ok(())
}

pub async fn run_proxy_server(config: ProxyConfig) -> Result<()> {
    info!("running proxy server");

    let discover = agent_discover(&config.agents).await?;
    let proxy = Proxy::new(config.name.unwrap_or("proxy-todo".to_owned()), discover);
    let port = config.port.unwrap_or(DEFAULT_AGENT_PORT);
    let _server = AgentServer::new(proxy, port);

    // server.listen().await?;

    Ok(())
}

pub async fn cancel_remote_test(config: CancelConfig) -> Result<()> {
    info!("cancelling any running test");

    let discover = agent_discover(&config.agents).await?;
    let proxy = Proxy::new("local".to_owned(), discover);
    proxy.cancel().await?;

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
