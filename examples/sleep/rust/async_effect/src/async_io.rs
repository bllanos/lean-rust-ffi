use std::any::Any;
use std::time::Duration;

use crate::{io::BaseIo, sleep::Sleep};

mod inner;

use inner::{ConcurrentOrderInner, Inner};

/// A trait bound on the type parameter of [`AsyncIo`] that ensures that
/// [`AsyncIo`] itself satisfies these traits
pub trait Value: 'static + Any + Clone + Send + Sync {}

impl<T: 'static + Any + Clone + Send + Sync> Value for T {}

pub trait Callback: 'static + Send + Sync {}

impl<F: 'static + Send + Sync> Callback for F {}

#[derive(Clone)]
pub enum ConcurrentOrder<T: Value, U: Value> {
    First(T, AsyncIo<U>),
    Both(T, U),
    Second(AsyncIo<T>, U),
}

impl<T: Value, U: Value> From<ConcurrentOrderInner<T, U>> for ConcurrentOrder<T, U> {
    fn from(value: ConcurrentOrderInner<T, U>) -> Self {
        match value {
            ConcurrentOrderInner::First(a, y) => Self::First(a, AsyncIo(y)),
            ConcurrentOrderInner::Both(a, b) => Self::Both(a, b),
            ConcurrentOrderInner::Second(x, b) => Self::Second(AsyncIo(x), b),
        }
    }
}

#[derive(Clone)]
pub enum MaybeAsyncIo<T: Value> {
    Ready(T),
    Pending(AsyncIo<T>),
}

#[derive(Clone)]
pub struct AsyncIo<T: Value>(Inner<T>);

impl<T: Value> AsyncIo<T> {
    pub fn pure(value: T) -> Self {
        Self(Inner::pure(value))
    }

    pub fn of_base_io<F: BaseIo<T> + Callback>(io_effect: F) -> Self {
        Self(Inner::of_base_io(io_effect))
    }

    pub fn bind<U: Value, F: Fn(T) -> AsyncIo<U> + Value>(self, f: F) -> AsyncIo<U> {
        AsyncIo(self.0.bind(move |value| f(value).0))
    }

    pub fn map<U: Value, F: Fn(T) -> U + Value>(self, f: F) -> AsyncIo<U> {
        AsyncIo(self.0.map(f))
    }

    pub fn block_immediate(self) -> T {
        self.0.block_immediate()
    }

    pub fn block(self) -> impl BaseIo<T> + 'static {
        self.0.block()
    }

    pub fn concurrently<U: Value>(x: Self, y: AsyncIo<U>) -> AsyncIo<(T, U)> {
        AsyncIo(Inner::concurrently(x.0, y.0))
    }

    pub fn select<U: Value>(x: Self, y: AsyncIo<U>) -> AsyncIo<ConcurrentOrder<T, U>> {
        AsyncIo(
            Inner::select(x.0, y.0)
                .map(<ConcurrentOrderInner<T, U> as Into<ConcurrentOrder<T, U>>>::into),
        )
    }

    pub fn concurrently_all<I: IntoIterator<Item = Self>>(collection: I) -> AsyncIo<Vec<T>> {
        collection
            .into_iter()
            .fold(AsyncIo::pure(Vec::new()), |accumulator, item| {
                AsyncIo::concurrently(accumulator, item).map(|(mut v, value)| {
                    v.push(value);
                    v
                })
            })
    }

    fn select_all_iterator<I: Iterator<Item = Self>>(mut iter: I) -> AsyncIo<Vec<MaybeAsyncIo<T>>> {
        let first = iter.next();
        match first {
            None => AsyncIo::pure(Vec::new()),
            Some(x) => {
                let rest = Self::select_all_iterator(iter);
                Self::select(x, rest).bind(|order| match order {
                    ConcurrentOrder::First(a, y) => y.map(move |mut v| {
                        v.push(MaybeAsyncIo::Ready(a.clone()));
                        v
                    }),
                    ConcurrentOrder::Both(a, mut v) => {
                        v.push(MaybeAsyncIo::Ready(a));
                        AsyncIo::pure(v)
                    }
                    ConcurrentOrder::Second(x, v) => x.map(move |a| {
                        let mut v = v.clone();
                        v.push(MaybeAsyncIo::Ready(a));
                        v
                    }),
                })
            }
        }
    }

    pub fn select_all<I: IntoIterator<Item = Self>>(
        collection: I,
    ) -> AsyncIo<Vec<MaybeAsyncIo<T>>> {
        Self::select_all_iterator(collection.into_iter()).map(|mut v| {
            v.reverse();
            v
        })
    }
}

impl From<Sleep> for AsyncIo<()> {
    fn from(sleep: Sleep) -> Self {
        Self(sleep.into())
    }
}

impl AsyncIo<()> {
    pub fn for_m<U: Value, I, F>(collection: I, f: F) -> Self
    where
        I: IntoIterator<Item = U>,
        F: Fn(U) -> Self + Value,
    {
        Self(Inner::for_m(collection, move |value| f(value).0))
    }

    pub fn sleep(sleep_duration: Duration) -> Self {
        Sleep::new(sleep_duration).into()
    }

    pub fn sleep_from_milliseconds<N: Into<u64>>(sleep_duration_milliseconds: N) -> Self {
        Sleep::from_milliseconds(sleep_duration_milliseconds).into()
    }

    pub fn sleep_from_seconds<N: Into<u64>>(sleep_duration_seconds: N) -> Self {
        Sleep::from_seconds(sleep_duration_seconds).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enforce_trait_bounds<T: Value>(value: T) -> T {
        value
    }

    #[test]
    fn async_io_implements_desired_traits() {
        enforce_trait_bounds(AsyncIo::pure(Vec::<usize>::new()));
    }
}
