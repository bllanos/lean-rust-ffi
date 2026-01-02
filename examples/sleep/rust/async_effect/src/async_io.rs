use std::any::Any;
use std::sync::Arc;
use std::time::Duration;

use crate::{
    io::BaseIo,
    sleep::{self, Sleep},
};

#[derive(Clone)]
struct DeferredEffect<T: 'static + Clone> {
    sleep: Sleep,
    next: Arc<dyn Fn() -> AsyncIoInner<T> + 'static>,
}

#[derive(Clone)]
struct DeferredValue<T: 'static + Clone> {
    value: T,
    next: Arc<dyn Fn(T) -> AsyncIoInner<T> + 'static>,
}

#[derive(Clone)]
struct DeferredIo<T: 'static + Clone> {
    io: Arc<dyn BaseIo<Box<dyn Any>> + 'static>,
    next: Arc<dyn Fn(Box<dyn Any>) -> AsyncIoInner<T> + 'static>,
}

#[derive(Clone)]
enum AsyncIoInner<T: 'static + Clone> {
    Effect(DeferredEffect<T>),
    Io(DeferredIo<T>),
    Value(DeferredValue<T>),
    None,
}

impl<T: 'static + Clone> DeferredEffect<T> {
    pub fn next(self) -> AsyncIoInner<T> {
        let Self { sleep, next } = self;
        sleep::run(sleep);
        (next)()
    }
}

impl<T: 'static + Clone> DeferredValue<T> {
    pub fn pure(value: T) -> Self {
        Self {
            value,
            next: Arc::new(|_| AsyncIoInner::None),
        }
    }

    pub fn next(self) -> AsyncIoInner<T> {
        let Self { value, next } = self;
        (next)(value)
    }
}

impl<T: 'static + Clone> DeferredIo<T> {
    pub fn next(self) -> AsyncIoInner<T> {
        let Self { io, next } = self;
        let value = io();
        (next)(value)
    }
}

impl<T: 'static + Clone> DeferredIo<T> {
    pub fn of_base_io<F: BaseIo<T> + 'static>(io_effect: F) -> Self {
        Self {
            io: Arc::new(move || {
                let value = io_effect();
                Box::new(value)
            }),
            next: Arc::new(|io_boxed_value| {
                let io_any_value: &dyn Any = &*io_boxed_value;
                match io_any_value.downcast_ref::<T>() {
                    Some(io_value) => AsyncIoInner::pure(io_value.clone()),
                    None => {
                        unreachable!();
                    }
                }
            }),
        }
    }
}

