use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        oneshot,
    },
    task::JoinHandle,
};
use url::Url;

use crate::{wait, Agent, Plan, Sink};

const CHAN_SIZE: usize = 1024;

#[derive(Clone, Debug)]
pub enum Signaller {
    /// A `Dedicated` signaller creates a dedicated thread for producing
    /// timing signals. This is the most accurate signaller for interval-
    /// based timing due to the fact that it does not need to cooperate with
    /// the scheduler.
    Dedicated,

    /// A `Cooperative` signaller uses a cooperatively scheduled Tokio task
    /// to produce timing signals. This type of signaller is useful in single-
    /// threaded environments or when you want to dedicate your threading
    /// resources elsewhere.
    Cooperative,
}

#[allow(unused)]
pub struct Runner {
    name: String,
    update_tx: Sender<Update>,
    sinks: Vec<Sink>,
}

impl Runner {
    pub fn run(name: String, sinks: Vec<Sink>, signaller: Signaller) -> Self {
        let (signal_tx, mut signal_rx) = mpsc::channel(CHAN_SIZE);
        let (update_tx, update_rx) = mpsc::channel(CHAN_SIZE);
        let (sample_tx, sample_rx) = mpsc::channel(CHAN_SIZE);

        let signaller_handle = match signaller {
            Signaller::Dedicated => Self::spawn_dedicated_signaller(signal_tx, update_rx),
            Signaller::Cooperative => Self::spawn_cooperative_signaller(signal_tx, update_rx),
        };

        // let executor_handle =

        Self {
            name,
            update_tx,
            sinks,
        }
    }

    // fn spawn_executor() -> JoinHandle<()> {}

    fn spawn_dedicated_signaller(
        signal_tx: Sender<Signal>,
        mut update_rx: Receiver<Update>,
    ) -> JoinHandle<()> {
        let (tx, rx) = tokio::sync::watch::channel::<Option<Update>>(None);
        // tx.send(Some(Update::new(Plan {})))

        tokio::task::spawn(async move {
            let mut spawned: Option<(JoinHandle<Result<()>>, oneshot::Sender<()>)> = None;
            while let Some(update) = update_rx.recv().await {
                if let Some((handle, stop_tx)) = spawned.take() {
                    // Tokio doesn't allow blocking tasks to be aborted in the usual way. Instead, we
                    // send a oneshot message to instruct the task to shut down and then wait on it.
                    stop_tx.send(());
                    handle.await;
                }

                let signal_tx = signal_tx.clone();
                let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel();

                let handle = tokio::task::spawn_blocking(move || {
                    for t in update.plan.ticks(update.start) {
                        wait::spin_until(t);
                        signal_tx.blocking_send(Signal::new(t))?;

                        // TODO: Probably don't want to do this every time through the loop.
                        match stop_rx.try_recv() {
                            Ok(()) => break,
                            Err(oneshot::error::TryRecvError::Empty) => continue,
                            Err(e) => return Err(e.into()),
                        }
                    }

                    Ok(())
                });

                spawned = Some((handle, stop_tx));
            }
        })
    }

    fn spawn_cooperative_signaller(
        signal_tx: Sender<Signal>,
        mut update_rx: Receiver<Update>,
    ) -> JoinHandle<()> {
        tokio::task::spawn(async move {
            let mut handle: Option<JoinHandle<Result<()>>> = None;
            while let Some(update) = update_rx.recv().await {
                if let Some(handle) = handle.take() {
                    handle.abort();
                }

                let signal_tx = signal_tx.clone();
                handle = Some(tokio::task::spawn(async move {
                    for t in update.plan.ticks(update.start) {
                        wait::sleep_until(t).await;
                        signal_tx.send(Signal::new(t)).await?;
                    }
                    Ok(())
                }));
            }

            if let Some(handle) = handle.take() {
                handle.abort();
            }
        })
    }
}

impl Agent for Runner {
    async fn test(&self, plan: &Plan) -> Result<(), crate::AgentError> {
        // TODO: This needs to be when the test started.
        let now = Instant::now();
        self.update_tx
            .send(Update::new(plan.clone(), now))
            .await
            .context("could not update runner")?;
        Ok(())
    }

    async fn cancel(&self) -> Result<(), crate::AgentError> {
        let now = Instant::now();
        self.update_tx
            .send(Update::new(Plan::empty(), now))
            .await
            .context("could not cancel runner")?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Signal {
    pub due: Instant,
}

impl Signal {
    fn new(due: Instant) -> Self {
        Self { due }
    }
}

#[derive(Clone, Debug)]
struct Update {
    plan: Plan,
    start: Instant,
}

impl Update {
    fn new(plan: Plan, start: Instant) -> Self {
        Self { plan, start }
    }
}

#[derive(Debug)]
pub struct Sample {
    pub target: Url,
    pub due: Instant,
    pub sent: Instant,
    pub done: Instant,
    pub status: Result<u16, anyhow::Error>,
}

impl Sample {
    pub fn actual_latency(&self) -> Duration {
        self.done - self.sent
    }

    pub fn corrected_latency(&self) -> Duration {
        self.done - self.due
    }

    // TODO: What is this?
    pub fn client_latency(&self) -> Duration {
        self.sent - self.due
    }
}
