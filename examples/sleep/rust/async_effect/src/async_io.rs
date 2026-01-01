use std::sync::Arc;
use std::time::Duration;

use crate::{
    io::BaseIo,
    sleep::{self, Sleep},
};

pub struct AsyncIoInner<T> {
    sleep: Option<Sleep>,
    value: T,
}

impl<T> AsyncIoInner<T> {
    pub fn pure(value: T) -> Self {
        Self { sleep: None, value }
    }

    pub fn run(self) -> T {
        let Self { sleep, value } = self;
        if let Some(sleep) = sleep {
            sleep::run(sleep);
        }
        value
    }
}

impl From<Sleep> for AsyncIoInner<()> {
    fn from(sleep: Sleep) -> Self {
        Self {
            sleep: Some(sleep),
            value: (),
        }
    }
}

pub trait AsyncIo<T>: BaseIo<AsyncIoInner<T>> {}

impl<T, F: BaseIo<AsyncIoInner<T>>> AsyncIo<T> for F {}

pub fn from_arc<T, F: AsyncIo<T>>(f: Arc<F>) -> impl AsyncIo<T> + Clone {
    move || (*f)()
}

pub fn arc<T, F: AsyncIo<T>>(f: F) -> impl AsyncIo<T> + Clone {
    from_arc(Arc::new(f))
}

/// A monad lift operation
pub fn of_base_io<T, F: BaseIo<T>>(f: F) -> impl AsyncIo<T> {
    // It would be nice to use [`io::bind`] but it seems to be difficult to do
    // so without compilation errors
    move || {
        let value = f();
        AsyncIoInner::pure(value)
    }
}

pub fn from_sleep(sleep: Sleep) -> impl 'static + AsyncIo<()> {
    move || sleep.into()
}

pub fn sleep(sleep_duration: Duration) -> impl 'static + AsyncIo<()> {
    from_sleep(Sleep::sleep(sleep_duration))
}

pub fn sleep_from_milliseconds<N: Into<u64>>(
    sleep_duration_milliseconds: N,
) -> impl 'static + AsyncIo<()> {
    from_sleep(Sleep::sleep_from_milliseconds(sleep_duration_milliseconds))
}

pub fn sleep_from_seconds<N: Into<u64>>(sleep_duration_seconds: N) -> impl 'static + AsyncIo<()> {
    from_sleep(Sleep::sleep_from_seconds(sleep_duration_seconds))
}

pub fn pure_clone<T: Clone>(value: T) -> impl AsyncIo<T> {
    move || AsyncIoInner::pure(value.clone())
}

pub fn pure_copy<T: Copy>(value: T) -> impl AsyncIo<T> {
    move || AsyncIoInner::pure(value)
}

pub fn pure_ref<T>(value: &T) -> impl AsyncIo<&T> {
    move || AsyncIoInner::pure(value)
}

pub fn bind<T, U, F: AsyncIo<T>, G, H: AsyncIo<U>>(async_io: F, g: G) -> impl AsyncIo<U>
where
    G: Fn(T) -> H,
{
    move || {
        let inner = async_io();
        let value = inner.run();
        g(value)()
    }
}

pub fn block<T, F: AsyncIo<T>>(async_io: F) -> impl BaseIo<T> {
    move || {
        let inner = async_io();
        let value = inner.run();
        value
    }
}

pub fn for_m<T, I, F, H: 'static + AsyncIo<()>>(collection: I, f: F) -> Box<dyn AsyncIo<()>>
where
    I: IntoIterator<Item = T>,
    F: Fn(T) -> H,
{
    let mut effect = Box::new(pure_copy(())) as Box<dyn AsyncIo<()>>;
    for item in collection.into_iter() {
        let next_effect = arc(f(item));
        effect = Box::new(bind(effect, move |_| next_effect.clone()));
    }
    effect
}
