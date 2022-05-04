use anyhow::bail;
use std::{str::FromStr, time::Instant};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

use crate::load::plan::Plan;

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
/// use wrkr::Rate;
///
/// #[tokio::main]
/// async fn main() {
///     // Maximal throughput plan that runs for 60 seconds.
///     let plan = Builder::new()
///         .duration(Duration::from_secs(60))
///         .build()
///         .unwrap();
///
///     // Create an on-demand signaller that always produces the current time
///     // when a signal is requested.
///     let signaller = Signaller::new_on_demand(plan);
///     let sig = signaller.recv().await.unwrap();
///     println!("The next request should be sent at {}", sig.due);
///
///     // 100 RPS plan that runs for 60 seconds.
///     let plan = Builder::new()
///         .duration(Duration::from_secs(60))
///         .rate(Rate(100))
///         .build()
///         .unwrap();
///
///     // Create a blocking-thread signaller that uses a dedicated thread to place
///     // signals on a channel at the appropriate time.
///     let signaller = Signaller::new_blocking_thread(plan);
///     let sig = signaller.recv().await.unwrap();
///     println!("The next request should be sent at {}", sig.due);
///
///     // Create a cooperative signaller that uses a dedicated task to place
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// An `OnDemand` signaller produces timing signals when they are requested.
    /// This type of signaller does not use a background task or thread for
    /// producing timing signals; rather, it simply queries the supplied plan
    /// when [`Signaller::recv`] is called.
    OnDemand,

    /// A `BlockingThread` signaller creates a dedicated thread for producing
    /// timing signals. This is the most accurate signaller for interval-
    /// based timing due to the fact that it does not need to cooperate with
    /// the scheduler.
    BlockingThread,

    /// A `Cooperative` signaller uses a cooperatively scheduled Tokio task
    /// to produce timing signals. This type of signaller is useful in single-
    /// threaded environments or when you want to dedicate your threading
    /// resources elsewhere.
    Cooperative,
}

impl Signaller {
    /// Creates a new `Signaller`.
    ///
    /// # Arguments
    ///
    /// * `kind` - Kind of `Signaller` to create
    /// * `plan` - Plan used to determine request timing
    pub fn new(kind: Kind, plan: Plan) -> Self {
        let (tx, rx) = if kind == Kind::OnDemand {
            (None, None)
        } else {
            let (tx, rx) = tokio::sync::mpsc::channel(BACK_PRESSURE_CHAN_SIZE);
            (Some(tx), Some(rx))
        };

        Self { kind, plan, tx, rx }
    }

    /// Creates a new [on-demand][Kind::OnDemand] `Signaller`.
    ///
    /// # Arguments
    ///
    /// * `plan` - Plan used to determine request timing
    pub fn new_on_demand(plan: Plan) -> Self {
        Self::new(Kind::OnDemand, plan)
    }

    /// Creates a new [blocking-thread][Kind::BlockingThread] `Signaller`.
    ///
    /// # Arguments
    ///
    /// * `plan` - Plan used to determine request timing
    pub fn new_blocking_thread(plan: Plan) -> Self {
        Self::new(Kind::BlockingThread, plan)
    }

    /// Creates a new [cooperative][Kind::Cooperative] `Signaller`.
    ///
    /// # Arguments
    ///
    /// * `plan` - Plan used to determine request timing
    pub fn new_cooperative(plan: Plan) -> Self {
        Self::new(Kind::OnDemand, plan)
    }

    /// Start any background processes needed to generate timing signals.
    ///
    /// This function returns a [JoinHandle] that may be used to interact with
    /// the background workload. The completion of this `JoinHandle` should only
    /// be taken to mean that any asynchronous work needed to produce the
    /// signals is complete and not that there are no more signals available to
    /// be read. To ensure all signals have been read, the client should
    /// continue to call [`recv`][Self::recv] until `None` is returned.
    pub fn start(&mut self) -> JoinHandle<Result<(), anyhow::Error>> {
        let x = tokio::task::spawn(std::future::ready(Ok(())));
        x
        // match self.kind {
        //     Kind::OnDemand => tokio::task::spawn(std::future::ready(Ok(()))),
        //     Kind::BlockingThread => {
        //         let tx = self.tx.take().expect(MULTIPLE_STARTS_ERROR);
        //         let plan = self.plan.clone();
        //         tokio::task::spawn_blocking(move || {
        //             for t in plan {
        //                 crate::wait::spin_until(t);
        //                 tx.blocking_send(Signal::new(t))?;
        //             }

        //             Ok(())
        //         })
        //     }
        //     Kind::Cooperative => {
        //         let tx = self.tx.take().expect(MULTIPLE_STARTS_ERROR);
        //         let plan = self.plan.clone();
        //         tokio::task::spawn(async move {
        //             for t in plan {
        //                 crate::wait::sleep_until(t).await;
        //                 tx.send(Signal::new(t)).await?;
        //             }

        //             Ok(())
        //         })
        //     }
        // }
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
        if self.kind == Kind::OnDemand {
            // We are doing on-demand signalling so retrieve the next instant
            // directly from the plan rather than from the channel.
            self.plan.next().map(|t| Signal::new(t))
        } else {
            // Safe to unwrap since we control the lifecycle of rx.
            let rx = self.rx.as_mut().unwrap();
            rx.recv().await
        }
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

    fn now() -> Self {
        Self {
            due: Instant::now(),
        }
    }
}

impl FromStr for Kind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "blocking-thread" => Ok(Kind::BlockingThread),
            "cooperative" => Ok(Kind::Cooperative),
            "on-demand" => Ok(Kind::OnDemand),
            _ => bail!("Invalid signaller: {}", s),
        }
    }
}
