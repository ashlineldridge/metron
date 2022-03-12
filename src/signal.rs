use crate::schedule::RequestSchedule;

use anyhow::Error;
use std::time::Instant;
use tokio::{sync::mpsc::Receiver, task::JoinHandle};

// Make it so it's possible to combine multiple signal generators
// e.g., to interleave requests
pub struct SignalGenerator {
    pub handle: JoinHandle<Result<(), Error>>,
    pub stream: Receiver<Signal>,
}

#[derive(Debug)]
pub struct Signal {
    pub due: Instant,
}

pub fn blocking_generator<S>(schedule: S, chan_size: usize) -> SignalGenerator
where
    S: Iterator<Item = RequestSchedule> + Send + 'static,
{
    let (tx, rx) = tokio::sync::mpsc::channel(chan_size);

    let handle = tokio::task::spawn_blocking(move || {
        for r in schedule {
            wait_until(r.start);
            tx.blocking_send(Signal { due: r.start })?;
        }

        Ok(())
    });

    SignalGenerator { handle, stream: rx }
}

pub fn async_generator<S>(schedule: S, chan_size: usize) -> SignalGenerator
where
    S: Iterator<Item = RequestSchedule> + Send + 'static,
{
    let (tx, rx) = tokio::sync::mpsc::channel(chan_size);

    let handle = tokio::task::spawn(async move {
        for r in schedule {
            wait_until(r.start);
            tx.send(Signal { due: r.start }).await?;
        }

        Ok(())
    });

    SignalGenerator { handle, stream: rx }
}

fn wait_until(t: Instant) {
    loop {
        if Instant::now() >= t {
            break;
        }

        std::hint::spin_loop();
    }
}
