use std::thread::{Builder, JoinHandle, Scope, ScopedJoinHandle};

use crate::{Modules, SyncRuntimeComponents, ThreadRuntime};

fn run_lean_thread<
    C: SyncRuntimeComponents,
    M: Modules,
    R: ThreadRuntime<C, M>,
    T,
    Run: FnOnce(&R) -> T,
>(
    run: Run,
) -> Result<T, R::ThreadInitializationError> {
    let output;
    {
        let runtime = (unsafe { R::new_secondary_thread() })?;
        output = run(&runtime);
    }
    Ok(output)
}

pub fn run_in_thread_with_lean_runtime<
    C: SyncRuntimeComponents,
    M: Modules,
    R: ThreadRuntime<C, M>,
    T: Send + 'static,
    Run: FnOnce(&R) -> T + Send + 'static,
>(
    _runtime: &R,
    run: Run,
) -> JoinHandle<Result<T, R::ThreadInitializationError>>
where
    <R as ThreadRuntime<C, M>>::ThreadInitializationError: Send + 'static,
{
    std::thread::spawn(move || run_lean_thread(run))
}

pub fn run_in_custom_thread_with_lean_runtime<
    C: SyncRuntimeComponents,
    M: Modules,
    R: ThreadRuntime<C, M>,
    T: Send + 'static,
    Run: FnOnce(&R) -> T + Send + 'static,
>(
    _runtime: &R,
    builder: Builder,
    run: Run,
) -> std::io::Result<JoinHandle<Result<T, R::ThreadInitializationError>>>
where
    <R as ThreadRuntime<C, M>>::ThreadInitializationError: Send + 'static,
{
    builder.spawn(move || run_lean_thread(run))
}

pub fn run_in_custom_scoped_thread_with_lean_runtime<
    'scope,
    'env,
    C: SyncRuntimeComponents,
    M: Modules,
    R: ThreadRuntime<C, M>,
    T: Send + 'scope,
    Run: FnOnce(&R) -> T + Send + 'scope,
>(
    _runtime: &R,
    builder: Builder,
    scope: &'scope Scope<'scope, 'env>,
    run: Run,
) -> std::io::Result<ScopedJoinHandle<'scope, Result<T, R::ThreadInitializationError>>>
where
    <R as ThreadRuntime<C, M>>::ThreadInitializationError: Send + 'static,
{
    builder.spawn_scoped(scope, move || run_lean_thread(run))
}
