use std::ops::Range;
use std::process::{ExitCode, Termination};

use async_effect::{async_io, io};
use sleeper::{block_and_print, sleep_and_print};

const SECONDS_DURATION_RANGE: Range<u64> = 0..6;

fn main() -> impl Termination {
    let io_effect = io::println("Sequential sleep operations");
    let io_effect = io::bind(io_effect, |_| {
        let action = async_io::for_m(SECONDS_DURATION_RANGE, sleep_and_print);
        let io_effect = block_and_print(action);
        io::bind(io_effect, |_| io::pure_copy(ExitCode::SUCCESS))
    });
    io::run(io_effect)
}
