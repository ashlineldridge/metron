use anyhow::Result;
use tower::discover::Discover;

use crate::{Agent, Plan, Report, ReportKind};

#[allow(unused)]
#[derive(Clone)]
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

    pub async fn run(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<D> Agent for Proxy<D>
where
    D: Discover + Send + Sync + 'static,
    D::Service: Agent + Send + Sync + 'static,
{
    async fn exec(&self, _plan: Plan) -> Result<()> {
        todo!()
    }

    async fn report(&self, _kind: ReportKind) -> Result<Report> {
        todo!()
    }

    async fn stop(&self) -> Result<()> {
        todo!()
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
