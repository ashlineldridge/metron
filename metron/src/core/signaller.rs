use std::time::Instant;

use anyhow::Result;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{
        mpsc,
        mpsc::{Receiver, Sender},
    },
    task,
    task::JoinHandle,
};

use crate::profile::plan::Plan;

const BACK_PRESSURE_CHAN_SIZE: usize = 1024;
const MULTIPLE_STARTS_ERROR: &str = "`Signaller` can only be started once";

/// Produces timing signals that indicate when the next request should be sent.
///
/// # Examples
/// ```
/// use crate::plan::Builder;
/// use crate::signaller::Signaller;
///
/// use std::time::Duration;
/// use metron_old::Rate;
///
/// #[tokio::main]
/// async fn main() {
///   // 100 RPS plan that runs for 60 seconds.
///   let plan = Builder::new()
///       .fixed_rate_block(Rate(100), Some(Duration::from_secs(60)))
///       .build()
///       .unwrap();
///
///     // Create a blocking signaller that uses a dedicated thread to place
///     // signals on a channel at the appropriate time.
///     let signaller = Signaller::new_blocking_thread(plan);
///     let sig = signaller.recv().await.unwrap();
///     println!("The next request should be sent at {}", sig.due);
/// }
/// ```
pub struct Signaller {
    /// Signaller kind.
    kind: Kind,
    /// Plan used to determine request timing.
    plan: Plan,
    /// Sender part of the back-pressure channel.
    tx: Option<Sender<Signal>>,
    /// Receiver part of the back-pressure channel.
    rx: Option<Receiver<Signal>>,
}

/// The kind of signaller.
///
/// The signaller kind dictates the concurrency model that the signaller uses
/// to produce timing signals.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// A `Blocking` signaller creates a dedicated thread for producing
    /// timing signals. This is the most accurate signaller for interval-
    /// based timing due to the fact that it does not need to cooperate with
    /// the scheduler.
    Blocking,

    /// A `Cooperative` signaller uses a cooperatively scheduled Tokio task
    /// to produce timing signals. This type of signaller is useful in single-
    /// threaded environments or when you want to dedicate your threading
    /// resources elsewhere.
    Cooperative,
}

impl Kind {
    pub fn is_blocking(&self) -> bool {
        *self == Self::Blocking
    }
}

impl Default for Kind {
    fn default() -> Self {
        Self::Blocking
    }
}

impl Signaller {
    /// Creates and runs a new `Signaller`.
    ///
    /// # Arguments
    ///
    /// * `kind` - Kind of `Signaller` to create
    /// * `plan` - Plan used to determine request timing
    pub fn start(kind: Kind, plan: Plan) -> Self {
        let mut signaller = Self::new(kind, plan);
        signaller.spawn();
        signaller
    }

    /// Creates a new `Signaller`.
    ///
    /// # Arguments
    ///
    /// * `kind` - Kind of `Signaller` to create
    /// * `plan` - Plan used to determine request timing
    fn new(kind: Kind, plan: Plan) -> Self {
        let (tx, rx) = mpsc::channel(BACK_PRESSURE_CHAN_SIZE);
        let (tx, rx) = (Some(tx), Some(rx));

        Self { kind, plan, tx, rx }
    }

    /// Spawns background process used to generate timing signals.
    ///
    /// This function returns a [JoinHandle] that may be used to interact with
    /// the background workload. The completion of this `JoinHandle` should only
    /// be taken to mean that any asynchronous work needed to produce the
    /// signals is complete and not that there are no more signals available to
    /// be read. To ensure all signals have been read, the client should
    /// continue to call [`recv`][Self::recv] until `None` is returned.
    fn spawn(&mut self) -> JoinHandle<Result<()>> {
        let tx = self.tx.take().expect(MULTIPLE_STARTS_ERROR);
        let plan = self.plan.clone();

        let start = Instant::now();

        match self.kind {
            Kind::Blocking => task::spawn_blocking(move || {
                for t in plan.ticks(start) {
                    crate::wait::spin_until(t);
                    tx.blocking_send(Signal::new(t))?;
                }

                Ok(())
            }),
            Kind::Cooperative => task::spawn(async move {
                for t in plan.ticks(start) {
                    crate::wait::sleep_until(t).await;
                    tx.send(Signal::new(t)).await?;
                }

                Ok(())
            }),
        }
    }

    /// Receive a waiting signal or wait until one is available.
    ///
    /// This function can be used to obtain the next available timing signal.
    /// The oldest available timing signal will be returned or the returned
    /// future will block until one is available. This function is intended
    /// to be used as the synchronization point that drives request timing.
    ///
    /// More to come...
    pub async fn recv(&mut self) -> Option<Signal> {
        // Safe to unwrap since we control the lifecycle of rx.
        let rx = self.rx.as_mut().unwrap();
        rx.recv().await
    }
}

#[derive(Debug)]
pub struct Signal {
    pub due: Instant,
}

impl Signal {
    fn new(due: Instant) -> Self {
        Self { due }
    }
}
