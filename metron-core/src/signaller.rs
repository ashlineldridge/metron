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
use tracing::info;

use crate::Plan;

// TODO(REALLY_NEXT): Move this code into runner.rs. Get everything
// (i.e. basic action (e.g. HTTP call) running) working and encapsulated
// nicely within the Runner type. Then consider splitting things out
// into different components/modules.

const CHAN_SIZE: usize = 1024;

/// Produces timing signals that indicate when the next request should be sent.
///
/// # Examples
/// ```
/// use crate::plan::Builder;
/// use crate::signaller::Signaller;
///
/// use std::time::Duration;
/// use metron::Rate;
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
    /// Dunno yet.
    handle: JoinHandle<Result<()>>,
    /// Signal channel (receiver end).
    signal_rx: Receiver<Signal>,
    /// Control channel (sender end).
    control_tx: Sender<ControlMessage>,
}

/// The kind of signaller.
///
/// The signaller kind dictates the concurrency model that the signaller uses
/// to produce timing signals.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum SignallerKind {
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

impl Default for SignallerKind {
    fn default() -> Self {
        Self::Dedicated
    }
}

impl Signaller {
    /// Creates and runs a new `Signaller`.
    ///
    /// This function returns a [JoinHandle] that may be used to interact with
    /// the background workload. The completion of this `JoinHandle` should only
    /// be taken to mean that any asynchronous work needed to produce the
    /// signals is complete and not that there are no more signals available to
    /// be read. To ensure all signals have been read, the client should
    /// continue to call [`recv`][Self::recv] until `None` is returned.
    ///
    /// # Arguments
    ///
    /// * `kind` - Kind of `Signaller` to create
    pub fn run(kind: SignallerKind) -> Self {
        let (signal_tx, signal_rx) = mpsc::channel(CHAN_SIZE);
        let (control_tx, mut control_rx) = mpsc::channel(CHAN_SIZE);

        let handle = match kind {
            SignallerKind::Dedicated => task::spawn_blocking(move || {
                match control_rx.blocking_recv() {
                    None => return Ok(()),
                    Some(ControlMessage { plan, start }) => {
                        for t in plan.ticks(start) {
                            crate::wait::spin_until(t);
                            signal_tx.blocking_send(Signal::new(t))?;

                            // TODO: Don't check every single time through the loop.
                            if !control_rx.is_empty() {
                                info!("recieved another control message but don't know what to do");
                            }
                        }
                    }
                }
                Ok(())
            }),
            SignallerKind::Cooperative => task::spawn(async move {
                match control_rx.recv().await {
                    None => return Ok(()),
                    Some(ControlMessage { plan, start }) => {
                        for t in plan.ticks(start) {
                            crate::wait::sleep_until(t).await;
                            signal_tx.send(Signal::new(t)).await?;

                            // TODO: Don't check every single time through the loop.
                            if !control_rx.is_empty() {
                                info!("recieved another control message but don't know what to do");
                            }
                        }
                    }
                }
                Ok(())
            }),
        };

        Signaller {
            handle,
            signal_rx,
            control_tx,
        }
    }

    /// Receive a waiting signal or wait until one is available.
    ///
    /// This function can be used to obtain the next available timing signal.
    /// The oldest available timing signal will be returned or the returned
    /// future will block until one is available. This function is intended
    /// to be used as the synchronization point that drives request timing.
    pub async fn recv(&mut self) -> Option<Signal> {
        self.signal_rx.recv().await
    }

    pub async fn update(&self, plan: Plan, start: Instant) -> Result<()> {
        self.control_tx.send(ControlMessage { plan, start }).await?;
        Ok(())
    }

    pub fn stop(&self) {
        self.handle.abort();
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
struct ControlMessage {
    plan: Plan,
    start: Instant,
}
