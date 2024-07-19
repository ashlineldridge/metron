use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::Stream;
use pin_project::pin_project;
use tower::{discover::Change, Service};
use url::Url;

use crate::{Plan, RunnerError};

#[pin_project]
pub struct RunnerRegistry<S> {
    registry: Vec<(Url, S)>,
}

impl<S> RunnerRegistry<S> {
    pub fn new(registry: Vec<(Url, S)>) -> Self {
        Self { registry }
    }

    pub fn register(&mut self, address: Url, s: S) {
        self.registry.push((address, s));
    }
}

impl<S> Stream for RunnerRegistry<S>
where
    S: Service<Plan, Response = (), Error = RunnerError>,
{
    type Item = Result<Change<Url, S>, RunnerError>;

    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project().registry.pop() {
            Some((url, s)) => Poll::Ready(Some(Ok(Change::Insert(url, s)))),
            None => {
                // There may be more later.
                Poll::Pending
            }
        }
    }
}
