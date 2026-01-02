use std::any::Any;
use std::sync::Arc;
use std::time::Duration;

use crate::{
    io::BaseIo,
    sleep::{self, ConcurrentOrder, Sleep},
};

/// A trait bound on the type parameter of [`AsyncIo`] that ensures that
/// [`AsyncIo`] itself satisfies these traits
pub trait AsyncIoValue: 'static + Clone + Send + Sync {}

impl<T: 'static + Clone + Send + Sync> AsyncIoValue for T {}

#[derive(Clone)]
struct DeferredEffect<T: AsyncIoValue> {
    sleep: Sleep,
    next: Arc<dyn Fn() -> AsyncIoInner<T> + 'static + Send + Sync>,
}

#[derive(Clone)]
struct DeferredValue<T: AsyncIoValue> {
    value: T,
    next: Arc<dyn Fn(T) -> AsyncIoInner<T> + 'static + Send + Sync>,
}

#[derive(Clone)]
struct DeferredIo<T: AsyncIoValue> {
    io: Arc<dyn BaseIo<Arc<dyn Any + Send + Sync>> + 'static + Send + Sync>,
    next: Arc<dyn Fn(Arc<dyn Any>) -> AsyncIoInner<T> + 'static + Send + Sync>,
}

#[derive(Clone)]
enum AsyncIoInner<T: AsyncIoValue> {
    Effect(DeferredEffect<T>),
    Io(DeferredIo<T>),
    Value(DeferredValue<T>),
    None,
}

impl<T: AsyncIoValue> DeferredEffect<T> {
    pub fn next(self) -> AsyncIoInner<T> {
        let Self { sleep, next } = self;
        sleep::run(sleep);
        (next)()
    }

    pub fn concurrently<U: AsyncIoValue>(
        self,
        y: AsyncIoInner<U>,
    ) -> AsyncIoInner<(Option<T>, Option<U>)> {
        match y {
            AsyncIoInner::Effect(effect) => match Sleep::concurrently(self.sleep, effect.sleep) {
                ConcurrentOrder::Equal(sleep) => AsyncIoInner::Effect(DeferredEffect {
                    sleep,
                    next: Arc::new(move || {
                        let first = (self.next.clone())();
                        let second = (effect.next.clone())();
                        AsyncIoInner::concurrently(first, second)
                    }),
                }),
                ConcurrentOrder::SameOrder(first_sleep, second_sleep) => {
                    AsyncIoInner::Effect(DeferredEffect {
                        sleep: first_sleep,
                        next: Arc::new(move || {
                            let first = (self.next.clone())();
                            AsyncIoInner::concurrently(
                                first,
                                AsyncIoInner::Effect(DeferredEffect {
                                    sleep: second_sleep,
                                    next: effect.next.clone(),
                                }),
                            )
                        }),
                    })
                }
                ConcurrentOrder::ReverseOrder(first_sleep, second_sleep) => {
                    AsyncIoInner::Effect(DeferredEffect {
                        sleep: first_sleep,
                        next: Arc::new(move || {
                            let second = (effect.next.clone())();
                            AsyncIoInner::concurrently(
                                AsyncIoInner::Effect(DeferredEffect {
                                    sleep: second_sleep,
                                    next: self.next.clone(),
                                }),
                                second,
                            )
                        }),
                    })
                }
            },
            AsyncIoInner::Io(effect) => AsyncIoInner::Io(DeferredIo {
                io: effect.io,
                next: Arc::new(move |io_value| {
                    let second = (effect.next.clone())(io_value);
                    AsyncIoInner::concurrently(AsyncIoInner::Effect(self.clone()), second)
                }),
            }),
            AsyncIoInner::Value(effect) => AsyncIoInner::Value(DeferredValue {
                value: (None, Some(effect.value.clone())),
                next: Arc::new(move |(_, second_value)| {
                    let second = (effect.next.clone())(second_value.unwrap());
                    AsyncIoInner::concurrently(AsyncIoInner::Effect(self.clone()), second)
                }),
            }),
            AsyncIoInner::None => AsyncIoInner::Effect(self).map(|first| (Some(first), None)),
        }
    }
}

impl<T: AsyncIoValue> DeferredValue<T> {
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

    pub fn concurrently<U: AsyncIoValue>(
        self,
        y: AsyncIoInner<U>,
    ) -> AsyncIoInner<(Option<T>, Option<U>)> {
        match y {
            AsyncIoInner::Effect(effect) => AsyncIoInner::Value(DeferredValue {
                value: (Some(self.value.clone()), None),
                next: Arc::new(move |(first_value, _)| {
                    let first = (self.next.clone())(first_value.unwrap());
                    AsyncIoInner::concurrently(first, AsyncIoInner::Effect(effect.clone()))
                }),
            }),
            AsyncIoInner::Io(effect) => AsyncIoInner::Value(DeferredValue {
                value: (Some(self.value.clone()), None),
                next: Arc::new(move |(first_value, _)| {
                    let first = (self.next.clone())(first_value.unwrap());
                    AsyncIoInner::concurrently(first, AsyncIoInner::Io(effect.clone()))
                }),
            }),
            AsyncIoInner::Value(effect) => AsyncIoInner::Value(DeferredValue {
                value: (Some(self.value.clone()), Some(effect.value.clone())),
                next: Arc::new(move |(first_value, second_value)| {
                    let first = (self.next.clone())(first_value.unwrap());
                    let second = (effect.next.clone())(second_value.unwrap());
                    AsyncIoInner::concurrently(first, second)
                }),
            }),
            AsyncIoInner::None => AsyncIoInner::Value(self).map(|first| (Some(first), None)),
        }
    }
}

