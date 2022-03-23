// pub struct ClientConfig {
//     uri: Uri,
// }

// // TODO: Is Client a really bad name?
// pub struct Client {
//     config: ClientConfig,
//     rx: Receiver<Signal>,
//     tx: Sender<ClientResult>,
// }

// impl Client {
//     pub fn new(config: ClientConfig, rx: Receiver<Signal>, tx: Sender<ClientResult>) -> Self {
//         Self { config, rx, tx }
//     }

//     pub fn run(&mut self) -> Result<(), Error> {
//         Ok(())
//     }
// }

use std::time::{Duration, Instant};

use anyhow::Error;

pub type StatusCode = u16;

#[derive(Debug)]
pub struct ClientResult {
    pub due: Instant,
    pub sent: Instant,
    pub done: Instant,
    pub status: Result<StatusCode, Error>,
}

impl ClientResult {
    pub fn actual_latency(&self) -> Duration {
        self.done - self.sent
    }

    pub fn corrected_latency(&self) -> Duration {
        self.done - self.due
    }
}
