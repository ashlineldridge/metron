use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::Result;
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::info;

use crate::{wait, Action, Agent, Plan, Sink};

const CHAN_SIZE: usize = 1024;
const POLL_PERIOD: Duration = Duration::from_secs(5);

#[derive(Clone, Debug)]
struct State {
    plan: Plan,
    start: Instant,
}

#[allow(unused)]
pub struct Runner {
    name: String,
    state: Arc<Mutex<Option<State>>>,
    sinks: Vec<Sink>,
}

impl Runner {
    pub fn run(name: String, sinks: Vec<Sink>) -> Self {
        let (signal_tx, signal_rx) = mpsc::channel(CHAN_SIZE);
        let state: Arc<Mutex<Option<State>>> = Arc::new(Mutex::new(None));

        let _h1 = Self::spawn_signaller(signal_tx, state.clone());
        let _h2 = Self::spawn_executor(signal_rx, state.clone());

        // TODO: Potentially need to put these handles into Self or equivalent.
        Self { name, state, sinks }
    }

    fn spawn_signaller(
        signal_tx: mpsc::Sender<Signal>,
        state: Arc<Mutex<Option<State>>>,
    ) -> JoinHandle<Result<()>> {
        tokio::task::spawn_blocking(move || {
            while state.lock().unwrap().is_none() {
                std::thread::sleep(POLL_PERIOD);
            }

            let state = state.lock().unwrap().clone().unwrap();

            // TODO: Consider forcing the plan to have a stop time.
            let plan_duration = state.plan.calculate_duration().unwrap();
            let stop_at = state.start.checked_add(plan_duration).unwrap();

            for (i, t) in state
                .plan
                .ticks(state.start)
                .filter(|&i| i <= stop_at)
                .enumerate()
            {
                let since_pre_a_micros = Instant::now()
                    .checked_duration_since(t)
                    .map(|d| d.as_micros());
                let since_pre_b_micros = t
                    .checked_duration_since(Instant::now())
                    .map(|d| d.as_micros());

                wait::spin_until(t);

                let since_a_micros = Instant::now()
                    .checked_duration_since(t)
                    .map(|d| d.as_micros());
                let since_b_micros = t
                    .checked_duration_since(Instant::now())
                    .map(|d| d.as_micros());

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

                // TODO: Check the mutex every POLL_PERIOD and if it has changed break the loop.
            }

            Ok(())
        })
    }

    fn spawn_executor(
        mut signal_rx: mpsc::Receiver<Signal>,
        state: Arc<Mutex<Option<State>>>,
    ) -> JoinHandle<Result<()>> {
        // Launch async "executor" task
        tokio::task::spawn(async move {
            while state.lock().unwrap().is_none() {
                tokio::time::sleep(POLL_PERIOD).await;
            }

            let state = state.lock().unwrap().clone().unwrap();

            loop {
                let sig = signal_rx.recv().await;
                match sig {
                    // Signaller sender has been dropped so complete.
                    None => {
                        info!("executor: signal_tx must have been dropped");
                        return Ok(());
                    }
                    Some(sig) => {
                        info!(
                            id = sig.id,
                            elapsed_micros = sig.due.elapsed().as_micros(),
                            "executor: received signal and have current state"
                        );

                        let actions = state.plan.actions.clone();
                        tokio::task::spawn(async move {
                            for action in &actions {
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
                    }
                };

                // TODO: Check the mutex every POLL_PERIOD and if it has changed break the loop.
            }
        })
    }
}

impl Agent for Runner {
    async fn test(&self, plan: &Plan) -> Result<(), crate::AgentError> {
        info!("sending test state to agent");

        let mut state = self.state.lock().unwrap();
        *state = Some(State {
            plan: plan.clone(),
            // TODO: This needs to be when the test started.
            start: Instant::now(),
        });

        info!("test state has been sent to agent");
        Ok(())
    }

    async fn cancel(&self) -> Result<(), crate::AgentError> {
        let mut state = self.state.lock().unwrap();
        *state = None;
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
