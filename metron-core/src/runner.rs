use anyhow::{Context, Result};
use quanta::{Clock, Instant};
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::{info, warn};

use crate::{Action, Agent, AgentRequest, Plan, Sink};

const CHAN_SIZE: usize = 1024;

#[derive(Clone, Debug)]
struct State {
    plan: Plan,
    start: Instant,
}

#[allow(unused)]
#[derive(Clone)]
pub struct Runner {
    name: String,
    sinks: Vec<Sink>,
    state_tx: mpsc::Sender<Option<State>>,
}

impl Runner {
    pub fn run(name: String, sinks: Vec<Sink>) -> Self {
        let clock = Clock::new();
        let (state_tx, mut state_rx) = mpsc::channel::<Option<State>>(CHAN_SIZE);

        let _h0 = tokio::task::spawn(async move {
            let mut handles: Option<(JoinHandle<Result<()>>, JoinHandle<Result<()>>)> = None;

            // TODO: Incorporate current state to check if there's actually a diff. This will make it easier
            // on the proxy to send updates knowing that the agent won't stop/start the signalling unless there's
            // and actual change.
            // let mut current_state = None;

            while let Some(state) = state_rx.recv().await {
                if let Some((_signaller, executor)) = &handles {
                    executor.abort();
                    // executor.await;
                    // signaller.await;
                }

                if let Some(state) = state {
                    let (signal_tx, signal_rx) = mpsc::channel(CHAN_SIZE);
                    let signaller =
                        Self::spawn_blocking_signaller(clock.clone(), signal_tx, state.clone());
                    let executor = Self::spawn_executor(signal_rx, state);
                    handles = Some((signaller, executor));
                }
            }
        });

        Self {
            name,
            sinks,
            state_tx,
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

    fn spawn_blocking_signaller(
        clock: Clock,
        signal_tx: mpsc::Sender<Signal>,
        state: State,
    ) -> JoinHandle<Result<()>> {
        tokio::task::spawn_blocking(move || {
            let plan_duration = state.plan.calculate_duration();
            let stop_at = state.start.checked_add(plan_duration).unwrap();

            for (i, t) in state
                .plan
                .ticks(state.start)
                .filter(|&i| i <= stop_at)
                .enumerate()
            {
                let since_pre_a_micros =
                    clock.now().checked_duration_since(t).map(|d| d.as_micros());
                let since_pre_b_micros =
                    t.checked_duration_since(clock.now()).map(|d| d.as_micros());

                Self::spin_until(&clock, t);

                let since_a_micros = clock.now().checked_duration_since(t).map(|d| d.as_micros());
                let since_b_micros = t.checked_duration_since(clock.now()).map(|d| d.as_micros());

                if signal_tx.blocking_send(Signal::new(i as u32, t)).is_err() {
                    break;
                }

                info!(
                    id = i,
                    since_pre_a_micros = since_pre_a_micros,
                    since_pre_b_micros = since_pre_b_micros,
                    since_a_micros = since_a_micros,
                    since_b_micros = since_b_micros,
                    "signaller: after blocking send",
                );
            }

            warn!("signaller: done");
            Ok(())
        })
    }

    fn spawn_executor(
        mut signal_rx: mpsc::Receiver<Signal>,
        state: State,
    ) -> JoinHandle<Result<()>> {
        // Launch async "executor" task
        tokio::task::spawn(async move {
            while let Some(sig) = signal_rx.recv().await {
                info!(
                    id = sig.id,
                    elapsed_micros = sig.due.elapsed().as_micros(),
                    "executor: received signal and have current state"
                );

                let actions = state.plan.actions.clone();
                tokio::task::spawn(async move {
                    for action in &actions {
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
                    Result::<(), anyhow::Error>::Ok(())
                });
            }

            // signal_rx must have returned None to indicate that signal_tx has been dropped
            warn!("executor: done");
            Ok(())
        })
    }

    // async fn wait(&self) {
    //     self.
    // }
}

impl Agent for Runner {
    async fn execute(&self, req: AgentRequest) -> Result<()> {
        info!("runner: sending test state to agent");
        let AgentRequest { plan, start } = req;
        self.state_tx
            .send(Some(State { plan, start }))
            .await
            .context("could not send state")?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Signal {
    pub id: u32,
    pub due: Instant,
}

impl Signal {
    fn new(id: u32, due: Instant) -> Self {
        Self { id, due }
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
