use anyhow::Result;
use std::time::Instant;
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
    pub fn start(&mut self) -> JoinHandle<Result<()>> {
        match self.kind {
            Kind::OnDemand => tokio::task::spawn(std::future::ready(Ok(()))),
            Kind::BlockingThread => {
                let tx = self.tx.take().expect(MULTIPLE_STARTS_ERROR);
                let plan = self.plan.clone();
                tokio::task::spawn_blocking(move || {
                    for t in plan {
                        crate::wait::spin_until(t);
                        tx.blocking_send(Signal::new(t))?;
                    }

                    Ok(())
                })
            }
            Kind::Cooperative => {
                let tx = self.tx.take().expect(MULTIPLE_STARTS_ERROR);
                let plan = self.plan.clone();
                tokio::task::spawn(async move {
                    for t in plan {
                        crate::wait::sleep_until(t).await;
                        tx.send(Signal::new(t)).await?;
                    }

                    Ok(())
                })
            }
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

// /// Builds a [`Signaller`] with custom configuration values.
// ///
// /// Methods can be chained in order to set the configuration values. The final
// /// `Signaller` is constructed by calling [`build`].
// ///
// /// New instances of `Builder` are obtained via [`Builder::new`] or the helper
// /// `Builder::new_xxx` functions that construct a specific kind of builder.
// ///
// /// See function level documentation for details on the various configuration
// /// settings.
// ///
// /// [`build`]: method@Self::build
// /// [`Builder::new`]: method@Self::new
// ///
// /// # Examples
// ///
// /// ```
// /// use crate::signaller::Builder;
// ///
// /// fn main() {
// ///     let signaller = Builder::new_blocking()
// ///         .worker_threads(4)
// ///         .thread_name("my-custom-name")
// ///         .thread_stack_size(3 * 1024 * 1024)
// ///         .build()
// ///         .unwrap();
// ///
// ///     // Use signaller...
// /// }
// /// ```
// pub struct Builder {
//     /// Signaller kind.
//     kind: Kind,
//     /// Plan used to determine request rate (none implies ASAP signalling).
//     plan: Option<Plan>,
// }

// impl Builder {
//     /// Returns a new runtime builder initialized with default configuration
//     /// values.
//     ///
//     /// Configuration methods can be chained on the return value.
//     pub(crate) fn new(kind: Kind) -> Builder {
//         Builder { kind }
//     }

//     pub(crate) fn build(&self) -> Result<Signaller> {
//         let signaller = Signaller {
//             kind: self.kind,
//             plan: self.plan,
//             tx: todo!(),
//         };

//         match self.kind {
//             Kind::Asap => Signaller,
//             Kind::Coop => todo!(),
//             Kind::NonCoop => todo!(),
//         }
//     }
// }

// // TODO:
// // Maybe builder pattern is better here
// // Look at Tokio Runtime Builder for example
// // Runtime has Kind which can equal either single or multi-threaded

// use crate::{error::SIGNALLER_MULTI_START_ERROR, plan::Plan};

// const CHAN_SIZE: usize = 1024;

// pub fn asap_signaller(duration: Option<Duration>) -> AsapSignaller {
//     AsapSignaller::new(duration)
// }

// pub fn blocking_signaller<S: Plan>(plan: S) -> BlockingSignaller<S> {
//     BlockingSignaller::new(plan, CHAN_SIZE)
// }

// pub fn async_signaller<S: Plan>(plan: S) -> AsyncSignaller<S> {
//     AsyncSignaller::new(plan, CHAN_SIZE)
// }

// #[derive(Debug)]
// pub struct Signal {
//     pub due: Instant,
// }

// impl Signal {
//     fn new(due: Instant) -> Self {
//         Self { due }
//     }

//     fn now() -> Self {
//         Self {
//             due: Instant::now(),
//         }
//     }
// }

// pub trait Signaller: Send {
//     fn start(&mut self) -> JoinHandle<Result<(), Error>> {
//         tokio::task::spawn(std::future::ready(Ok(())))
//     }

//     fn recv(&mut self) -> SignalFuture;

//     fn boxed<'a>(self) -> Box<dyn Signaller + 'a>
//     where
//         Self: Sized + 'a,
//     {
//         Box::new(self) as Box<dyn Signaller>
//     }
// }

// pub struct SignalFuture<'a> {
//     inner: Pin<Box<dyn Future<Output = Option<Signal>> + Send + 'a>>,
// }

// impl<'a> SignalFuture<'a> {
//     fn new<F>(value: F) -> Self
//     where
//         F: Future<Output = Option<Signal>> + Send + 'a,
//     {
//         Self {
//             inner: Box::pin(value),
//         }
//     }

//     fn now() -> Self {
//         Self::new(std::future::ready(Some(Signal::now())))
//     }

//     fn none() -> Self {
//         Self::new(std::future::ready(None))
//     }
// }

// impl<'a> Future for SignalFuture<'a> {
//     type Output = Option<Signal>;

//     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         self.inner.as_mut().poll(cx)
//     }
// }

// pub struct AsapSignaller {
//     since: Option<Instant>,
//     duration: Option<Duration>,
// }

// impl AsapSignaller {
//     pub fn new(duration: Option<Duration>) -> Self {
//         Self {
//             since: None,
//             duration,
//         }
//     }
// }

// impl Signaller for AsapSignaller {
//     fn recv(&mut self) -> SignalFuture {
//         let since = self.since.get_or_insert_with(Instant::now);
//         if self
//             .duration
//             .map(|limit| since.elapsed() < limit)
//             .unwrap_or(true)
//         {
//             SignalFuture::now()
//         } else {
//             SignalFuture::none()
//         }
//     }
// }

// pub struct BlockingSignaller<S: Plan> {
//     plan: Option<S>,
//     tx: Option<Sender<Signal>>,
//     rx: Receiver<Signal>,
// }

// impl<S: Plan> BlockingSignaller<S> {
//     fn new(plan: S, chan_size: usize) -> Self {
//         let (tx, rx) = tokio::sync::mpsc::channel(chan_size);
//         Self {
//             plan: Some(plan),
//             tx: Some(tx),
//             rx,
//         }
//     }
// }

// impl<S> Signaller for BlockingSignaller<S>
// where
//     S: Plan + Send + 'static,
// {
//     fn start(&mut self) -> JoinHandle<Result<(), Error>> {
//         let tx = self.tx.take().expect(SIGNALLER_MULTI_START_ERROR);
//         let plan = self.plan.take().expect(SIGNALLER_MULTI_START_ERROR);

//         tokio::task::spawn_blocking(move || {
//             for t in plan.iter_from(Instant::now()) {
//                 spin_until(t);
//                 tx.blocking_send(Signal::new(t))?;
//             }

//             Ok(())
//         })
//     }

//     fn recv(&mut self) -> SignalFuture {
//         SignalFuture::new(self.rx.recv())
//     }
// }

// pub struct AsyncSignaller<S: Plan> {
//     plan: Option<S>,
//     tx: Sender<Signal>,
//     rx: Receiver<Signal>,
// }

// impl<S: Plan> AsyncSignaller<S> {
//     fn new(plan: S, chan_size: usize) -> Self {
//         let (tx, rx) = tokio::sync::mpsc::channel(chan_size);
//         Self {
//             plan: Some(plan),
//             tx,
//             rx,
//         }
//     }
// }

// impl<S> Signaller for AsyncSignaller<S>
// where
//     S: Plan + Send + 'static,
// {
//     fn start(&mut self) -> JoinHandle<Result<(), Error>> {
//         let tx = self.tx.clone();
//         let plan = self
//             .plan
//             .take()
//             .expect("`AsyncSignaller` can only be started once");

//         tokio::task::spawn(async move {
//             for t in plan.iter_from(Instant::now()) {
//                 sleep_until(t).await;
//                 tx.send(Signal::new(t)).await?;
//             }

//             Ok(())
//         })
//     }

//     fn recv(&mut self) -> SignalFuture {
//         SignalFuture::new(self.rx.recv())
//     }
// }
