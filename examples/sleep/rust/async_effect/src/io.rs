use std::process::ExitCode;
use std::sync::Arc;
use std::time::Instant;

/// An IO monad is a state monad
///
/// The state is an implied unit type argument and part of the return value,
/// representing the world. It is equivalent to Lean's `IO.RealWorld`, which is
/// erased when Lean code is compiled. In other words, `Fn() -> T` should be
/// used as though it is `Fn(IoRealWorld) -> (IoRealWorld, T)` for some
/// non-clonable type `IoRealWorld`.
pub trait BaseIo<T>: Fn() -> T {}

impl<T, F: Fn() -> T> BaseIo<T> for F {}

pub fn from_arc<T, F: BaseIo<T>>(f: Arc<F>) -> impl BaseIo<T> + Clone {
    move || (*f)()
}

pub fn arc<T, F: BaseIo<T>>(f: F) -> impl BaseIo<T> + Clone {
    from_arc(Arc::new(f))
}

pub fn println<T: AsRef<str>>(s: T) -> impl BaseIo<()> {
    move || {
        println!("{}", s.as_ref());
    }
}

pub fn monotonic_now() -> impl 'static + BaseIo<Instant> {
    || Instant::now()
}

pub fn pure_clone<T: Clone>(value: T) -> impl BaseIo<T> {
    move || value.clone()
}

pub fn pure_copy<T: Copy>(value: T) -> impl BaseIo<T> {
    move || value
}

pub fn pure_ref<T>(value: &T) -> impl BaseIo<&T> {
    move || value
}

pub fn bind<T, U, F: BaseIo<T>, G, H: BaseIo<U>>(io_effect: F, g: G) -> impl BaseIo<U>
where
    G: Fn(T) -> H,
{
    move || {
        let value = io_effect();
        g(value)()
    }
}

pub fn run<F: BaseIo<ExitCode>>(io_effect: F) -> ExitCode {
    io_effect()
}
