use std::any::Any;
use std::sync::Arc;
use std::time::Duration;

use crate::{
    io::BaseIo,
    sleep::{self, Sleep, SleepOrdering},
};

/// A trait bound on the type parameter of [`AsyncIo`] that ensures that
/// [`AsyncIo`] itself satisfies these traits
pub trait Value: 'static + Any + Clone + Send + Sync {}

impl<T: 'static + Any + Clone + Send + Sync> Value for T {}

trait DynamicValue: Any + Send + Sync {}

impl<T: Any + Send + Sync> DynamicValue for T {}

pub trait Callback: 'static + Send + Sync {}

impl<F: 'static + Send + Sync> Callback for F {}

trait EffectCallback<T: Value>: Fn() -> Inner<T> + Callback {}

impl<T: Value, F: Fn() -> Inner<T> + Callback> EffectCallback<T> for F {}

trait ValueCallback<T: Value>: Fn(Arc<dyn DynamicValue>) -> Inner<T> + Callback {}

impl<T: Value, F: Fn(Arc<dyn DynamicValue>) -> Inner<T> + Callback> ValueCallback<T> for F {}

trait IoAction: BaseIo<Arc<dyn DynamicValue>> + Callback {}

impl<F: BaseIo<Arc<dyn DynamicValue>> + Callback> IoAction for F {}

#[derive(Clone)]
struct DeferredEffect<T: Value> {
    sleep: Sleep,
    next: Arc<dyn EffectCallback<T>>,
}

#[derive(Clone)]
struct DeferredIo<T: Value> {
    io: Arc<dyn IoAction>,
    next: Arc<dyn ValueCallback<T>>,
}

#[derive(Clone)]
struct DeferredBind<T: Value> {
    value: Arc<dyn DynamicValue>,
    next: Arc<dyn ValueCallback<T>>,
}

/// A pure value
///
/// All effect chains end in values
#[derive(Clone)]
struct Pure<T: Value> {
    value: T,
}

#[derive(Clone)]
enum Inner<T: Value> {
    Effect(DeferredEffect<T>),
    Io(DeferredIo<T>),
    Bind(DeferredBind<T>),
    Pure(Pure<T>),
}

impl<T: Value> DeferredEffect<T> {
    pub fn next(self) -> Inner<T> {
        let Self { sleep, next } = self;
        sleep::run(sleep);
        (next)()
    }

    pub fn concurrently<U: Value>(self, y: Inner<U>) -> Inner<(T, U)> {
        match y {
            Inner::Effect(effect) => match Sleep::concurrently(self.sleep, effect.sleep) {
                SleepOrdering::Equal(sleep) => Inner::Effect(DeferredEffect {
                    sleep,
                    next: Arc::new(move || {
                        let first = (self.next.clone())();
                        let second = (effect.next.clone())();
                        Inner::concurrently(first, second)
                    }),
                }),
                SleepOrdering::SameOrder(first_sleep, second_sleep) => {
                    Inner::Effect(DeferredEffect {
                        sleep: first_sleep,
                        next: Arc::new(move || {
                            let first = (self.next.clone())();
                            Inner::concurrently(
                                first,
                                Inner::Effect(DeferredEffect {
                                    sleep: second_sleep,
                                    next: effect.next.clone(),
                                }),
                            )
                        }),
                    })
                }
                SleepOrdering::ReverseOrder(first_sleep, second_sleep) => {
                    Inner::Effect(DeferredEffect {
                        sleep: first_sleep,
                        next: Arc::new(move || {
                            let second = (effect.next.clone())();
                            Inner::concurrently(
                                Inner::Effect(DeferredEffect {
                                    sleep: second_sleep,
                                    next: self.next.clone(),
                                }),
                                second,
                            )
                        }),
                    })
                }
            },
            Inner::Io(effect) => Inner::Io(DeferredIo {
                io: effect.io,
                next: Arc::new(move |io_value| {
                    let second = (effect.next.clone())(io_value);
                    Inner::concurrently(Inner::Effect(self.clone()), second)
                }),
            }),
            Inner::Bind(effect) => Inner::Bind(DeferredBind {
                value: effect.value,
                next: Arc::new(move |second_value| {
                    let second = (effect.next.clone())(second_value);
                    Inner::concurrently(Inner::Effect(self.clone()), second)
                }),
            }),
            Inner::Pure(effect) => Inner::Effect(DeferredEffect {
                sleep: self.sleep,
                next: Arc::new(move || {
                    let first = (self.next.clone())();
                    Inner::concurrently(first, Inner::Pure(effect.clone()))
                }),
            }),
        }
    }

