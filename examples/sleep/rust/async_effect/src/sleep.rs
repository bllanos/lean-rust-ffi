use std::cmp::{Ord, Ordering};
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy)]
pub enum ConcurrentOrder {
    Equal(Sleep),
    SameOrder(Sleep, Sleep),
    ReverseOrder(Sleep, Sleep),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Sleep {
    sleep_duration: Duration,
}

impl Sleep {
    pub fn new(sleep_duration: Duration) -> Self {
        Self { sleep_duration }
    }

    pub fn from_seconds<N: Into<u64>>(sleep_duration_seconds: N) -> Self {
        Self::new(Duration::from_secs(sleep_duration_seconds.into()))
    }

    pub fn from_milliseconds<N: Into<u64>>(sleep_duration_milliseconds: N) -> Self {
        Self::new(Duration::from_millis(sleep_duration_milliseconds.into()))
    }

    pub fn concurrently(first: Sleep, second: Sleep) -> ConcurrentOrder {
        match first.cmp(&second) {
            Ordering::Equal => ConcurrentOrder::Equal(first),
            Ordering::Less => ConcurrentOrder::SameOrder(
                first,
                Self {
                    sleep_duration: second.sleep_duration - first.sleep_duration,
                },
            ),
            Ordering::Greater => ConcurrentOrder::ReverseOrder(
                second,
                Self {
                    sleep_duration: first.sleep_duration - second.sleep_duration,
                },
            ),
        }
    }
}

pub fn run(sleep: Sleep) {
    if !sleep.sleep_duration.is_zero() {
        thread::sleep(sleep.sleep_duration);
    }
}
