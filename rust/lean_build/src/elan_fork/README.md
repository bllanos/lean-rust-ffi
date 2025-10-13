# Partial fork of elan

This directory contains code adapted from [Elan](https://github.com/leanprover/elan), the Lean version manager.

Only the code required to query the Lean toolchain used by a Lake package and determine the source of the toolchain (e.g. an environment variable or a toolchain specification file) is included. These are the only features needed by `lean-build`.

The Elan commit used as a reference was the following:

```text
commit 58e8d545e33641f66dbcbd22c4283109e71757be (HEAD -> master, tag: v4.1.2, origin/master, origin/HEAD)
Author: Sebastian Ullrich <sebasti@nullri.ch>
Date:   Mon May 26 10:58:24 2025 +0200

    chore: Release
```

At the time, Elan was distributed under either the MIT or Apache 2.0 licenses (the same licenses used by `lean-build`). Copies of Elan's license files are included in this directory.

## Modifications

Aside from the limited scope of the code included from Elan, the following other modifications were made:

1. Interactions with the network (e.g. fetching URLs) were removed, for the following reasons:
   1. To avoid modifying the user's installed Lean toolchains (although `lean-build` runs Lake commands that may do so regardless).
   2. To make builds more deterministic by reducing reliance on data from external systems.
2. Code was changed to return an error when it would otherwise have modified files or directories, to avoid modifying the user's Lean installation.
3. [`thiserror`](https://github.com/dtolnay/thiserror) was used instead of [`error-chain`](https://github.com/rust-lang-deprecated/error-chain), which is deprecated.
4. Toolchain resolution no longer requires a directory argument but uses the current directory when the toolhain is not overridden by the `ELAN_TOOLCHAIN` environment variable.

## Rationale for inclusion

We forked some of Elan's code because:

1. Removing dependencies unneeded by the subset of the code that was forked improves build times and saves disk space.
2. Elan was not published on `crates.io` (see <https://crates.io/search?q=elan>) at the time of writing, which would prevent `lean-build` from being published to `crates.io` (as [explained in the Cargo book](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#local-paths-in-published-crates)).
