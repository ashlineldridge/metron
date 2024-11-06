use std::time::Instant;

pub(crate) fn spin_until(t: Instant) {
    loop {
        if Instant::now() >= t {
            break;
        }

        std::hint::spin_loop();
    }
}

#[allow(unused)]
pub(crate) async fn sleep_until(t: Instant) {
    tokio::time::sleep_until(t.into()).await
}

#[allow(unused)]
pub(crate) fn sleep_thread_until(t: Instant) {
    if let Some(d) = t.checked_duration_since(Instant::now()) {
        std::thread::sleep(d)
    }
}
