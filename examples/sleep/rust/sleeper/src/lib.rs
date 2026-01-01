use std::fmt::Display;
use std::time::{Duration, Instant};

use async_effect::{
    async_io::{self, AsyncIo},
    io::{self, BaseIo},
};

fn format_elapsed_time(start_time: Instant, end_time: Instant) -> (u64, u128) {
    let elapsed_time = end_time - start_time;
    let elapsed_time_seconds = elapsed_time.as_secs();
    let elapsed_time_remainder =
        (elapsed_time - Duration::from_secs(elapsed_time_seconds)).as_millis();
    (elapsed_time_seconds, elapsed_time_remainder)
}

pub fn sleep_and_print<N: 'static + Copy + Display + Into<u64>>(
    sleep_duration_seconds: N,
) -> impl AsyncIo<()> {
    let start_time_effect = async_io::of_base_io(io::monotonic_now());
    async_io::bind(start_time_effect, move |start_time| {
        let sleep_effect = async_io::sleep_from_seconds(sleep_duration_seconds);
        async_io::bind(sleep_effect, move |_| {
            let end_time_effect = async_io::of_base_io(io::monotonic_now());
            async_io::bind(end_time_effect, move |end_time| {
                let (elapsed_time_seconds, elapsed_time_remainder) =
                    format_elapsed_time(start_time, end_time);
                async_io::of_base_io(io::println(format!(
                    "Called sleep for {sleep_duration_seconds} seconds (actual sleep duration {elapsed_time_seconds} seconds {elapsed_time_remainder} milliseconds)"
                )))
            })
        })
    })
}

pub fn block_and_print<F: AsyncIo<()>>(action: F) -> impl BaseIo<()> {
    let cloneable_action = async_io::arc(action);
    let start_time_effect = io::monotonic_now();
    io::bind(start_time_effect, move |start_time| {
        let io_effect = async_io::block(cloneable_action.clone());
        io::bind(io_effect, move |_| {
            let end_time_effect = io::monotonic_now();
            io::bind(end_time_effect, move |end_time| {
                let (elapsed_time_seconds, elapsed_time_remainder) =
                    format_elapsed_time(start_time, end_time);
                io::println(format!(
                    "Total duration {elapsed_time_seconds} seconds {elapsed_time_remainder} milliseconds"
                ))
            })
        })
    })
}
