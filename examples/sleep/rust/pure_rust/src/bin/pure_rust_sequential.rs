use std::ops::Range;
use std::process::{ExitCode, Termination};

use async_effect::{async_io::AsyncIo, io};
use sleeper_pure_rust::{block_and_print, sleep_and_print};

const MAXIMUM_SLEEP_TIME_SECONDS: u64 = 6;

const SECONDS_DURATION_RANGE: Range<u64> = 0..MAXIMUM_SLEEP_TIME_SECONDS;

fn main() -> impl Termination {
    let io_effect = io::println("Sequential sleep operations");
    let io_effect = io::bind(io_effect, |_| {
        let ascending_action = AsyncIo::for_m(SECONDS_DURATION_RANGE, sleep_and_print);
        let descending_action = AsyncIo::for_m(SECONDS_DURATION_RANGE, |x| {
            sleep_and_print(MAXIMUM_SLEEP_TIME_SECONDS - x)
        });
        let action = ascending_action.bind(move |_| descending_action.clone());
        let io_effect = block_and_print(action);
        io::bind(io_effect, |_| io::pure(ExitCode::SUCCESS))
    });
    io::run(io_effect)
}
