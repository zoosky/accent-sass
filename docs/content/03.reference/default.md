---
title: Reference
template: docs
lead: >-
  Every flag, function, option and known divergence.
menu:
  visible: true
  order: 3
description: >-
  Reference for accent-sass: command-line flags, the Rust API, the JavaScript
  API, Cargo features, and compatibility with dart-sass.
---

| Page | Covers |
|---|---|
| [Command line](/reference/command-line) | Every flag the binary accepts |
| [Rust API](/reference/rust-api) | Entry points, `Options`, the traits, and Cargo features |
| [JavaScript API](/reference/javascript-api) | `compileString`, `compile`, options, results and errors |
| [Compatibility](/reference/compatibility) | What matches dart-sass, and what does not |

The Rust API is also published on
[docs.rs](https://docs.rs/accent-sass), generated from the source.

## Versions

| | |
|---|---|
| Crate version | 0.15.0 |
| Reference implementation | dart-sass 1.104.0 |
| Minimum supported Rust | 1.96.1 |
| Licence | MIT |

Three crates share a version and are released together: `accent-sass`,
`accent_sass_compiler` and `accent-sass-macro`. Depend on `accent-sass`; the
other two are its internals and its proc macro.

## Stability

The Sass language behaviour is the contract, and it is defined by dart-sass.
Two things are explicitly **not** stable and should not be matched on:

- **Error message wording.** It may change between bugfix versions.
- **Error spans.** Which characters an error points at may change.

Everything else follows semantic versioning. While the major version is `0`, a
breaking change bumps the minor version.
