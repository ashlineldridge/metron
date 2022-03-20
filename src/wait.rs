use std::time::Instant;

pub(crate) fn spin_until(t: Instant) {
    loop {
        if Instant::now() >= t {
            break;
        }

        std::hint::spin_loop();
    }
}

pub(crate) async fn sleep_until(t: Instant) {
    tokio::time::sleep_until(t.into()).await
}