    pub fn select<U: Value>(self, y: Inner<U>) -> Inner<ConcurrentOrderInner<T, U>> {
        match y {
            Inner::Effect(effect) => match Sleep::concurrently(self.sleep, effect.sleep) {
                SleepOrdering::Equal(sleep) => Inner::Effect(DeferredEffect {
                    sleep,
                    next: Arc::new(move || {
                        let first = (self.next.clone())();
                        let second = (effect.next.clone())();
                        Inner::select(first, second)
                    }),
                }),
                SleepOrdering::SameOrder(first_sleep, second_sleep) => {
                    Inner::Effect(DeferredEffect {
                        sleep: first_sleep,
                        next: Arc::new(move || {
                            let first = (self.next.clone())();
                            Inner::select(
                                first,
                                Inner::Effect(DeferredEffect {
                                    sleep: second_sleep,
                                    next: effect.next.clone(),
                                }),
                            )
                        }),
                    })
                }
                SleepOrdering::ReverseOrder(first_sleep, second_sleep) => {
                    Inner::Effect(DeferredEffect {
                        sleep: first_sleep,
                        next: Arc::new(move || {
                            let second = (effect.next.clone())();
                            Inner::select(
                                Inner::Effect(DeferredEffect {
                                    sleep: second_sleep,
                                    next: self.next.clone(),
                                }),
                                second,
                            )
                        }),
                    })
                }
            },
            Inner::Io(effect) => Inner::Io(DeferredIo {
                io: effect.io,
                next: Arc::new(move |io_value| {
                    let second = (effect.next.clone())(io_value);
                    Inner::select(Inner::Effect(self.clone()), second)
                }),
            }),
            Inner::Bind(effect) => Inner::Bind(DeferredBind {
                value: effect.value,
                next: Arc::new(move |second_value| {
                    let second = (effect.next.clone())(second_value);
                    Inner::select(Inner::Effect(self.clone()), second)
                }),
            }),
            Inner::Pure(effect) => Inner::Pure(Pure::new(ConcurrentOrderInner::Second(
                Inner::Effect(self),
                effect.value,
            ))),
        }
    }
}

impl<T: Value> DeferredIo<T> {
    pub fn of_base_io<F: BaseIo<T> + Callback>(io_effect: F) -> Self {
        Self {
            io: Arc::new(move || {
                let value = io_effect();
                Arc::new(value)
            }),
            next: Arc::new(|io_arc_value| {
                let io_any_value: &dyn Any = &*io_arc_value;
                match io_any_value.downcast_ref::<T>() {
                    Some(io_value) => Inner::pure(io_value.clone()),
                    None => {
                        unreachable!("bad dynamic cast of IO effect's return value");
                    }
                }
            }),
        }
    }

    pub fn next(self) -> Inner<T> {
        let Self { io, next } = self;
        let value = io();
        (next)(value)
    }

    pub fn concurrently<U: Value>(self, y: Inner<U>) -> Inner<(T, U)> {
        match y {
            Inner::Effect(effect) => Inner::Io(DeferredIo {
                io: self.io,
                next: Arc::new(move |io_value| {
                    let first = (self.next.clone())(io_value);
                    Inner::concurrently(first, Inner::Effect(effect.clone()))
                }),
            }),
            Inner::Io(effect) => Inner::Io(DeferredIo {
                io: Arc::new(move || {
                    let first_value = (self.io.clone())();
                    let second_value = (effect.io.clone())();
                    Arc::new((first_value, second_value))
                }),
                next: Arc::new(move |io_arc_pair| {
                    let io_any_pair: &dyn Any = &*io_arc_pair;
                    match io_any_pair
                        .downcast_ref::<(Arc<dyn DynamicValue>, Arc<dyn DynamicValue>)>()
                    {
                        Some((first_io_arc_value, second_io_arc_value)) => {
                            let first = (self.next.clone())(first_io_arc_value.clone());
                            let second = (effect.next.clone())(second_io_arc_value.clone());
                            Inner::concurrently(first, second)
                        }
                        None => {
                            unreachable!(
                                "bad dynamic cast of concurrent IO effects' return values"
                            );
                        }
                    }
                }),
            }),
            Inner::Bind(effect) => Inner::Io(DeferredIo {
                io: self.io,
                next: Arc::new(move |io_value| {
                    let first = (self.next.clone())(io_value);
                    Inner::concurrently(first, Inner::Bind(effect.clone()))
                }),
            }),
            Inner::Pure(effect) => Inner::Io(DeferredIo {
                io: self.io,
                next: Arc::new(move |io_value| {
                    let first = (self.next.clone())(io_value);
                    Inner::concurrently(first, Inner::Pure(effect.clone()))
                }),
            }),
        }
    }

