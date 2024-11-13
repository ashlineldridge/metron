use std::sync::Arc;

use anyhow::Result;
use hdrhistogram::Histogram;
use quanta::{Clock, Instant};
use tokio::sync::{mpsc, RwLock};
use tracing::info;

use crate::{Action, Agent, Plan, Report, ReportKind, Sink};

const CHAN_SIZE: usize = 1024;

#[derive(Clone, Debug)]
pub struct Signal {
    pub due: Instant,
}

impl Signal {
    fn new(due: Instant) -> Self {
        Self { due }
    }
}

#[derive(Clone)]
#[allow(unused)]
enum ControlMessage {
    Plan(Plan),
    Exit,
}

/////////////////////////////////////////////////////////////////////////////
// Executor
/////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
struct ExecutorHandle {
    ctl_tx: mpsc::Sender<ControlMessage>,
    // Use separate channel as signaling is sacred.
    sig_tx: mpsc::Sender<Signal>,
}

impl ExecutorHandle {
    pub fn new(ctl_tx: mpsc::Sender<ControlMessage>, sig_tx: mpsc::Sender<Signal>) -> Self {
        Self { ctl_tx, sig_tx }
    }

    #[allow(unused)]
    async fn signal(&self, sig: Signal) {
        let _ = self.sig_tx.send(sig).await;
    }

    fn blocking_signal(&self, sig: Signal) {
        let _ = self.sig_tx.blocking_send(sig);
    }

    async fn message(&self, msg: ControlMessage) {
        let _ = self.ctl_tx.send(msg).await;
    }
}

fn spawn_executor() -> ExecutorHandle {
    let (msg_tx, mut msg_rx) = mpsc::channel::<ControlMessage>(CHAN_SIZE);
    let (sig_tx, mut sig_rx) = mpsc::channel::<Signal>(CHAN_SIZE);

    tokio::spawn(async move {
        let current_plan: Arc<RwLock<Plan>> = Arc::new(RwLock::new(Plan::empty()));
        loop {
            tokio::select! {
                Some(msg) = msg_rx.recv() => {
                    match msg {
                        ControlMessage::Plan(plan) => {
                            let mut current_plan = current_plan.write().await;
                            *current_plan = plan;
                        },
                        ControlMessage::Exit => {
                            break;
                        },
                    }
                },
                Some(_sig) = sig_rx.recv() => {
                    let current_plan = current_plan.clone();
                    tokio::task::spawn(async move {
                        let plan = current_plan.read().await;
                        for action in &plan.actions {
                            if let Action::Http {
                                name: _,
                                target,
                                method,
                                headers,
                                payload,
                            } = action
                            {
                                let headers = headers.try_into()?;
                                let client = reqwest::ClientBuilder::new()
                                    .default_headers(headers)
                                    .build()?;
                                client
                                    .request(method.into(), target.to_string())
                                    .body(payload.clone())
                                    .send()
                                    .await?;
                            }
                        }
                        anyhow::Result::<()>::Ok(())
                    });
                },
                else => break,
            };
        }
    });

    ExecutorHandle::new(msg_tx, sig_tx)
}

/////////////////////////////////////////////////////////////////////////////
// Signaller
/////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
struct SignallerHandle {
    ctl_tx: mpsc::Sender<ControlMessage>,
}

impl SignallerHandle {
    pub fn new(ctl_tx: mpsc::Sender<ControlMessage>) -> Self {
        Self { ctl_tx }
    }

    async fn message(&self, msg: ControlMessage) {
        let _ = self.ctl_tx.send(msg).await;
    }
}

fn spawn_blocking_signaller(clock: Clock, handle: ExecutorHandle) -> SignallerHandle {
    let (ctl_tx, mut ctl_rx) = mpsc::channel::<ControlMessage>(CHAN_SIZE);

    tokio::task::spawn_blocking(move || {
        loop {
            let plan = match ctl_rx.blocking_recv() {
                None | Some(ControlMessage::Exit) => break,
                Some(ControlMessage::Plan(plan)) => plan,
            };

            for tick in plan.ticks(clock.now()) {
                spin_until(&clock, tick);
                handle.blocking_signal(Signal::new(tick));

                // If the message receiver is closed or contains a message then break out
                // of this loop and we'll evaluate the state at the top of the outer loop.
                if ctl_rx.is_closed() || !ctl_rx.is_empty() {
                    break;
                }
            }
        }
        anyhow::Result::<()>::Ok(())
    });

    SignallerHandle::new(ctl_tx)
}

fn spin_until(clock: &Clock, t: Instant) {
    loop {
        if clock.now() >= t {
            break;
        }
        std::hint::spin_loop();
    }
}

/////////////////////////////////////////////////////////////////////////////
// Runner
/////////////////////////////////////////////////////////////////////////////

#[allow(unused)]
#[derive(Clone)]
pub struct Runner {
    name: String,
    signaller: SignallerHandle,
    executor: ExecutorHandle,
    sinks: Vec<Sink>,
}

impl Runner {
    pub fn spawn(name: String, clock: Clock, sinks: Vec<Sink>) -> Self {
        let executor = spawn_executor();
        let signaller = spawn_blocking_signaller(clock, executor.clone());
        Self {
            name,
            signaller,
            executor,
            sinks,
        }
    }

    async fn update_plan(&self, plan: Plan) {
        let msg = ControlMessage::Plan(plan);
        self.executor.message(msg.clone()).await;
        self.signaller.message(msg).await;
    }
}

impl Agent for Runner {
    async fn exec(&self, plan: Plan) -> Result<()> {
        self.update_plan(plan).await;
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        self.update_plan(Plan::empty()).await;
        Ok(())
    }

    async fn report(&self, _kind: ReportKind) -> Result<Report> {
        Ok(Report {
            data: Histogram::new(4)?,
        })
    }
}
