# accent-sass

Sass infrastructure for Rust, at parity with `dart-sass`.

`accent-sass` compiles [Sass](https://sass-lang.com/documentation/) to CSS in
pure Rust, with no Node and no libsass. It is a fork of
[`connorskees/grass`](https://github.com/connorskees/grass) that carries the
modern Dart Sass features upstream has not released, so that a Rust program
can build real-world stylesheets -- Bulma, Pico, Foundation, USWDS -- without
shelling out to another toolchain.

Parity with `dart-sass` is the goal, not an aspiration to approximate it: a
deviation from the reference implementation is a bug, except in error messages
and error spans.

It is built for and maintained alongside [Accent CMS](https://accentcms.dev),
the single-binary markdown CMS, which compiles theme Sass in-process through
this crate. It is a general-purpose library, and does not depend on Accent.

## Install

Not published to crates.io. Depend on it by git revision:

```toml
accent-sass = { git = "https://github.com/zoosky/accent-sass.git", rev = "<commit>" }
```

## Use

As a library:

```rust
fn main() -> Result<(), Box<accent_sass::Error>> {
    let css = accent_sass::from_string(
        "a { b { color: &; } }".to_owned(),
        &accent_sass::Options::default(),
    )?;
    assert_eq!(css, "a b {\n  color: a b;\n}\n");
    Ok(())
}
```

The API is deliberately small: `from_string`, `from_path`, and an `Options`
builder.

As a binary, intended as a drop-in for the `sass` executable:

```bash
accent-sass input.scss
```

## Status

13,560 of 14,218 sass-spec tests pass against the pinned spec revision
(measured 2026-09-04). CI compiles Bulma, Pico, Foundation and USWDS with both
engines on every commit and fails on any colour-value difference; that corpus
currently shows none.

| Job | Gates? | What it checks |
|---|---|---|
| `tests`, `fmt`, `clippy` | yes | the crate's own suite, on the 1.85.0 MSRV |
| `frameworks` | yes | the four-framework corpus, gated on colour values |
| `bootstrap` | advisory | Bootstrap 5.0.2; prints the delta |
| `sass-spec` | advisory | publishes the spec tallies |

What each release changed is in [`CHANGELOG.md`](CHANGELOG.md). What is left
is in [`specs/docs/features/`](specs/docs/features/README.md), one document
per work item, ranked by the spec tests it unlocks.

`accent-sass` is not a drop-in replacement for `libsass` and does not intend
to be.

## Cargo features

| Feature | Default | Effect |
|---|---|---|
| `commandline` | yes | build the binary, using clap |
| `random` | yes | the builtin [`random([$limit])`](https://sass-lang.com/documentation/modules/math/#random) and [`unique-id()`](https://sass-lang.com/documentation/modules/string/#unique-id) |
| `macro` | no | the `accent_sass::include!` macro, compiling Sass at build time |
| `nightly` | no | lets `include!` use [`proc_macro::tracked_path`](https://github.com/rust-lang/rust/issues/99515) |

## Testing

Running `cargo test` should be all you need. The crate keeps a suite distinct
from `sass-spec`, following the same [philosophy as
`rust-analyzer`](https://internals.rust-lang.org/t/experience-report-contributing-to-rust-lang-rust/12012/17),
so tests run without ruby and can be more granular than the official spec.

To run the official suite (node >= v14.14.0; does not work on Windows):

```bash
git clone https://github.com/zoosky/accent-sass --recursive
cd accent-sass && cargo build --release
cd sass-spec && npm install
npm run sass-spec -- --impl=dart-sass --command '../target/release/accent-sass' \
  --trim-errors --ignore-warning-diffs --ignore-error-diffs
```

The leniency flags score CSS output and error messages only. Without them a
test that differs solely in a missing deprecation warning or in error wording
counts as a failure; that gap is sized in
[`specs/docs/features/08-calculation-warnings-and-error-wording.md`](specs/docs/features/08-calculation-warnings-and-error-wording.md).

## Versioning

[Semantic Versioning](https://semver.org/spec/v2.0.0.html). While the major
version is `0`, a breaking change bumps the minor version. Version numbers are
this fork's own and do not track upstream `grass`.

The crates are on the Rust 2024 edition, which sets the minimum supported
Rust version at `1.85.0`; CI gates on it. Raising the MSRV is a minor version
bump.

`accent-sass` targets `dart-sass` version `1.103.1`.
