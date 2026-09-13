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

**Documentation: <https://zoosky.github.io/accent-sass>** -- a usage guide, a
reference, and a [demo](https://zoosky.github.io/accent-sass/demo/) that
compiles Bulma and USWDS from source in your browser. The site is built from
[`docs/`](docs/README.md) with Accent CMS, which compiles its stylesheet with
this compiler.

## Install

```toml
accent-sass = "0.15.0"
```

To track work that has not been released yet, pin a git revision instead:

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

The API is deliberately small: `from_string`, `from_path`,
`from_string_with_file_name`, and an `Options` builder. `Options::fs` takes any
`Fs` implementation, so a host that already holds its stylesheets can compile
them without writing them to disk first -- `MemoryFs` is one such
implementation, and is what the browser build resolves imports through.

As a binary, intended as a drop-in for the `sass` executable:

```bash
accent-sass input.scss                # compile to stdout
accent-sass --check app.scss app.css  # verify app.css is up to date
```

`--check` compiles and writes nothing, exiting `3` when the output file is
stale or missing and `1` when the stylesheet does not compile, so a CI job can
tell "your CSS is out of date" from "your Sass is broken".

As a browser package, built with `wasm-pack`:

```js
import init, { compileString } from "accent-sass";

await init();

const { css, loadedUrls } = compileString('@use "theme";', {
  files: {
    "theme/_colors.scss": "$brand: #bada55 !default;",
    "theme/_index.scss": '@forward "colors";',
  },
  style: "expanded",
  logger: (event) => console.warn(event.message),
});
```

**Every file a compile might touch must be in `files` before you call it.**
Reads are synchronous: the compiler asks for a file and gets bytes back, with
nothing to await, so an importer cannot `fetch`, cannot `await`, and cannot
reach the File System Access API. An editor loads the theme's stylesheets into
the map first, then compiles. Making imports async would mean an async
evaluator, which is a rewrite rather than a binding change.

Options are named after dart-sass's JavaScript API wherever the two have the
same knob: `style`, `syntax`, `loadPaths`, `charset`, `alertAscii` and `url`,
plus `files`, `logger`, and `quiet` for the one knob dart-sass has no name for.
A failed compile throws an `Error` carrying `message`, `formatted`, `file`,
`line` and `column`.

Build it with:

```bash
wasm-pack build crates/lib --release --target web --out-name index -- \
  --no-default-features --features wasm-exports,random
```

`wasm-exports` is not a default feature. Without it wasm-bindgen exports
nothing and the module contains no compiler at all.
[`docs/demo`](docs/demo/README.md) is a page built on it, compiling Bulma and
USWDS from source in the browser.

## Status

14,085 of 14,266 sass-spec tests pass against the pinned spec revision
`b39c32768`, leaving 173 failures (measured 2026-09-11). CI compiles Bulma,
Pico, Foundation and USWDS with both engines on every commit and fails on any
colour-value difference; that corpus currently shows none.

| Job | Gates? | What it checks |
|---|---|---|
| `tests`, `fmt`, `clippy` | yes | the crate's own suite, on the 1.96.1 MSRV |
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
| `wasm-exports` | no | the JavaScript API for a `wasm32-unknown-unknown` browser build |
| `wasi-exports` | no | a C ABI for embedding the `wasm32-wasip1` module in a host |

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

The crates are on the Rust 2024 edition. The minimum supported Rust version is
`1.96.1`, normalised across the Accent crates; CI gates on it. Raising the MSRV
is a minor version bump.

`accent-sass` targets `dart-sass` version `1.104.1`.
