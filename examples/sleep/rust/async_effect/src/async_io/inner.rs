use std::any::Any;
use std::sync::Arc;

use crate::{
    io::BaseIo,
    sleep::{self, Sleep, SleepOrdering},
};

use super::{Callback, Value};

trait DynamicValue: Any + Send + Sync {}

impl<T: Any + Send + Sync> DynamicValue for T {}

trait EffectCallback<T: Value>: Fn() -> Inner<T> + Callback {}

impl<T: Value, F: Fn() -> Inner<T> + Callback> EffectCallback<T> for F {}

trait ValueCallback<T: Value>: Fn(Arc<dyn DynamicValue>) -> Inner<T> + Callback {}

impl<T: Value, F: Fn(Arc<dyn DynamicValue>) -> Inner<T> + Callback> ValueCallback<T> for F {}

trait IoAction: BaseIo<Arc<dyn DynamicValue>> + Callback {}

impl<F: BaseIo<Arc<dyn DynamicValue>> + Callback> IoAction for F {}

#[derive(Clone)]
pub struct DeferredEffect<T: Value> {
    sleep: Sleep,
    next: Arc<dyn EffectCallback<T>>,
}

#[derive(Clone)]
pub struct DeferredIo<T: Value> {
    io: Arc<dyn IoAction>,
    next: Arc<dyn ValueCallback<T>>,
}

#[derive(Clone)]
pub struct DeferredBind<T: Value> {
    value: Arc<dyn DynamicValue>,
    next: Arc<dyn ValueCallback<T>>,
}

/// A pure value
///
/// All effect chains end in values
#[derive(Clone)]
pub struct Pure<T: Value> {
    value: T,
}

#[derive(Clone)]
pub enum Inner<T: Value> {
    Effect(DeferredEffect<T>),
    Io(DeferredIo<T>),
    Bind(DeferredBind<T>),
    Pure(Pure<T>),
}

impl<T: Value> DeferredEffect<T> {
    fn next(self) -> Inner<T> {
        let Self { sleep, next } = self;
        sleep::run(sleep);
        (next)()
    }

    fn concurrently<U: Value>(self, y: Inner<U>) -> Inner<(T, U)> {
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

    fn select<U: Value>(self, y: Inner<U>) -> Inner<ConcurrentOrderInner<T, U>> {
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
    fn of_base_io<F: BaseIo<T> + Callback>(io_effect: F) -> Self {
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

    fn next(self) -> Inner<T> {
        let Self { io, next } = self;
        let value = io();
        (next)(value)
    }

    fn concurrently<U: Value>(self, y: Inner<U>) -> Inner<(T, U)> {
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

    fn select<U: Value>(self, y: Inner<U>) -> Inner<ConcurrentOrderInner<T, U>> {
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
    fn next(self) -> Inner<T> {
        let Self { value, next } = self;
        (next)(value)
    }

    fn concurrently<U: Value>(self, y: Inner<U>) -> Inner<(T, U)> {
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

    fn select<U: Value>(self, y: Inner<U>) -> Inner<ConcurrentOrderInner<T, U>> {
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
    fn new(value: T) -> Self {
        Self { value }
    }

    fn map<U: Value, F: Fn(T) -> U + Value>(self, f: F) -> Pure<U> {
        Pure {
            value: f(self.value),
        }
    }

    fn next(self) -> T {
        let Self { value } = self;
        value
    }

    fn concurrently<U: Value>(self, y: Inner<U>) -> Inner<(T, U)> {
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

    fn select<U: Value>(self, y: Inner<U>) -> Inner<ConcurrentOrderInner<T, U>> {
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
pub enum ConcurrentOrderInner<T: Value, U: Value> {
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
