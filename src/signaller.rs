use anyhow::Error;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

// TODO:
// Maybe builder pattern is better here
// Look at Tokio Runtime Builder for example
// Runtime has Kind which can equal either single or multi-threaded

use crate::{error::SIGNALLER_MULTI_START_ERROR, schedule::Schedule};

const CHAN_SIZE: usize = 1024;

pub fn asap_signaller(duration: Option<Duration>) -> AsapSignaller {
    AsapSignaller::new(duration)
}

pub fn blocking_signaller<S: Schedule>(schedule: S) -> BlockingSignaller<S> {
    BlockingSignaller::new(schedule, CHAN_SIZE)
}

pub fn async_signaller<S: Schedule>(schedule: S) -> AsyncSignaller<S> {
    AsyncSignaller::new(schedule, CHAN_SIZE)
}

#[derive(Debug)]
pub struct Signal {
    pub due: Instant,
}

impl Signal {
    fn new(due: Instant) -> Self {
        Self { due }
    }

    fn now() -> Self {
        Self {
            due: Instant::now(),
        }
    }
}

pub trait Signaller: Send {
    fn start(&mut self) -> JoinHandle<Result<(), Error>> {
        tokio::task::spawn(std::future::ready(Ok(())))
    }

    fn recv(&mut self) -> SignalFuture;

    fn boxed<'a>(self) -> Box<dyn Signaller + 'a>
    where
        Self: Sized + 'a,
    {
        Box::new(self) as Box<dyn Signaller>
    }
}

pub struct SignalFuture<'a> {
    inner: Pin<Box<dyn Future<Output = Option<Signal>> + Send + 'a>>,
}

impl<'a> SignalFuture<'a> {
    fn new<F>(value: F) -> Self
    where
        F: Future<Output = Option<Signal>> + Send + 'a,
    {
        Self {
            inner: Box::pin(value),
        }
    }

    fn now() -> Self {
        Self::new(std::future::ready(Some(Signal::now())))
    }

    fn none() -> Self {
        Self::new(std::future::ready(None))
    }
}

impl<'a> Future for SignalFuture<'a> {
    type Output = Option<Signal>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.inner.as_mut().poll(cx)
    }
}

pub struct AsapSignaller {
    since: Option<Instant>,
    duration: Option<Duration>,
}

impl AsapSignaller {
    pub fn new(duration: Option<Duration>) -> Self {
        Self {
            since: None,
            duration,
        }
    }
}

impl Signaller for AsapSignaller {
    fn recv(&mut self) -> SignalFuture {
        let since = self.since.get_or_insert_with(Instant::now);
        if self
            .duration
            .map(|limit| since.elapsed() < limit)
            .unwrap_or(true)
        {
            SignalFuture::now()
        } else {
            SignalFuture::none()
        }
    }
}

pub struct BlockingSignaller<S: Schedule> {
    schedule: Option<S>,
    tx: Option<Sender<Signal>>,
    rx: Receiver<Signal>,
}

impl<S: Schedule> BlockingSignaller<S> {
    fn new(schedule: S, chan_size: usize) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(chan_size);
        Self {
            schedule: Some(schedule),
            tx: Some(tx),
            rx,
        }
    }
}

impl<S> Signaller for BlockingSignaller<S>
where
    S: Schedule + Send + 'static,
{
    fn start(&mut self) -> JoinHandle<Result<(), Error>> {
        let tx = self.tx.take().expect(SIGNALLER_MULTI_START_ERROR);
        let schedule = self.schedule.take().expect(SIGNALLER_MULTI_START_ERROR);

        tokio::task::spawn_blocking(move || {
            for t in schedule.iter_from(Instant::now()) {
                spin_until(t);
                tx.blocking_send(Signal::new(t))?;
            }

            Ok(())
        })
    }

    fn recv(&mut self) -> SignalFuture {
        SignalFuture::new(self.rx.recv())
    }
}

pub struct AsyncSignaller<S: Schedule> {
    schedule: Option<S>,
    tx: Sender<Signal>,
    rx: Receiver<Signal>,
}

impl<S: Schedule> AsyncSignaller<S> {
    fn new(schedule: S, chan_size: usize) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(chan_size);
        Self {
            schedule: Some(schedule),
            tx,
            rx,
        }
    }
}

impl<S> Signaller for AsyncSignaller<S>
where
    S: Schedule + Send + 'static,
{
    fn start(&mut self) -> JoinHandle<Result<(), Error>> {
        let tx = self.tx.clone();
        let schedule = self
            .schedule
            .take()
            .expect("`AsyncSignaller` can only be started once");

        tokio::task::spawn(async move {
            for t in schedule.iter_from(Instant::now()) {
                sleep_until(t).await;
                tx.send(Signal::new(t)).await?;
            }

            Ok(())
        })
    }

    fn recv(&mut self) -> SignalFuture {
        SignalFuture::new(self.rx.recv())
    }
}

fn spin_until(t: Instant) {
    loop {
        if Instant::now() >= t {
            break;
        }

        std::hint::spin_loop();
    }
}

async fn sleep_until(t: Instant) {
    tokio::time::sleep_until(t.into()).await
}
