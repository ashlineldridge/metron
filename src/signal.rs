use crate::schedule::RequestSchedule;

use anyhow::Error;
use std::time::Instant;
use tokio::{sync::mpsc::Sender, task::JoinHandle};

pub struct SignalGenerator {
    tx: Sender<Signal>,
}

#[derive(Debug)]
pub struct Signal {
    pub due: Instant,
}

impl SignalGenerator {
    fn new(tx: Sender<Signal>) -> Self {
        Self { tx }
    }

    pub fn run_blocking<S>(&self, schedule: S) -> JoinHandle<Result<(), Error>>
    where
        S: Iterator<Item = RequestSchedule> + Send + 'static,
    {
        let tx = self.tx.clone();
        tokio::task::spawn_blocking(move || {
            for r in schedule {
                spin_until(r.start);
                tx.blocking_send(Signal { due: r.start })?;
            }

            Ok(())
        })
    }

    pub fn run_async<S>(&self, schedule: S) -> JoinHandle<Result<(), Error>>
    where
        S: Iterator<Item = RequestSchedule> + Send + 'static,
    {
        let tx = self.tx.clone();
        tokio::task::spawn(async move {
            for r in schedule {
                sleep_until(r.start);
                tx.send(Signal { due: r.start }).await?;
            }

            Ok(())
        })
    }
}

fn spin_until(t: Instant) {
    loop {
        if Instant::now() >= t {
            break;
        }

        std::hint::spin_loop();
    }
}

async fn sleep_until(t: Instant) {
    tokio::time::sleep_until(t.into()).await
}