    pub fn select<U: Value>(self, y: Inner<U>) -> Inner<ConcurrentOrderInner<T, U>> {
        match y {
            Inner::Effect(effect) => Inner::Io(DeferredIo {
                io: self.io,
                next: Arc::new(move |io_value| {
                    let first = (self.next.clone())(io_value);
                    Inner::select(first, Inner::Effect(effect.clone()))
                }),
            }),
            Inner::Io(effect) => Inner::Io(DeferredIo {
                io: Arc::new(move || {
                    let first_value = (self.io.clone())();
                    let second_value = (effect.io.clone())();
                    Arc::new((first_value, second_value))
                }),
                next: Arc::new(move |io_arc_pair| {
                    let io_any_pair: &dyn Any = &*io_arc_pair;
                    match io_any_pair
                        .downcast_ref::<(Arc<dyn DynamicValue>, Arc<dyn DynamicValue>)>()
                    {
                        Some((first_io_arc_value, second_io_arc_value)) => {
                            let first = (self.next.clone())(first_io_arc_value.clone());
                            let second = (effect.next.clone())(second_io_arc_value.clone());
                            Inner::select(first, second)
                        }
                        None => {
                            unreachable!("bad dynamic cast of select IO effects' return values");
                        }
                    }
                }),
            }),
            Inner::Bind(effect) => Inner::Io(DeferredIo {
                io: self.io,
                next: Arc::new(move |io_value| {
                    let first = (self.next.clone())(io_value);
                    Inner::select(first, Inner::Bind(effect.clone()))
                }),
            }),
            Inner::Pure(effect) => Inner::Io(DeferredIo {
                io: self.io,
                next: Arc::new(move |io_value| {
                    let first = (self.next.clone())(io_value);
                    Inner::select(first, Inner::Pure(effect.clone()))
                }),
            }),
        }
    }
}

impl<T: Value> DeferredBind<T> {
    pub fn next(self) -> Inner<T> {
        let Self { value, next } = self;
        (next)(value)
    }

    pub fn concurrently<U: Value>(self, y: Inner<U>) -> Inner<(T, U)> {
        match y {
            Inner::Effect(effect) => Inner::Bind(DeferredBind {
                value: self.value,
                next: Arc::new(move |value| {
                    let first = (self.next.clone())(value);
                    Inner::concurrently(first, Inner::Effect(effect.clone()))
                }),
            }),
            Inner::Io(effect) => Inner::Io(DeferredIo {
                io: effect.io,
                next: Arc::new(move |io_value| {
                    let second = (effect.next.clone())(io_value);
                    Inner::concurrently(Inner::Bind(self.clone()), second)
                }),
            }),
            Inner::Bind(effect) => Inner::Bind(DeferredBind {
                value: Arc::new((self.value, effect.value)),
                next: Arc::new(move |value_arc_pair| {
                    let value_arc_pair: &dyn Any = &*value_arc_pair;
                    match value_arc_pair
                        .downcast_ref::<(Arc<dyn DynamicValue>, Arc<dyn DynamicValue>)>()
                    {
                        Some((first_arc_value, second_arc_value)) => {
                            let first = (self.next.clone())(first_arc_value.clone());
                            let second = (effect.next.clone())(second_arc_value.clone());
                            Inner::concurrently(first, second)
                        }
                        None => {
                            unreachable!("bad dynamic cast of concurrent bind effects' values");
                        }
                    }
                }),
            }),
            Inner::Pure(effect) => Inner::Bind(DeferredBind {
                value: self.value,
                next: Arc::new(move |value| {
                    let first = (self.next.clone())(value);
                    Inner::concurrently(first, Inner::Pure(effect.clone()))
                }),
            }),
        }
    }

