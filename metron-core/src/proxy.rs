use tower::discover::Discover;

use crate::{Agent, AgentError, Plan, Report};

#[derive(Clone)]
#[allow(unused)]
pub struct Proxy<D> {
    name: String,
    discover: D,
}

impl<D> Proxy<D>
where
    D: Discover,
    D::Service: Agent,
    //     <D::Service as Service<Plan>>::Response: Send + Sync + 'static,
    //     <D::Service as Service<Plan>>::Error: std::error::Error + Send + Sync + 'static,
    //     <D::Service as Service<Plan>>::Future: Send + 'static,
{
    pub fn new(name: String, discover: D) -> Self {
        Self { name, discover }
    }

    pub async fn run(&mut self) -> Result<(), AgentError> {
        // self.discover.pol
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
        //         .map_err(|e| AgentError::Unexpected(e.into()))?;
        // }

        // Ok(())
    }
}

impl<D: Send + Sync> Agent for Proxy<D> {
    async fn test(&self, _plan: &Plan) -> Result<(), AgentError> {
        Ok(())
    }

    async fn cancel(&self) -> Result<(), AgentError> {
        Ok(())
    }
}

// For now, the Controller just gives the same plan to all runners.
// impl<S> Service<Plan> for Proxy<S>
// where
//     S: Service<Plan> + Clone + Send + Sync + 'static,
//     S::Response: Send + Sync + 'static,
//     S::Error: std::error::Error + Send + Sync + 'static,
//     S::Future: Send + 'static,
// {
//     type Response = ();
//     type Error = AgentError;
//     type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

//     fn poll_ready(
//         &mut self,
//         _cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
//         // let mut dead = 0;
//         // for s in &mut self.agents {
//         //     match s.poll_ready(cx) {
//         //         Poll::Ready(Ok(_)) => return Poll::Ready(Ok(())),
//         //         Poll::Ready(Err(_)) => dead += 1,
//         //         _ => continue,
//         //     }
//         // }

//         // if dead == self.agents.len() {
//         //     return Poll::Ready(Err(anyhow!("all agents have terminally failed").into()));
//         // }

//         Poll::Pending
//     }

//     fn call(&mut self, req: Plan) -> Self::Future {
//         let agent = self.clone();
//         Box::pin(async move { agent.run(&req).await })
//     }
// }
