---
title: Install
template: docs
lead: >-
  As a binary, as a library, or as a browser package.
menu:
  visible: true
  order: 1
description: >-
  Install accent-sass from crates.io as a command-line binary or a Rust
  library, pin a git revision, or build the WebAssembly package.
---

## The command-line binary

```sh
cargo install accent-sass
```

That gives you `accent-sass`, which takes the same arguments as `sass` for the
options both support:

```sh
accent-sass input.scss output.css
```

## The library

```toml
[dependencies]
accent-sass = "0.15.0"
```

The `commandline` feature is on by default and pulls in `clap` to build the
binary. A library dependency does not need it:

```toml
[dependencies]
accent-sass = { version = "0.15.0", default-features = false, features = ["random"] }
```

Keep `random` unless you know you do not want it: dropping it removes
`math.random()`, `random()`, `string.unique-id()` and `unique-id()` from the
build, and a stylesheet that calls one of them then fails to compile.

[Cargo features](/reference/rust-api#cargo-features) lists the rest.

## Track unreleased work

Accent CMS pins a revision of this repository rather than a crates.io release,
and you can do the same:

```toml
[dependencies]
accent-sass = { git = "https://github.com/zoosky/accent-sass.git", rev = "<commit>" }
```

## The browser package

There is no published npm package yet. Build it from a checkout with
[`wasm-pack`](https://rustwasm.github.io/wasm-pack/):

```sh
rustup target add wasm32-unknown-unknown
wasm-pack build crates/lib --release --target web --out-name index -- \
  --no-default-features --features wasm-exports,random
```

`wasm-exports` is not a default feature, and the build is quietly useless
without it: wasm-bindgen exports nothing, every symbol is dead code, and you
get a 0.2 MB module with no compiler in it. With the feature, the module is
1.69 MB, or 0.60 MB gzipped.

[In a browser](/guide/browser) covers using it.

## Requirements

The minimum supported Rust version is **1.96.1**, which is what CI pins for the
gating jobs. There is no C toolchain, no Node, and no `libsass` anywhere in the
dependency tree.