    pub fn select<U: Value>(self, y: Inner<U>) -> Inner<ConcurrentOrderInner<T, U>> {
        match y {
            Inner::Effect(effect) => Inner::Bind(DeferredBind {
                value: self.value,
                next: Arc::new(move |value| {
                    let first = (self.next.clone())(value);
                    Inner::select(first, Inner::Effect(effect.clone()))
                }),
            }),
            Inner::Io(effect) => Inner::Io(DeferredIo {
                io: effect.io,
                next: Arc::new(move |io_value| {
                    let second = (effect.next.clone())(io_value);
                    Inner::select(Inner::Bind(self.clone()), second)
                }),
            }),
            Inner::Bind(effect) => Inner::Bind(DeferredBind {
                value: Arc::new((self.value, effect.value)),
                next: Arc::new(move |value_arc_pair| {
                    let value_arc_pair: &dyn Any = &*value_arc_pair;
                    match value_arc_pair
                        .downcast_ref::<(Arc<dyn DynamicValue>, Arc<dyn DynamicValue>)>()
                    {
                        Some((first_arc_value, second_arc_value)) => {
                            let first = (self.next.clone())(first_arc_value.clone());
                            let second = (effect.next.clone())(second_arc_value.clone());
                            Inner::select(first, second)
                        }
                        None => {
                            unreachable!("bad dynamic cast of select bind effects' values");
                        }
                    }
                }),
            }),
            Inner::Pure(effect) => Inner::Bind(DeferredBind {
                value: self.value,
                next: Arc::new(move |value| {
                    let first = (self.next.clone())(value);
                    Inner::select(first, Inner::Pure(effect.clone()))
                }),
            }),
        }
    }
}

impl<T: Value> Pure<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }

    pub fn map<U: Value, F: Fn(T) -> U + Value>(self, f: F) -> Pure<U> {
        Pure {
            value: f(self.value),
        }
    }

    pub fn next(self) -> T {
        let Self { value } = self;
        value
    }

    pub fn concurrently<U: Value>(self, y: Inner<U>) -> Inner<(T, U)> {
        match y {
            Inner::Effect(effect) => Inner::Effect(DeferredEffect {
                sleep: effect.sleep,
                next: Arc::new(move || {
                    let second = (effect.next.clone())();
                    Inner::concurrently(Inner::Pure(self.clone()), second)
                }),
            }),
            Inner::Io(effect) => Inner::Io(DeferredIo {
                io: effect.io,
                next: Arc::new(move |io_value| {
                    let second = (effect.next.clone())(io_value);
                    Inner::concurrently(Inner::Pure(self.clone()), second)
                }),
            }),
            Inner::Bind(effect) => Inner::Bind(DeferredBind {
                value: effect.value,
                next: Arc::new(move |value| {
                    let second = (effect.next.clone())(value);
                    Inner::concurrently(Inner::Pure(self.clone()), second)
                }),
            }),
            Inner::Pure(effect) => Inner::Pure(Pure::new((self.value, effect.value))),
        }
    }

    pub fn select<U: Value>(self, y: Inner<U>) -> Inner<ConcurrentOrderInner<T, U>> {
        match y {
            Inner::Effect(_) => Inner::Pure(Pure::new(ConcurrentOrderInner::First(self.value, y))),
            Inner::Io(effect) => Inner::Io(DeferredIo {
                io: effect.io,
                next: Arc::new(move |io_value| {
                    let second = (effect.next.clone())(io_value);
                    Inner::select(Inner::Pure(self.clone()), second)
                }),
            }),
            Inner::Bind(effect) => Inner::Bind(DeferredBind {
                value: effect.value,
                next: Arc::new(move |value| {
                    let second = (effect.next.clone())(value);
                    Inner::select(Inner::Pure(self.clone()), second)
                }),
            }),
            Inner::Pure(effect) => Inner::Pure(Pure::new(ConcurrentOrderInner::Both(
                self.value,
                effect.value,
            ))),
        }
    }
}

enum Next<T: Value> {
    More(Inner<T>),
    End(T),
}

#[derive(Clone)]
enum ConcurrentOrderInner<T: Value, U: Value> {
    First(T, Inner<U>),
    Both(T, U),
    Second(Inner<T>, U),
}

