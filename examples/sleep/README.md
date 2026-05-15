# Asynchronous operations example <!-- omit from toc -->

## Table of contents <!-- omit from toc -->

- [Overview](#overview)
- [Rust code design](#rust-code-design)
- [Key features](#key-features)
  - [Destructive updates](#destructive-updates)
  - [Error handling](#error-handling)
  - [Pruning the Lean runtime](#pruning-the-lean-runtime)
- [References](#references)

## Overview

This example demonstrates bidirectional FFI dependencies between Rust and Lean using asynchronous operations as a sample problem. Lean code delegates asynchronous operation execution to Rust code. We selected asynchronous operations because they highlight some core concepts of, and key differences between, Lean and Rust, such as:

1. Monads
2. Concurrency
3. Lazy evaluation
4. Immutability
5. Performance tradeoffs

## Rust code design

This example does _not_ demonstrate how to do the following:

1. Build a fully-featured asynchronous operations runtime. Lean's runtime uses a general-purpose asynchronous runtime, and there is no need to reimplement Lean.
2. Prevent synchronous code from blocking asynchronous operations

The motivations behind the design are to:

1. Avoid dependencies on anything other than the standard library with respect to asynchronous code, which provides the following benefits:
   1. Faster builds
   2. No need to choose a third-party asynchronous code ecosystem, [which is not an easy decision](https://rust-lang.github.io/async-book/08_ecosystem/00_chapter.html#determining-ecosystem-compatibility)
2. Choose one type of asynchronous operation, sleeping, for simplicity, but still explore it in depth:
   1. Accommodate all the ways of using the operation assuming that it only needs to be composed with itself and with short synchronous operations
   2. Avoid approaches that incur high performance overhead, such as spawning background threads or starting a general-purpose asynchronous runtime
   3. Avoid global variables or other non-local interactions between code. Asynchronous runtimes often introduce such interactions.
   4. Hint at how a wider range of asynchronous operations could be supported
3. Reveal some of the subtleties of supporting asynchronous operations that would normally be hidden inside third-party asynchronous runtimes

For a more scalable approach to implementing time-related asynchronous operations, see the following article [cited in the Tokio runtime's source code](https://docs.rs/tokio/1.49.0/src/tokio/runtime/time/mod.rs.html#85):

> G. Varghese and A. Lauck, "Hashed and hierarchical timing wheels: efficient data structures for implementing a timer facility," in IEEE/ACM Transactions on Networking, vol. 5, no. 6, pp. 824-834, Dec. 1997, doi: 10.1109/90.650142. (<https://www.cs.columbia.edu/~nahum/w6998/papers/ton97-timing-wheels.pdf>)

Given that sleeping is the only supported asynchronous operation, the example never needs to handle the case where another operation needs to interrupt a sleep operation. All sleep operations last for known times, so concurrent operations can be scheduled as follows:

1. Equal sleep operations are merged into one sleep operation, after which operations that followed the sleep operations are scheduled
2. The shorter sleep operation in a pair of unequal sleep operations is scheduled first, after which the remaining part of the longer sleep operation is scheduled with operations that followed the shorter operation.

This design allows sleeping to be implemented using [`std::thread::sleep`](https://doc.rust-lang.org/std/thread/fn.sleep.html). Otherwise, like fully-featured asynchronous runtimes, we would need a system call that can select from multiple possible event sources while respecting a time limit (such as [explained in _Operating Systems: Three Easy Pieces_ by Remzi H. Arpaci-Dusseau and Andrea C. Arpaci-Dusseau](https://pages.cs.wisc.edu/~remzi/OSTEP/threads-events.pdf)). For simplicity, sleeping until a specified time is not supported, as the example cannot guarantee that any interleaved synchronous operations will be sufficiently brief to avoid overshooting the deadline. Sleep operations last at least as long as the times specified by their arguments.

We did not use the Rust [`Future`](https://doc.rust-lang.org/std/future/trait.Future.html) trait because:

1. The [`Waker` mechanism](https://tokio.rs/tokio/tutorial/async#wakers) introduces complexity that is only necessary for running arbitrary asynchronous operations that can complete without the runtime being aware. Our example code has full knowledge about the asynchronous operations it runs.
2. The Rust compiler's support for statically composing `Future` objects using `async`/`await` syntax improves efficiency and readability but cannot be applied to asynchronous operations that are being composed in a different programming language (in this case, Lean). We therefore need to compose futures dynamically at runtime.
3. We want to mimic Lean's asynchronous programming API, both to provide a drop-in partial replacement via FFI, and to show what the API would look like if translated into Rust. Rust's `Future` trait imposes a more imperative programming style that differs from Lean, with `Future::poll()` relying on mutation.

## Key features

### Destructive updates

The [`LeanAsyncIo`](./rust/async_effect_ffi/src/async_io.rs) type imitates Lean's [destructive array update optimization](https://lean-lang.org/functional_programming_in_lean/Programming___-Proving___-and-Performance/Insertion-Sort-and-Array-Mutation/#insertion-sort-mutation) by allowing mutation of instances that are exclusively owned by single Lean objects. Instances that are shared by multiple Lean objects are cloned rather than mutated during an update.

In Rust code, in contrast, the [`AsyncIo`](./rust/async_effect/src/async_io.rs) type makes the caller responsible for cloning. Rust's first-class support for exclusive versus shared ownership semantics allows copy-on-write behavior to be checked by the compiler.

### Error handling

The [`EAsyncIO`](./lean/async_effect/AsyncEffect/AsyncIO.lean) Lean type cancels other asynchronous operations in a set of concurrent operations when one operation in the set resolves to an error. To do this, `EAsyncIO`'s `concurrently` operation is implemented on top of `AsyncIO.select`, rather than `AsyncIO.concurrently`. `AsyncIO` is a thin wrapper over a Rust type, [`LeanAsyncIo`](./rust/async_effect_ffi/src/async_io.rs), and is similar to Lean's [`BaseAsync`](https://github.com/leanprover/lean4/blob/3c6317b6d77a565b4217532d1190ac6955dba842/src/Std/Async/Basic.lean#L387) type, which does not handle errors.

Lean's own equivalent of `EAsyncIO`, [`EAsync`](https://github.com/leanprover/lean4/blob/3c6317b6d77a565b4217532d1190ac6955dba842/src/Std/Async/Basic.lean#L568) implements short-circuiting of concurrent asynchronous operations by [awaiting concurrent operations in series](https://github.com/leanprover/lean4/blob/3c6317b6d77a565b4217532d1190ac6955dba842/src/Std/Async/Basic.lean#L797), skipping later `await` actions when one resolves to an error. This behavior differs from the Rust-backed `EAsyncIO`, because `EAsync` short-circuits on results in the order in which they are awaited, not in the order in which they resolve.

There is no pure Rust implementation of `EAsyncIO`, as the code would be similar to `EAsyncIO` (but translated into Rust) and would not demonstrate anything sufficiently interesting. Therefore, the `short_circuit` example has a pure Lean version and a Rust-backed Lean version, but no pure Rust version.

### Pruning the Lean runtime

[`concurrent_no_runtime.rs`](rust/sleeper/src/bin/concurrent_no_runtime.rs) is a version of [`concurrent.rs`](rust/sleeper/src/bin/concurrent.rs) that does not start the Lean runtime. Not starting the Lean runtime before running Lean code is only possible if the Lean code does not rely on any features of the Lean runtime. As the Rust compiler cannot verify whether Lean code uses the Lean runtime, skipping Lean runtime initialization is `unsafe` and is therefore not a feature of the [safe `lean` crate](../../lean/).

The table below compares performance characteristics of the different versions of the `concurrent` example:

| Metric                                    | Pure Lean            | Lean using Rust      | No Lean Runtime      | Pure Rust           |
| ----------------------------------------- | -------------------- | -------------------- | -------------------- | ------------------- |
| Executable size                           | 5196048 bytes (5.0M) | 4594400 bytes (4.4M) | 4531880 bytes (4.4M) | 484360 bytes (474K) |
| Maximum resident set size (kbytes)        | 17400                | 6952                 | 6400                 | 2228                |
| Minor (reclaiming a frame) page faults    | 280                  | 207                  | 195                  | 89                  |
| Voluntary context switches                | 314                  | 20                   | 18                   | 18                  |
| Involuntary context switches              | 24                   | 0                    | 0                    | 1                   |
| Wall clock time                           | 26.01 secs           | 26.01 secs           | 26.01 secs           | 26.01 secs          |
| User time                                 | 10.96 millis         | 1.01 millis          | 4.28 millis          | 0.79 millis         |
| System time                               | 11.88 millis         | 7.06 millis          | 3.43 millis          | 3.94 millis         |
| Number of system calls                    | 1672                 | 171                  | 131                  | 108                 |
| Number of threads (`clone3` system calls) | 5                    | 1                    | 0                    | 0                   |

Data in the table was collected using commands, resembling the following, that were run on a Linux system:

```bash
cargo build --release -p sleeper-lean --bin concurrent
ls -l target/release/concurrent
ls -lh target/release/concurrent
/usr/bin/time -v target/release/concurrent
# [Fish shell](https://fishshell.com/) `time` command
time target/release/concurrent
strace -cf target/release/concurrent
```

## References

1. The [`Many` monad](https://lean-lang.org/functional_programming_in_lean/Monads/Example___-Arithmetic-in-Monads/#nondeterministic-search) in [Functional Programming in Lean](https://lean-lang.org/functional_programming_in_lean/), by David Thrane Christiansen, updated October 2025.

2. [Getting Lazy with C++](https://bartoszmilewski.com/2014/04/21/getting-lazy-with-c/), by Bartosz Milewski, April 21, 2014.

3. [min-sized-rust](https://github.com/johnthagen/min-sized-rust), tips for reducing Rust executable sizes.
