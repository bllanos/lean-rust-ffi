# lean-build <!-- omit from toc -->

As described in the [main README](../README.md#unsafe-rust), this crate contains utilities for writing the [build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html) of `*-sys` crates that use the Lean runtime, the Lean standard library, and/or user-written Lean libraries.

## Table of contents <!-- omit from toc -->

- [Design](#design)
  - [Linking](#linking)
- [Credits](#credits)

## Design

`lean-build` uses a version of Lean already present on the user's system. Users can specify the Lean version using the same methods for specifying the Lean version when building an ordinary Lean package. In other words, `lean-build` intends to follow the same conventions as the Lean community for version management.

There are several implications of this approach:

1. `lean-build` depends on [`bindgen`](https://rust-lang.github.io/rust-bindgen/) to create Rust bindings for arbitrary versions of Lean at build time, and therefore requires users to install [`bindgen`'s own dependencies](https://rust-lang.github.io/rust-bindgen/requirements.html).
2. Some versions of Lean may be so different from those tested with `lean-build` that they may be incompatible with the crate. In particular, `lean-build` assumes the existence of some [Lean runtime functions](https://lean-lang.org/doc/reference/latest/Run-Time-Code/Foreign-Function-Interface/#ffi-initialization) whose declarations are not in Lean's public C/C++ header files, and therefore cannot be processed by `bindgen`. See [`src/lean_sys_root_module.rs`](src/lean_sys_root_module.rs).

[Other Rust projects for using Lean](../README.md#references) may make different choices, such as packaging Rust bindings for particular versions of Lean so that users cannot use arbitrary versions of Lean but benefit from stability and fewer dependencies.

### Linking

Following the [arguments expressed in `min-sized-rust`](https://github.com/johnthagen/min-sized-rust#dynamic-linking-why-it-doesnt-work), we use static linking where possible. The only libraries linked dynamically are those that are pre-installed on most platforms (e.g. `libm`).

For simplicity, we do not intend to support both static and dynamic linking. To use dynamic linking, one can do either of the following:

1. Change the [`rustc-link-lib`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-link-lib) directives in the code to use dynamic linking.
2. Use another project that supports [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html) for choosing different linking strategies ([example](https://github.com/digama0/lean-sys/blob/cb1a8e4996120d6a7b10b31b2d2235c237b8336f/build.rs#L34)).

## Credits

The [`elan_fork/`](src/elan_fork) directory contains code adapted from [Elan](https://github.com/leanprover/elan), the Lean version manager. Refer to [`src/elan_fork/README.md`](src/elan_fork/README.md) for more information.
