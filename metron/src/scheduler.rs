use std::{
    future::Future,
    hash::Hash,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures_core::ready;
use thiserror::Error;
use tower::{
    discover::{Change, Discover},
    ready_cache::ReadyCache,
    Service, ServiceExt,
};
use tracing::{debug, trace};

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

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

    fn discover_agents(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<(), SchedulerError>>> {
        debug!("discovering agents");
        loop {
            let ready = ready!(Pin::new(&mut self.discover).poll_discover(cx))
                .transpose()
                .map_err(|e| {
                    SchedulerError::Unexpected(anyhow::anyhow!("boom: {:?}", e.into().to_string()))
                })?;

            match ready {
                None => return Poll::Ready(None),
                Some(Change::Remove(key)) => {
                    debug!("removing agent");
                    self.services.evict(&key);
                }
                Some(Change::Insert(key, svc)) => {
                    debug!("inserting agent");
                    self.services.push(key, svc);
                }
            }
        }
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

    fn call(&mut self, request: Req) -> Self::Future {
        Box::pin(async {
            tokio::time::sleep(Duration::from_millis(250)).await;
            Ok(())
        })
    }
}
