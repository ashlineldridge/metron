use std::sync::{Arc, Mutex};

use anyhow::Result;
use hdrhistogram::Histogram;
use quanta::{Clock, Instant};
use tokio::sync::{mpsc, oneshot, Notify};
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

#[derive(Clone, Debug)]
#[allow(unused)]
enum ControlMessage {
    Plan(Plan),
    Exit,
}

/////////////////////////////////////////////////////////////////////////////
// Executor
/////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
struct Executor(mpsc::Sender<Signal>);

impl Executor {
    fn spawn(plan: Arc<Plan>) -> Executor {
        let (tx, mut rx) = mpsc::channel::<Signal>(CHAN_SIZE);
        tokio::spawn(async move {
            info!("executor: spawned");
            while let Some(sig) = rx.recv().await {
                let plan = plan.clone();
                tokio::task::spawn(async move {
                    info!(
                        signal_delay_micros = Instant::now().duration_since(sig.due).as_micros(),
                        "executor: spawned signal processing task"
                    );

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
            }

            info!("executor: done");
            anyhow::Result::<()>::Ok(())
        });

        Executor(tx)
    }
}

/////////////////////////////////////////////////////////////////////////////
// Signaller
/////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
struct Signaller(Arc<Notify>);

impl Signaller {
    fn spawn(
        plan: Arc<Plan>,
        clock: Clock,
        executor: Executor,
        mut cancel: oneshot::Receiver<()>,
    ) -> Signaller {
        let done = Arc::new(Notify::new());
        let signaller = Self(done.clone());

        tokio::task::spawn_blocking(move || {
            for tick in plan.ticks(clock.now()) {
                if cancel.try_recv().is_ok() {
                    info!("signaller: cancelling");
                    break;
                }

                spin_until(&clock, tick);
                if executor.0.blocking_send(Signal::new(tick)).is_err() {
                    info!("signaller: couldn't signal executor - exiting");
                    break;
                }
            }

            info!("signaller: done");
            done.notify_waiters();
            anyhow::Result::<()>::Ok(())
        });

        signaller
    }

    async fn done(&self) {
        self.0.notified().await;
    }
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

struct Handle {
    signaller: Signaller,
    cancel: oneshot::Sender<()>,
}

impl Handle {
    fn new(signaller: Signaller, cancel: oneshot::Sender<()>) -> Self {
        Self { signaller, cancel }
    }
}

#[allow(unused)]
#[derive(Clone)]
pub struct Runner {
    name: String,
    clock: Clock,
    sinks: Vec<Sink>,
    running: Arc<Mutex<Option<Handle>>>,
}

impl Runner {
    pub fn new(name: String, clock: Clock, sinks: Vec<Sink>) -> Self {
        Self {
            name,
            clock,
            sinks,
            running: Arc::new(Mutex::new(None)),
        }
    }

    fn run(&self, plan: Plan) -> Result<Signaller> {
        let mut guard = self.running.lock().unwrap();
        if let Some(handle) = guard.take() {
            let _ = handle.cancel.send(());
        }

        let plan = Arc::new(plan);
        let executor = Executor::spawn(plan.clone());
        let (cancel_tx, cancel_rx) = oneshot::channel();
        let signaller = Signaller::spawn(plan, self.clock.clone(), executor, cancel_rx);
        let handle = Handle::new(signaller.clone(), cancel_tx);

        *guard = Some(handle);

        Ok(signaller)
    }

    pub async fn run_wait(&self, plan: Plan) -> Result<()> {
        let signaller = self.run(plan)?;
        signaller.done().await;
        Ok(())
    }
}

impl Agent for Runner {
    async fn exec(&self, plan: Plan) -> Result<()> {
        info!("runner: exec called");
        self.run(plan)?;
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        info!("runner: stop called");
        let handle = {
            let mut guard = self.running.lock().unwrap();
            guard.take()
        };

        if let Some(handle) = handle {
            if handle.cancel.send(()).is_ok() {
                handle.signaller.done().await;
            }
        }

        Ok(())
    }

    async fn report(&self, _kind: ReportKind) -> Result<Report> {
        info!("runner: report called");
        Ok(Report {
            data: Histogram::new(4)?,
        })
    }
}
