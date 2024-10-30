use std::{future::Future, hash::Hash, pin::Pin, task::Poll};

use anyhow::anyhow;
use tower::{balance::p2c::Balance, discover::Discover, Service};

use crate::Plan;

#[derive(Clone)]
pub struct Proxy<D> {
    discover: D,
}

impl<D> Proxy<D>
// where
//     D: Discover + Clone,
//     D::Key: Hash,
//     D::Service: Service<Plan> + Clone + Send + Sync + 'static,
//     <D::Service as Service<Plan>>::Response: Send + Sync + 'static,
//     <D::Service as Service<Plan>>::Error: std::error::Error + Send + Sync + 'static,
//     <D::Service as Service<Plan>>::Future: Send + 'static,
{
    pub fn new(discover: D) -> Self {
        Self { discover }
    }

    pub async fn run(&self, plan: &Plan) -> Result<(), crate::AgentError> {
        // let mut balancer = Balance::new(self.discover.clone());

        // let requests = (1..10)
        //     .map(|id| Request::new(format!("request-{id}")))
        //     .collect::<Vec<_>>();
        // let requests = futures::stream::iter(requests);

        // info!("calling load balancing");

        // let mut result = balancer.ready().await.expect("oh no").call_all(requests);

        // while let Some(resp) = result.next().await {
        //     let resp = resp.expect("oh no");
        //     info!("got response {}", resp.value);
        // }

        // Ok(())
        Ok(())

        // // TODO: This needs to load balance over the agents.
        // for agent in &self.agents {
        //     agent
        //         .clone()
        //         .call(plan.clone())
        //         .await
        //         .map_err(|e| crate::AgentError::Unexpected(e.into()))?;
        // }

        // Ok(())
    }
}

// For now, the Controller just gives the same plan to all runners.
impl<S> Service<Plan> for Proxy<S>
where
    S: Service<Plan> + Clone + Send + Sync + 'static,
    S::Response: Send + Sync + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
    S::Future: Send + 'static,
{
    type Response = ();
    type Error = crate::AgentError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        // let mut dead = 0;
        // for s in &mut self.agents {
        //     match s.poll_ready(cx) {
        //         Poll::Ready(Ok(_)) => return Poll::Ready(Ok(())),
        //         Poll::Ready(Err(_)) => dead += 1,
        //         _ => continue,
        //     }
        // }

        // if dead == self.agents.len() {
        //     return Poll::Ready(Err(anyhow!("all agents have terminally failed").into()));
        // }

        Poll::Pending
    }

    fn call(&mut self, req: Plan) -> Self::Future {
        let agent = self.clone();
        Box::pin(async move { agent.run(&req).await })
    }
}