impl<T: AsyncIoValue> DeferredIo<T> {
    pub fn next(self) -> AsyncIoInner<T> {
        let Self { io, next } = self;
        let value = io();
        (next)(value)
    }

    pub fn concurrently<U: AsyncIoValue>(
        self,
        y: AsyncIoInner<U>,
    ) -> AsyncIoInner<(Option<T>, Option<U>)> {
        match y {
            AsyncIoInner::Effect(effect) => AsyncIoInner::Io(DeferredIo {
                io: self.io,
                next: Arc::new(move |io_value| {
                    let first = (self.next.clone())(io_value);
                    AsyncIoInner::concurrently(first, AsyncIoInner::Effect(effect.clone()))
                }),
            }),
            AsyncIoInner::Io(effect) => AsyncIoInner::Io(DeferredIo {
                io: Arc::new(move || {
                    let first_value = (self.io.clone())();
                    let second_value = (effect.io.clone())();
                    Arc::new((first_value, second_value))
                }),
                next: Arc::new(move |io_arc_pair| {
                    let io_any_pair: &dyn Any = &*io_arc_pair;
                    match io_any_pair.downcast_ref::<(Arc<dyn Any>, Arc<dyn Any>)>() {
                        Some((first_io_arc_value, second_io_arc_value)) => {
                            let first = (self.next.clone())(first_io_arc_value.clone());
                            let second = (effect.next.clone())(second_io_arc_value.clone());
                            AsyncIoInner::concurrently(first, second)
                        }
                        None => {
                            unreachable!();
                        }
                    }
                }),
            }),
            AsyncIoInner::Value(effect) => AsyncIoInner::Io(DeferredIo {
                io: self.io,
                next: Arc::new(move |io_value| {
                    let first = (self.next.clone())(io_value);
                    AsyncIoInner::concurrently(first, AsyncIoInner::Value(effect.clone()))
                }),
            }),
            AsyncIoInner::None => AsyncIoInner::Io(self).map(|first| (Some(first), None)),
        }
    }
}

impl<T: AsyncIoValue> DeferredIo<T> {
    pub fn of_base_io<F: BaseIo<T> + 'static + Send + Sync>(io_effect: F) -> Self {
        Self {
            io: Arc::new(move || {
                let value = io_effect();
                Arc::new(value)
            }),
            next: Arc::new(|io_arc_value| {
                let io_any_value: &dyn Any = &*io_arc_value;
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

impl<T: AsyncIoValue> AsyncIoInner<T> {
    pub fn pure(value: T) -> Self {
        Self::Value(DeferredValue::pure(value))
    }

    pub fn of_base_io<F: BaseIo<T> + 'static + Send + Sync>(io_effect: F) -> Self {
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

    pub fn bind<U: AsyncIoValue, F: Fn(T) -> AsyncIoInner<U> + AsyncIoValue>(
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

    pub fn map<U: AsyncIoValue, F: Fn(T) -> U + AsyncIoValue>(self, f: F) -> AsyncIoInner<U> {
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

    pub fn for_m<U: AsyncIoValue, I, F>(collection: I, f: F) -> Self
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

    pub fn concurrently<U: AsyncIoValue>(
        x: Self,
        y: AsyncIoInner<U>,
    ) -> AsyncIoInner<(Option<T>, Option<U>)> {
        match x {
            Self::Effect(effect) => effect.concurrently(y),
            Self::Io(effect) => effect.concurrently(y),
            Self::Value(effect) => effect.concurrently(y),
            Self::None => y.map(|second| (None, Some(second))),
        }
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
pub struct AsyncIo<T: AsyncIoValue>(AsyncIoInner<T>);

impl<T: AsyncIoValue> AsyncIo<T> {
    pub fn pure(value: T) -> Self {
        Self(AsyncIoInner::pure(value))
    }

    pub fn of_base_io<F: BaseIo<T> + 'static + Send + Sync>(io_effect: F) -> Self {
        Self(AsyncIoInner::of_base_io(io_effect))
    }

    pub fn bind<U: AsyncIoValue, F: Fn(T) -> AsyncIo<U> + AsyncIoValue>(self, f: F) -> AsyncIo<U> {
        AsyncIo(self.0.bind(move |value| f(value).0))
    }

    pub fn map<U: AsyncIoValue, F: Fn(T) -> U + AsyncIoValue>(self, f: F) -> AsyncIo<U> {
        AsyncIo(self.0.map(f))
    }

    pub fn block(self) -> impl BaseIo<Option<T>> + 'static {
        self.0.block()
    }

    pub fn for_m<U: AsyncIoValue, I, F>(collection: I, f: F) -> Self
    where
        I: IntoIterator<Item = U>,
        F: Fn(U) -> Self,
    {
        Self(AsyncIoInner::for_m(collection, move |value| f(value).0))
    }

    pub fn concurrently<U: AsyncIoValue>(
        x: Self,
        y: AsyncIo<U>,
    ) -> AsyncIo<(Option<T>, Option<U>)> {
        AsyncIo(AsyncIoInner::concurrently(x.0, y.0))
    }

    pub fn concurrently_all<I: IntoIterator<Item = Self>>(
        collection: I,
    ) -> AsyncIo<Vec<Option<T>>> {
        let mut action: AsyncIo<Vec<Option<T>>> = AsyncIo::pure(Vec::new());
        for item in collection {
            action = AsyncIo::concurrently(action, item).map(|(v_option, value)| {
                let mut v = v_option.unwrap_or_default();
                v.push(value);
                v
            });
        }
        action
    }
}

impl From<Sleep> for AsyncIo<()> {
    fn from(sleep: Sleep) -> Self {
        Self(sleep.into())
    }
}

impl AsyncIo<()> {
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
