use std::process::ExitCode;
use std::time::Instant;

/// An IO monad is a state monad
///
/// The state is an implied unit type argument and part of the return value,
/// representing the world. It is equivalent to Lean's `IO.RealWorld`, which is
/// erased when Lean code is compiled. In other words, `Fn() -> T` should be
/// used as though it is `Fn(IoRealWorld) -> (IoRealWorld, T)` for some
/// non-cloneable type `IoRealWorld`.
///
/// There are two ways to model IO monads:
///
/// 1. `Fn() -> T`: Such functions must clone any resources they own each time
///    they are invoked, otherwise they cannot implement the `Fn` trait, which
///    requires that it be possible for them to be invoked multiple times.
///
/// 2. `(FnOnce() -> T) + Clone`: Such functions must be cloned (i.e. cloning
///    their resources) by the caller in order to be invoked multiple times.
///
/// The second approach is preferable because it does not incur the cost of
/// cloning unless the function is invoked more than once. Unfortunately,
/// [`Clone`] is not dyn-compatible, so the first approach was chosen because it
/// is more flexible.
///
/// Note that `FnMut() -> T` would violate referential transparency by mutating
/// its resources, which makes it unsuitable for modelling pure functional
/// programming.
pub trait BaseIo<T>: Fn() -> T {}

impl<T, F: Fn() -> T> BaseIo<T> for F {}

pub fn println_immediate<T: AsRef<str>>(s: T) {
    println!("{}", s.as_ref());
}

pub fn println<T: Clone + AsRef<str>>(s: T) -> impl BaseIo<()> {
    move || println_immediate(s.clone())
}

pub fn monotonic_now_immediate() -> Instant {
    Instant::now()
}

pub fn monotonic_now() -> impl 'static + BaseIo<Instant> {
    || monotonic_now_immediate()
}

pub fn pure<T: Clone>(value: T) -> impl BaseIo<T> {
    move || value.clone()
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

/// Evaluate an IO effect
///
/// Evaluation should only be done at the outermost level of a program, which is
/// why this function is only implemented for effects that evaluate to process
/// exit codes.
pub fn run<F: BaseIo<ExitCode>>(io_effect: F) -> ExitCode {
    io_effect()
}
