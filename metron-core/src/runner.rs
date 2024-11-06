use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};

use crate::{wait, Action, Agent, Plan, Sink};

const CHAN_SIZE: usize = 1024;

// #[derive(Clone, Debug)]
// pub enum Signaller {
//     /// A `Dedicated` signaller creates a dedicated thread for producing
//     /// timing signals. This is the most accurate signaller for interval-
//     /// based timing due to the fact that it does not need to cooperate with
//     /// the scheduler.
//     Dedicated,

//     /// A `Cooperative` signaller uses a cooperatively scheduled Tokio task
//     /// to produce timing signals. This type of signaller is useful in single-
//     /// threaded environments or when you want to dedicate your threading
//     /// resources elsewhere.
//     Cooperative,
// }

#[derive(Clone, Debug)]
struct State {
    plan: Plan,
    start: Instant,
}

#[allow(unused)]
pub struct Runner {
    name: String,
    state_tx: watch::Sender<Option<State>>,
    sinks: Vec<Sink>,
}

impl Runner {
    pub fn run_dedicated(name: String, sinks: Vec<Sink>) -> Self {
        let (signal_tx, signal_rx) = mpsc::channel(CHAN_SIZE);
        let (state_tx, state_rx) = watch::channel(None);

        let _h1 = Self::spawn_dedicated_signaller(signal_tx, state_rx.clone());
        let _h2 = Self::spawn_executor(signal_rx, state_rx);

        // TODO: Potentially need to put these handles into Self or equivalent.
        Self {
            name,
            state_tx,
            sinks,
        }
    }

    fn spawn_executor(
        mut signal_rx: mpsc::Receiver<Signal>,
        mut state_rx: watch::Receiver<Option<State>>,
    ) -> JoinHandle<Result<()>> {
        // Launch async "executor" task
        tokio::task::spawn(async move {
            let mut state = None;
            loop {
                tokio::select! {
                    res = state_rx.changed() => {
                        if res.is_err() {
                            // State sender has been dropped so complete.
                            return Ok(());
                        }
                        state = state_rx.borrow().clone();
                    },
                    sig = signal_rx.recv() => {
                        match (sig, state.clone()) {
                            // Signaller sender has been dropped so complete.
                            (None, _) => return Ok(()),
                            (Some(_sig), Some(state)) => {
                                tokio::task::spawn(async move {
                                    for action in &state.plan.actions {
                                        if let Action::Http {
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
                                    Result::<(), anyhow::Error>::Ok(())
                                });
                            },
                            _ => (),
                        };
                    },
                };
            }
        })
    }

    fn spawn_dedicated_signaller(
        signal_tx: mpsc::Sender<Signal>,
        mut state_rx: watch::Receiver<Option<State>>,
    ) -> JoinHandle<Result<()>> {
        // Start the thread-blocking task that generates the timing signals.
        let handle = tokio::task::spawn_blocking(move || {
            let mut state = state_rx.borrow().clone();
            loop {
                if let Some(State { plan, start }) = &state {
                    for t in plan.ticks(*start) {
                        wait::spin_until(t);

                        if signal_tx.blocking_send(Signal::new(t)).is_err() {
                            return Result::<(), anyhow::Error>::Ok(());
                        }

                        match state_rx.has_changed() {
                            Err(_) => return Result::<(), anyhow::Error>::Ok(()),
                            Ok(false) => (),
                            Ok(true) => {
                                let new_state = state_rx.borrow_and_update().clone();
                                // TODO: Should probably implement Eq, etc for State (and Plan...) as
                                // has_changed returns true even if the underlying value is the same.
                                // if state != new_state { break; }
                                state = new_state;
                                break;
                            }
                        }
                    }
                } else {
                    std::thread::sleep(Duration::from_millis(500));
                }
            }
        });

        handle
    }
}

impl Agent for Runner {
    async fn test(&self, plan: &Plan) -> Result<(), crate::AgentError> {
        // TODO: This needs to be when the test started.
        let state = State {
            plan: plan.clone(),
            start: Instant::now(),
        };
        self.state_tx
            .send(Some(state))
            .context("could not update runner")?;
        Ok(())
    }

    async fn cancel(&self) -> Result<(), crate::AgentError> {
        self.state_tx.send(None).context("could not stop runner")?;
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

// #[derive(Debug)]
// pub struct Sample {
//     pub target: Url,
//     pub due: Instant,
//     pub sent: Instant,
//     pub done: Instant,
//     pub status: Result<u16, anyhow::Error>,
// }

// impl Sample {
//     pub fn actual_latency(&self) -> Duration {
//         self.done - self.sent
//     }

//     pub fn corrected_latency(&self) -> Duration {
//         self.done - self.due
//     }

//     // TODO: What is this?
//     pub fn client_latency(&self) -> Duration {
//         self.sent - self.due
//     }
// }
