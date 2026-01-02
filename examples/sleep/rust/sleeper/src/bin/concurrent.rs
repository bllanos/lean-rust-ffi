use std::ops::Range;
use std::process::{ExitCode, Termination};

use async_effect::{async_io::AsyncIo, io};
use sleeper::{block_and_print, sleep_and_print};

const MAXIMUM_SLEEP_TIME_SECONDS: u64 = 6;

const SECONDS_DURATION_RANGE: Range<u64> = 0..MAXIMUM_SLEEP_TIME_SECONDS;

fn main() -> impl Termination {
    let io_effect = io::println("Concurrent sleep operations");
    let io_effect = io::bind(io_effect, move |_| {
        let mut action = AsyncIo::pure(());
        for i in SECONDS_DURATION_RANGE {
            action = AsyncIo::concurrently(action, sleep_and_print(i)).bind(|_| AsyncIo::pure(()));
        }
        let io_effect = block_and_print(action);
        io::bind(io_effect, |_| io::pure_copy(ExitCode::SUCCESS))
    });
    io::run(io_effect)
}