impl<T: 'static + Clone> AsyncIoInner<T> {
    pub fn pure(value: T) -> Self {
        Self::Value(DeferredValue::pure(value))
    }

    pub fn of_base_io<F: BaseIo<T> + 'static>(io_effect: F) -> Self {
        Self::Io(DeferredIo::of_base_io(io_effect))
    }

    pub fn union(self, other: Self) -> Self {
        match self {
            Self::Effect(effect) => Self::Effect(DeferredEffect {
                sleep: effect.sleep,
                next: Arc::new(move || (effect.next.clone())().union(other.clone())),
            }),
            Self::Io(effect) => Self::Io(DeferredIo {
                io: effect.io,
                next: Arc::new(move |io_value| {
                    (effect.next.clone())(io_value).union(other.clone())
                }),
            }),
            Self::Value(effect) => Self::Value(DeferredValue {
                value: effect.value,
                next: Arc::new(move |value| (effect.next.clone())(value).union(other.clone())),
            }),
            Self::None => other,
        }
    }

    pub fn bind<U: 'static + Clone, F: Fn(T) -> AsyncIoInner<U> + 'static + Clone>(
        self,
        f: F,
    ) -> AsyncIoInner<U> {
        match self {
            Self::Effect(effect) => AsyncIoInner::Effect(DeferredEffect {
                sleep: effect.sleep,
                next: Arc::new(move || (effect.next.clone())().bind(f.clone())),
            }),
            Self::Io(effect) => AsyncIoInner::Io(DeferredIo {
                io: effect.io,
                next: Arc::new(move |io_value| (effect.next.clone())(io_value).bind(f.clone())),
            }),
            Self::Value(effect) => ((f.clone())(effect.value.clone()))
                .union(((effect.next.clone())(effect.value.clone())).bind(f.clone())),
            Self::None => AsyncIoInner::None,
        }
    }

    pub fn map<U: 'static + Clone, F: Fn(T) -> U + 'static + Clone>(self, f: F) -> AsyncIoInner<U> {
        match self {
            Self::Effect(effect) => AsyncIoInner::Effect(DeferredEffect {
                sleep: effect.sleep,
                next: Arc::new(move || (effect.next.clone())().map(f.clone())),
            }),
            Self::Io(effect) => AsyncIoInner::Io(DeferredIo {
                io: effect.io,
                next: Arc::new(move |io_value| (effect.next.clone())(io_value).map(f.clone())),
            }),
            Self::Value(effect) => AsyncIoInner::Value(DeferredValue {
                value: (f.clone())(effect.value.clone()),
                next: Arc::new(move |_| (effect.next.clone())(effect.value.clone()).map(f.clone())),
            }),
            Self::None => AsyncIoInner::None,
        }
    }

    pub fn next(self) -> Option<Self> {
        match self {
            Self::Effect(effect) => Some(effect.next()),
            Self::Io(effect) => Some(effect.next()),
            Self::Value(effect) => Some(effect.next()),
            Self::None => None,
        }
    }

    pub fn block(self) -> impl BaseIo<Option<T>> + 'static {
        move || {
            let mut instance = self.clone();
            let mut value = match &instance {
                Self::Effect(_) => None,
                Self::Io(_) => None,
                Self::Value(effect) => Some(effect.value.clone()),
                Self::None => None,
            };
            while let Some(next) = instance.clone().next() {
                value = match &next {
                    Self::Effect(_) => None,
                    Self::Io(_) => None,
                    Self::Value(effect) => Some(effect.value.clone()),
                    Self::None => None,
                };
                instance = next;
            }
            value
        }
    }

    pub fn for_m<U: 'static + Clone, I, F>(collection: I, f: F) -> Self
    where
        I: IntoIterator<Item = U>,
        F: Fn(U) -> Self,
    {
        let mut effect = None;
        for item in collection.into_iter() {
            let next_effect = f(item);
            effect = match effect {
                None => Some(next_effect),
                Some(effect) => Some(Self::bind(effect, move |_| next_effect.clone())),
            }
        }
        effect.unwrap_or(Self::None)
    }
}

impl From<Sleep> for DeferredEffect<()> {
    fn from(sleep: Sleep) -> Self {
        Self {
            sleep,
            next: Arc::new(|| AsyncIoInner::pure(())),
        }
    }
}

impl From<Sleep> for AsyncIoInner<()> {
    fn from(sleep: Sleep) -> Self {
        Self::Effect(sleep.into())
    }
}

#[derive(Clone)]
pub struct AsyncIo<T: 'static + Clone>(AsyncIoInner<T>);

impl<T: 'static + Clone> AsyncIo<T> {
    pub fn pure(value: T) -> Self {
        Self(AsyncIoInner::pure(value))
    }

    pub fn of_base_io<F: BaseIo<T> + 'static>(io_effect: F) -> Self {
        Self(AsyncIoInner::of_base_io(io_effect))
    }

    pub fn bind<U: 'static + Clone, F: Fn(T) -> AsyncIo<U> + 'static + Clone>(
        self,
        f: F,
    ) -> AsyncIo<U> {
        AsyncIo(self.0.bind(move |value| f(value).0))
    }

    pub fn map<U: 'static + Clone, F: Fn(T) -> U + 'static + Clone>(self, f: F) -> AsyncIo<U> {
        AsyncIo(self.0.map(f))
    }

    pub fn block(self) -> impl BaseIo<Option<T>> + 'static {
        self.0.block()
    }

    pub fn for_m<U: 'static + Clone, I, F>(collection: I, f: F) -> Self
    where
        I: IntoIterator<Item = U>,
        F: Fn(U) -> Self,
    {
        Self(AsyncIoInner::for_m(collection, move |value| f(value).0))
    }
}

impl From<Sleep> for AsyncIo<()> {
    fn from(sleep: Sleep) -> Self {
        Self(sleep.into())
    }
}

impl AsyncIo<()> {
    pub fn sleep(sleep_duration: Duration) -> Self {
        Sleep::sleep(sleep_duration).into()
    }

    pub fn sleep_from_milliseconds<N: Into<u64>>(sleep_duration_milliseconds: N) -> Self {
        Sleep::sleep_from_milliseconds(sleep_duration_milliseconds).into()
    }

    pub fn sleep_from_seconds<N: Into<u64>>(sleep_duration_seconds: N) -> Self {
        Sleep::sleep_from_seconds(sleep_duration_seconds).into()
    }
}
