use crate::signal::Signal;
use anyhow::Error;
use hyper::{StatusCode, Uri};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct ClientConfig {
    uri: Uri,
}

pub struct Client {
    config: ClientConfig,
    rx: Receiver<Signal>,
    tx: Sender<ClientResult>,
}

impl Client {
    pub fn new(config: ClientConfig, rx: Receiver<Signal>, tx: Sender<ClientResult>) -> Self {
        Self { config, rx, tx }
    }

    pub fn run(&self) -> Result<(), Error> {
        let tx = self.tx.clone();
        let rx = self.rx;
        let uri = self.config.uri.clone();

        tokio::spawn(async move {
            let client = hyper::Client::new();

            while let Some(sig) = rx.recv().await {
                let client = client.clone();
                let tx = tx.clone();
                let uri = uri.clone();

                tokio::spawn(async move {
                    let sent = Instant::now();
                    let response = client.get(uri.clone()).await;
                    let done = Instant::now();

                    let result = ClientResult {
                        due: sig.due,
                        sent,
                        done,
                        response: response.map(|r| r.status()).map_err(|e| e.into()),
                    };

                    tx.send(result).await.unwrap();
                });
            }
        });

        Ok(())
    }
}

#[derive(Debug)]
pub struct ClientResult {
    pub due: Instant,
    pub sent: Instant,
    pub done: Instant,
    pub response: Result<StatusCode, Error>,
}

impl ClientResult {
    pub fn actual_latency(&self) -> Duration {
        self.done - self.sent
    }

    pub fn corrected_latency(&self) -> Duration {
        self.done - self.due
    }
}