impl<T: Value> Inner<T> {
    pub fn pure(value: T) -> Self {
        Self::Pure(Pure::new(value))
    }

    pub fn of_base_io<F: BaseIo<T> + Callback>(io_effect: F) -> Self {
        Self::Io(DeferredIo::of_base_io(io_effect))
    }

    pub fn bind<U: Value, F: Fn(T) -> Inner<U> + Value>(self, f: F) -> Inner<U> {
        match self {
            Self::Effect(effect) => Inner::Effect(DeferredEffect {
                sleep: effect.sleep,
                next: Arc::new(move || (effect.next.clone())().bind(f.clone())),
            }),
            Self::Io(effect) => Inner::Io(DeferredIo {
                io: effect.io,
                next: Arc::new(move |io_value| (effect.next.clone())(io_value).bind(f.clone())),
            }),
            Self::Bind(effect) => Inner::Bind(DeferredBind {
                value: effect.value,
                next: Arc::new(move |value| (effect.next.clone())(value).bind(f.clone())),
            }),
            Self::Pure(effect) => Inner::Bind(DeferredBind {
                value: Arc::new(effect.value),
                next: Arc::new(move |arc_value| {
                    let any_value: &dyn Any = &*arc_value;
                    match any_value.downcast_ref::<T>() {
                        Some(value) => (f.clone())(value.clone()),
                        None => {
                            unreachable!("bad dynamic cast of bind effect's value");
                        }
                    }
                }),
            }),
        }
    }

    pub fn map<U: Value, F: Fn(T) -> U + Value>(self, f: F) -> Inner<U> {
        match self {
            Self::Effect(effect) => Inner::Effect(DeferredEffect {
                sleep: effect.sleep,
                next: Arc::new(move || (effect.next.clone())().map(f.clone())),
            }),
            Self::Io(effect) => Inner::Io(DeferredIo {
                io: effect.io,
                next: Arc::new(move |io_value| (effect.next.clone())(io_value).map(f.clone())),
            }),
            Self::Bind(effect) => Inner::Bind(DeferredBind {
                value: effect.value,
                next: Arc::new(move |value| (effect.next.clone())(value).map(f.clone())),
            }),
            Self::Pure(effect) => Inner::Pure(effect.map(f)),
        }
    }

    fn next(self) -> Next<T> {
        match self {
            Self::Effect(effect) => Next::More(effect.next()),
            Self::Io(effect) => Next::More(effect.next()),
            Self::Bind(effect) => Next::More(effect.next()),
            Self::Pure(effect) => Next::End(effect.next()),
        }
    }

    pub fn block_immediate(mut self) -> T {
        loop {
            match self.clone().next() {
                Next::More(more) => self = more,
                Next::End(value) => break value,
            }
        }
    }

    pub fn block(self) -> impl BaseIo<T> + 'static {
        move || self.clone().block_immediate()
    }

    pub fn concurrently<U: Value>(x: Self, y: Inner<U>) -> Inner<(T, U)> {
        match x {
            Self::Effect(effect) => effect.concurrently(y),
            Self::Io(effect) => effect.concurrently(y),
            Self::Bind(effect) => effect.concurrently(y),
            Self::Pure(effect) => effect.concurrently(y),
        }
    }

    pub fn select<U: Value>(x: Self, y: Inner<U>) -> Inner<ConcurrentOrderInner<T, U>> {
        match x {
            Self::Effect(effect) => effect.select(y),
            Self::Io(effect) => effect.select(y),
            Self::Bind(effect) => effect.select(y),
            Self::Pure(effect) => effect.select(y),
        }
    }
}

impl Inner<()> {
    pub fn for_m<U: Value, I, F>(collection: I, f: F) -> Self
    where
        I: IntoIterator<Item = U>,
        F: Fn(U) -> Self + Value,
    {
        collection
            .into_iter()
            .fold(Self::pure(()), move |accumulator, item| {
                let f = f.clone();
                Self::bind(accumulator, move |_| (f.clone())(item.clone()))
            })
    }
}

impl From<Sleep> for DeferredEffect<()> {
    fn from(sleep: Sleep) -> Self {
        Self {
            sleep,
            next: Arc::new(|| Inner::pure(())),
        }
    }
}

impl From<Sleep> for Inner<()> {
    fn from(sleep: Sleep) -> Self {
        Self::Effect(sleep.into())
    }
}

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
