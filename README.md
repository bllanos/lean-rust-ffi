# Integrating Lean and Rust <!-- omit from toc -->

This repository contains Rust [crates](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html#packages-and-crates) for using Lean's runtime and Lean libraries in Rust programs, and provides example programs combining the two languages.

## Table of contents <!-- omit from toc -->

- [How to use](#how-to-use)
- [Setup](#setup)
  - [Other useful tools](#other-useful-tools)
- [Examples](#examples)
  - [Using Lean from Rust](#using-lean-from-rust)
    - [Lean](#lean)
    - [C](#c)
    - [Unsafe Rust](#unsafe-rust)
    - [Safe Rust](#safe-rust)
  - [Using Rust from Lean](#using-rust-from-lean)
- [FAQ](#faq)
  - [Why use Lean?](#why-use-lean)
  - [Why use Rust?](#why-use-rust)
  - [Why not use Rust from Lean?](#why-not-use-rust-from-lean)
  - [What are the advantages of combining the two languages?](#what-are-the-advantages-of-combining-the-two-languages)
- [Credits](#credits)
- [References](#references)
- [Contact](#contact)
- [License](#license)

## How to use

Use this project as a source of ideas. It should not be used as a dependency, but as reference material. Lean's Foreign Function Interface has an [explicit warning](https://lean-lang.org/doc/reference/latest/Run-Time-Code/Foreign-Function-Interface/#ffi):

> The current interface was designed for internal use in Lean and should be considered unstable.

This project is an exploration of Lean's Foreign Function Interface as it evolves over time. We do not plan to provide stability by publishing releases that are tied to specific versions of Lean. There are [other projects](#references) that may be suitable as stable dependencies.

See [`lean_build/README.md`](lean_build/README.md) for additional notes on stability.

## Setup

1. Install Rust: <https://rust-lang.org/tools/install/>

   This project does not have a well-known set of compatible Rust versions. At the time of writing, the Rust version used was:

   ```text
   cargo 1.95.0 (f2d3ce0bd 2026-03-21)
   rustc and rust-std 1.95.0 (59807616e 2026-04-14)
   ```

2. Install Lean: <https://lean-lang.org/install/manual/>

   Lean's documentation recommends an [installation through editor plugins](https://lean-lang.org/install/). Editor plugin-guided installation will work as long as the Lean toolchain is added to your shell's `PATH` environment variable. When in doubt, use the manual installation method linked above.

   This project does not have a well-known set of compatible Lean versions. At the time of writing, the Lean version used was:

   ```text
   leanprover/lean4:v4.29.1
   Lean (version 4.29.1, x86_64-unknown-linux-gnu, commit f72c35b3f637c8c6571d353742168ab66cc22c00, Release)
   ```

   The code in this project determines which Lean version to use the same way as ordinary Lean projects: We use [Lake](https://github.com/leanprover/lean4/tree/07dd2c9bf4d711a56791de8177eed41170860ef4/src/lake) to build Lean code, and also include some code from [Elan](https://github.com/leanprover/elan) as explained in [`lean_build/src/elan_fork/README.md`](lean_build/src/elan_fork/README.md). For example, as in a pure Lean project, setting the [`ELAN_TOOLCHAIN` environment variable](https://github.com/leanprover/lean4/blob/07dd2c9bf4d711a56791de8177eed41170860ef4/src/lake/Lake/Config/Env.lean#L82) will override the Lean toolchain version.

3. Install system dependencies of `bindgen`: <https://rust-lang.github.io/rust-bindgen/requirements.html>

   `bindgen` is used to generate Rust function and type signatures for Lean's C interfaces.

### Other useful tools

1. Tools for analyzing memory management problems, such as [Valgrind](https://valgrind.org/)

## Examples

### Using Lean from Rust

#### Lean

We created a small Lean library in [`examples/map_array/lean/map_array/MapArray/Basic.lean`](examples/map_array/lean/map_array/MapArray/Basic.lean) that we wish to use in a Rust program.

There is a sample Lean program that uses the library in [`examples/map_array/lean/map_array_bin/Main.lean`](examples/map_array/lean/map_array_bin/Main.lean).

To run the Lean program, run the following commands in a terminal:

```bash
# Navigate into the `lean/map_array_bin` directory
cd examples/map_array/lean/map_array_bin
# Build and run the program
lake exe main
```

The program should output

```text
Program start
MapOptions instance: { addend := 2, multiplicand := 3 }
Input array: #[0, 5, 10, 15, 20, 25]
Output array: #[6, 21, 36, 51, 66, 81]
Program end
```

The program creates an array of `6` numbers and passes it to the `map` function from the [Lean `MapArray` library](examples/map_array/lean/map_array/MapArray/Basic.lean). `map` requires an options argument, of type `MapOptions`, that contains an addend and a multiplicand. `map` produces an array where each number has been summed with the addend and then multiplied by the multiplicand.

#### C

Lean 4 compiles code by generating C code and then compiling the C code with a C compiler. As such, Lean libraries can be used directly by C programs, as explained in Lean's [Foreign Function Interface documentation](https://github.com/leanprover/lean4/blob/master/doc/dev/ffi.md).

Our first step in using Lean libraries from Rust is to learn how to use them from C.

We created a C program with the same functionality as the [Lean program](#lean) in [`examples/map_array/c/main.c`](examples/map_array/c/main.c).

To run the Lean program, run the following commands.

```bash
cd examples/map_array/c
# Run the `test.sh` script
./test.sh
```

> [!NOTE]
> The scripts used to build and run the program were developed on Linux. The scripts may not work in other environments. In contrast, there are no shell scripts for running the Lean and Rust sample programs. It may be possible to run these sample programs in other environments that Lean and Rust support.

The script output should look similar to the following:

```text
+ LAKE=lake
+ ./clean.sh
+ LAKE=lake
+ make run
lake --dir=../lean/map_array build @/MapArray:static
Build completed successfully (6 jobs).
mkdir -p out
cc -o out/main main.c -I $HOME/.elan/toolchains/leanprover--lean4---$VERSION/include \
   ../lean/map_array/.lake/build/lib/libmap_x2darray_MapArray.a \
   $HOME/.elan/toolchains/leanprover--lean4---$VERSION/lib/lean/libInit.a \
   $HOME/.elan/toolchains/leanprover--lean4---$VERSION/lib/lean/libleanrt.a \
   $HOME/.elan/toolchains/leanprover--lean4---$VERSION/lib/libuv.a \
   $HOME/.elan/toolchains/leanprover--lean4---$VERSION/lib/libgmp.a \
   $HOME/.elan/toolchains/leanprover--lean4---$VERSION/lib/libc++.a \
   $HOME/.elan/toolchains/leanprover--lean4---$VERSION/lib/libc++abi.a \
   -lm
out/main
Program start
MapOptions instance: { addend := 2, multiplicand := 3 }
Populating input array: [ 0, 5, 10, 15, 20, 25, ]
Output array: [ 6, 21, 36, 51, 66, 81, ]
Program end
```

The output includes the commands used to build the program, not only the output from the program itself.

#### Unsafe Rust

Rust programmers usually integrate code from other languages in two steps:

1. A low-level [`*-sys` crate](https://doc.rust-lang.org/cargo/reference/build-scripts.html#-sys-packages) links the foreign library and declares the types and functions needed to call into the library from Rust code. The declarations mirror the C interface of the library, and must be used from [`unsafe` Rust code](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html) because the Rust compiler cannot verify whether functions from other languages satisfy Rust's memory safety guarantees.

2. One or more high-level Rust crates depend on the low-level crate and provide idiomatic Rust types and functions that can be used by Rust code without the `unsafe` keyword. Different programmers may have different needs and preferences that lead to different high-level interfaces, but they can all reuse the minimal low-level interface of the library from the `*-sys` crate.

[`examples/map_array/rust/map_array_bin/src/bin/low_level.rs`](examples/map_array/rust/map_array_bin/src/bin/low_level.rs) is a Rust program that uses our `*-sys` crates for the Lean runtime and for the [Lean `MapArray` library](examples/map_array/lean/map_array/MapArray/Basic.lean). It replicates the [C program](#c) and therefore looks very similar. The `unsafe` keyword appears throughout.

To run `low_level.rs`, run the following commands:

```bash
cargo run -p map-array-bin --bin low_level
```

The commands should output build output from [Cargo](https://doc.rust-lang.org/cargo/index.html), followed by output from the program itself:

```text
Program start
Lean toolchain version used to build the lean-sys crate: leanprover/lean4:$VERSION
MapOptions instance: { addend := 2, multiplicand := 3 }
Populating input array: [ 0, 5, 10, 15, 20, 25, ]
Output array: [ 6, 21, 36, 51, 66, 81, ]
Program end
```

This low-level program depends on the following Rust crates from this project:

1. [`lean-build`](lean_build) contains utilities for writing the [build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html) of `*-sys` crates that use Lean and Lean libraries.

   `lean-build` helps with:
   1. Finding and linking to Lean library files for the Lean runtime and for custom Lean libraries.
   2. Defining Rust equivalents of the types and functions in the C code output by Lean's compiler. We do so using [`bindgen`](https://rust-lang.github.io/rust-bindgen/).
   3. Ensuring that Cargo automatically rebuilds Lean and Rust code whenever there the Lean toolchain version Lean code change.

2. [`lean-sys`](lean_sys) is a low-level crate that links to the Lean runtime using `lean-build`.

3. [`map-array-sys`](examples/map_array/rust/map_array_sys) is a low-level crate that links to the [Lean `MapArray` library](examples/map_array/lean/map_array/MapArray/Basic.lean) using `lean-build`.

#### Safe Rust

[`examples/map_array/rust/map_array_bin/src/bin/high_level.rs`](examples/map_array/rust/map_array_bin/src/bin/high_level.rs) is a Rust program that uses a high-level Rust interface for the [Lean `MapArray` library](examples/map_array/lean/map_array/MapArray/Basic.lean). It is intended to resemble the original [Lean program](#lean), while following Rust style conventions.

`high_level.rs` does not contain the `unsafe` keyword. To emphasize this point, it begins with `#![forbid(unsafe_code)]`, which causes the Rust compiler to raise an error when it encounters `unsafe` code.

To run `high_level.rs`, run the following commands:

```bash
cargo run -p map-array-bin --bin high_level
```

The output from the commands should resemble the following:

```text
Program start
Lean toolchain version used to build the lean-sys crate: leanprover/lean4:$VERSION
MapOptions instance: { addend := 2, multiplicand := 3 }
Input array: [0, 5, 10, 15, 20, 25]
Output array: [ 6, 21, 36, 51, 66, 81, ]
Program end
```

This high-level program depends on the following additional Rust crates:

1. [`lean`](lean) provides higher-level abstractions on top of [`lean-sys`](lean_sys).

2. [`map-array`](examples/map_array/rust/map_array) provides higher-level abstractions on top of [`map-array-sys`](examples/map_array/rust/map_array_sys).

### Using Rust from Lean

The [`examples/sleep/`](examples/sleep) directory contains a more advanced demonstration of using the two languages in the same program and compares and contrasts code written in each language. See [`examples/sleep/README.md`](examples/sleep/README.md).

## FAQ

### Why use Lean?

Lean is an expressive language that allows programmers to focus on the abstract meaning of code and ignore execution details. Programs written in Lean are easy to reason about using [denotational semantics](https://en.wikipedia.org/wiki/Denotational_semantics) and easier to formally verify for correctness.

### Why use Rust?

Rust is a systems programming language with a large ecosystem of tooling and libraries that support a wide range of applications.

### Why not use Rust from Lean?

The code in this repository assists with developing Lean libraries that depend on Rust libraries, as mentioned [above](#using-rust-from-lean). This project assumes that Lean and Rust libraries will be linked together using Rust's build tools and that the `main()` functions of programs will be written in Rust. It is not designed as a framework for writing Lean programs that depend on Rust libraries, but for writing Rust programs that depend on Lean libraries which may, in turn, depend on Rust libraries.

Writing Lean libraries that depend on Rust code is undesirable from two perspectives:

1. Lean's formal verification tools cannot reason about code in other languages. Rust code could make Lean libraries unsound. A related idea is the [F\* language's lattice of computational effects](https://fstar-lang.org/papers/mumon/paper.pdf) where pure, total functions cannot depend on partial or divergent functions.

2. Rust is an ideal language for writing runtime infrastructure that connects logic with the real world. Runtime infrastructure should have a one-way dependency on the abstract logic of a program, according to architecture patterns such as the [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html).

### What are the advantages of combining the two languages?

A program written in a functional programming language combines a pure functional core with a surrounding runtime that executes side effects (see [_Functional Programming in Lean_](https://lean-lang.org/functional_programming_in_lean/Hello___-World___/Running-a-Program)). Working on both the runtime (in a systems programming language such as Rust) and pure functional core of a program (in a pure functional language such as Lean) leads to better results than working only within the core, for the following reasons:

1. The runtime has some built-in overhead because it needs to run arbitrary programs. This is a "curse of generality". A runtime that can make assumptions about the program it is running can work more efficiently.

   For example, when the core opens a file, the runtime must keep track of references to the file to [avoid closing the file until after all references expire](https://lean-lang.org/functional_programming_in_lean/Hello___-World___/Worked-Example___--cat). If the runtime instead knew which file to open, invoked a function from the core on data from the file (not on the file itself), and closed the file immediately afterwards, it would not need file handle reference tracking.

2. Customizing the runtime provides a greater level of control over how a program executes, and allows parts of the runtime to be removed if it is known that the program will not use them.

3. Programs need to perform operations ("side effects") that are difficult to model as pure functions. Functional programming languages introduce abstractions to help manage side effects, such as [monads](https://lean-lang.org/functional_programming_in_lean/Monads/Summary), [algebraic effects](https://koka-lang.github.io/koka/doc/book.html#why-handlers), and [graded types](https://granule-project.github.io/granule.html). Unfortunately, these abstractions are sometimes:
   1. Difficult to understand

   2. Contagious, affecting the entire language or large portions of code that do not directly perform side effects

   3. A false sense of security, because the execution environment may violate assumptions made when reasoning about the program.

   By developing part of a program at the runtime level, we can entirely eliminate representations of some side effects from the pure functional core.

## Credits

The [`lean_build/src/elan_fork/`](lean_build/src/elan_fork) directory contains code adapted from [Elan](https://github.com/leanprover/elan), the Lean version manager. Refer to [`lean_build/src/elan_fork/README.md`](lean_build/src/elan_fork/README.md) for more information.

## References

1. Lean [Foreign Function Interface documentation](https://lean-lang.org/doc/reference/latest/Run-Time-Code/Foreign-Function-Interface/#ffi)

2. Lake [Reverse FFI example](https://github.com/leanprover/lean4/tree/14ff08db6f651775ead432d367b6b083878bb0f9/tests/lake/examples/reverse-ffi)

3. Lake [FFI example](https://github.com/leanprover/lean4/tree/0eb80e34a660f54b88002b33c4d93965651c71cb/tests/lake/examples/ffi)

4. Lean [Foreign type example](https://github.com/leanprover/lean4/blob/a166d6ee20ca87302e89b86f9e639889f633829e/tests/compiler/foreign/README.md)

5. [`lean-sys` Rust crate](https://github.com/digama0/lean-sys)

6. [`mimalloc` Rust crate](https://github.com/purpleprotocol/mimalloc_rust)

7. [The bindgen User Guide](https://rust-lang.github.io/rust-bindgen/)

8. [The Rustnomicon](https://doc.rust-lang.org/nomicon/index.html), in particular the [chapter on FFI](https://doc.rust-lang.org/nomicon/ffi.html)

9. [Leo3: Rust Bindings for Lean4](https://github.com/AndPuQing/leo3)

10. [lean4-rs](https://github.com/LemonHX/lean4-rs), which is likely stale, but which has a smaller volume of code so it is easier to review than [Leo3](https://github.com/AndPuQing/leo3).

## Contact

Feel free to [open an issue](https://github.com/bllanos/lean-rust-ffi/issues).

If you want to start a conversation outside of GitHub, you can contact us first by email, and then we can find the best place to continue the conversation. (See commit metadata for email addresses)

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
