use std::thread::{Builder, JoinHandle, Scope, ScopedJoinHandle};

use crate::{Modules, RuntimeComponents, ThreadRuntime};

fn run_lean_thread<
    C: RuntimeComponents,
    M: Modules,
    R: ThreadRuntime<C, M>,
    T,
    Run: FnOnce(&R) -> T,
>(
    run: Run,
) -> T {
    let output;
    {
        let runtime = unsafe { R::new_thread() };
        output = run(&runtime);
    }
    output
}

pub fn run_in_thread_with_lean_runtime<
    C: RuntimeComponents,
    M: Modules,
    R: ThreadRuntime<C, M>,
    T: Send + 'static,
    Run: FnOnce(&R) -> T + Send + 'static,
>(
    _runtime: &R,
    run: Run,
) -> JoinHandle<T> {
    std::thread::spawn(move || run_lean_thread(run))
}

pub fn run_in_custom_thread_with_lean_runtime<
    C: RuntimeComponents,
    M: Modules,
    R: ThreadRuntime<C, M>,
    T: Send + 'static,
    Run: FnOnce(&R) -> T + Send + 'static,
>(
    _runtime: &R,
    builder: Builder,
    run: Run,
) -> std::io::Result<JoinHandle<T>> {
    builder.spawn(move || run_lean_thread(run))
}

pub fn run_in_custom_scoped_thread_with_lean_runtime<
    'scope,
    'env,
    C: RuntimeComponents,
    M: Modules,
    R: ThreadRuntime<C, M>,
    T: Send + 'scope,
    Run: FnOnce(&R) -> T + Send + 'scope,
>(
    _runtime: &R,
    builder: Builder,
    scope: &'scope Scope<'scope, 'env>,
    run: Run,
) -> std::io::Result<ScopedJoinHandle<'scope, T>> {
    builder.spawn_scoped(scope, move || run_lean_thread(run))
}
