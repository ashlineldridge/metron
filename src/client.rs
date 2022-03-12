struct Result {
    due: Instant,
    sent: Instant,
    done: Instant,
    status: u16,
}

impl Result {
    fn actual_latency(&self) -> Duration {
        self.done - self.sent
    }

    fn corrected_latency(&self) -> Duration {
        self.done - self.due
    }
}
