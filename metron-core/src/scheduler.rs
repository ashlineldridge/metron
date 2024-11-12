use std::{
    future::Future,
    hash::Hash,
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::Result;
use tower::{discover::Discover, ready_cache::ReadyCache, Service};
use tracing::debug;

#[allow(unused)]
pub struct Scheduler<D, Req>
where
    D: Discover,
    D::Key: Hash,
{
    discover: D,
    services: ReadyCache<D::Key, D::Service, Req>,
}

impl<D, Req> Scheduler<D, Req>
where
    D: Discover + Unpin,
    D::Key: Hash + Clone,
    D::Error: Into<tower::BoxError>,
    D::Service: Service<Req>,
    <D::Service as Service<Req>>::Error: Into<tower::BoxError>,
{
    pub fn new(discover: D) -> Self {
        Self {
            discover,
            services: ReadyCache::default(),
        }
    }

    fn discover_agents(&mut self, _cx: &mut Context<'_>) -> Poll<Option<Result<()>>> {
        debug!("discovering agents");
        todo!()
        //     loop {
        //         let ready = ready!(Pin::new(&mut self.discover).poll_discover(cx)).transpose()?;

        //         match ready {
        //             None => return Poll::Ready(None),
        //             Some(Change::Remove(key)) => {
        //                 debug!("removing agent");
        //                 self.services.evict(&key);
        //             }
        //             Some(Change::Insert(key, svc)) => {
        //                 debug!("inserting agent");
        //                 self.services.push(key, svc);
        //             }
        //         }
        //     }
    }
}

impl<D, Req> Service<Req> for Scheduler<D, Req>
where
    D: Discover + Unpin,
    D::Key: Hash + Clone,
    D::Error: Into<tower::BoxError>,
    D::Service: Service<Req>,
    <D::Service as Service<Req>>::Error: Into<tower::BoxError>,
{
    type Response = ();
    type Error = tower::BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let _ = self.discover_agents(cx)?;
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _request: Req) -> Self::Future {
        todo!()
        // Box::pin(async {
        //     tokio::time::sleep(Duration::from_millis(250)).await;
        //     Ok(())
        // })
    }
}
