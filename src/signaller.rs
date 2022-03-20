use anyhow::Result;
use std::time::Instant;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

use crate::plan::Plan;

const CHAN_SIZE: usize = 1024;
const MULTIPLE_STARTS_ERROR: &str = "`Signaller` can only be started once";

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

pub enum Kind {
    Asap,
    Coop,
    NonCoop,
}

impl Signaller {
    /// Create a new `Signaller`.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of Signaller to create
    /// * `now` - The plan used to determine request timing
    pub fn new(kind: Kind, plan: Plan) -> Self {
        let (tx, rx) = if kind == Kind::Asap {
            (None, None)
        } else {
            let (tx, rx) = tokio::sync::mpsc::channel(CHAN_SIZE);
            (Some(tx), Some(rx))
        };

        Self { kind, plan, tx, rx }
    }

    ///
    pub fn start(&mut self) -> JoinHandle<Result<()>> {
        if self.kind == Kind::Asap {
            return tokio::task::spawn(std::future::ready(Ok(())));
        }

        let plan = self.plan.take().expect(MULTIPLE_STARTS_ERROR);
        let tx = self.tx.take().expect(MULTIPLE_STARTS_ERROR);

        if self.kind == Kind::Coop {}

        tokio::task::spawn_blocking(move || {
            for t in plan.iter_from(Instant::now()) {
                crate::wait::spin_until(t);
                tx.blocking_send(Signal::new(t))?;
            }

            Ok(())
        })

        //     Kind::Coop => self.start_coop(),
        //     Kind::NonCoop => self.start_noncoop(),
        // }
    }

    fn start_coop(&mut self) -> JoinHandle<Result<()>> {
        let plan = self.plan.take().expect(MULTIPLE_STARTS_ERROR);

        todo!();
        // tokio::task::spawn(async move {
        //     for t in plan(Instant::now()) {
        //         sleep_until(t).await;
        //         tx.send(Signal::new(t)).await?;
        //     }

        //     Ok(())
        // })
    }

    fn start_noncoop(&mut self) -> JoinHandle<Result<()>> {
        let plan = self.plan.take().expect(MULTIPLE_STARTS_ERROR);
        let tx = self.tx.take().expect(MULTIPLE_STARTS_ERROR);

        tokio::task::spawn_blocking(move || {
            for t in plan.iter_from(Instant::now()) {
                crate::wait::spin_until(t);
                tx.blocking_send(Signal::new(t))?;
            }

            Ok(())
        })
    }

    pub async fn recv(&mut self) -> Option<Signal> {
        if self.kind == Kind::Asap {
            Some(Signal::now())
        } else {
            // Safe to unwrap since we control the lifecycle of rx.
            self.rx.unwrap().recv().await
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
