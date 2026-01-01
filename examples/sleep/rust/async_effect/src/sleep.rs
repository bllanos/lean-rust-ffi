use std::thread;
use std::time::Duration;

#[derive(Clone, Copy)]
pub struct Sleep {
    sleep_duration: Duration,
}

impl Sleep {
    pub fn sleep(sleep_duration: Duration) -> Self {
        Self { sleep_duration }
    }

    pub fn sleep_from_seconds<N: Into<u64>>(sleep_duration_seconds: N) -> Self {
        Self::sleep(Duration::from_secs(sleep_duration_seconds.into()))
    }

    pub fn sleep_from_milliseconds<N: Into<u64>>(sleep_duration_milliseconds: N) -> Self {
        Self::sleep(Duration::from_millis(sleep_duration_milliseconds.into()))
    }
}

pub fn run(sleep: Sleep) {
    if !sleep.sleep_duration.is_zero() {
        thread::sleep(sleep.sleep_duration);
    }
}
