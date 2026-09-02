# grass

This crate aims to provide a high level interface for compiling [Sass](https://sass-lang.com/documentation/) into
plain CSS. It offers a very limited API, currently exposing only 2 functions.

In addition to a library, this crate also includes a binary that is intended to act as an invisible
replacement to the Sass commandline executable.

This crate aims to achieve complete feature parity with the `dart-sass` reference
implementation. A deviation from the `dart-sass` implementation can be considered
a bug except for in the case of error messages and error spans.

[Documentation](https://docs.rs/grass/)  
[crates.io](https://crates.io/crates/grass)

## Status

`grass` has reached a stage where one can be quite confident in its output. For the average user there should not be perceptible differences from `dart-sass`.

Every commit of `grass` is tested against bootstrap v5.0.2, and every release is tested against the last 2,500 commits of bootstrap's `main` branch.

That said, there are a number of known missing features and bugs. The rough edges of `grass` largely include `@forward` and more complex uses of `@use`. We support basic usage of these rules, but more advanced features such as `@import`ing modules containing `@forward` with prefixes may not behave as expected.

All known missing features and bugs are tracked in [#19](https://github.com/connorskees/grass/issues/19).

`grass` is not a drop-in replacement for `libsass` and does not intend to be. If you are upgrading to `grass` from `libsass`, you may have to make modifications to your stylesheets, though these changes should not differ from those you would have to make if upgrading to `dart-sass`.

## Performance

`grass` is benchmarked against `dart-sass` and `sassc` (`libsass`) [here](https://github.com/connorskees/sass-perf). In general, `grass` appears to be ~2x faster than `dart-sass` and ~1.7x faster than `sassc`.

## Cargo Features

### commandline

(enabled by default): build a binary using clap

### random

(enabled by default): enable the builtin functions [`random([$limit])`](https://sass-lang.com/documentation/modules/math/#random) and [`unique-id()`](https://sass-lang.com/documentation/modules/string/#unique-id)

### macro

(disabled by default): enable the macro `grass::include!` for compiling Sass to
CSS at compile time

### nightly

(disabled by default): currently only used by `grass::include!` to enable 
[proc_macro::tracked_path](https://github.com/rust-lang/rust/issues/99515)

## Testing

As much as possible this library attempts to follow the same [philosophy for testing as
`rust-analyzer`](https://internals.rust-lang.org/t/experience-report-contributing-to-rust-lang-rust/12012/17).
Namely, all one should have to do is run `cargo test` to run all its tests.
This library maintains a test suite distinct from the `sass-spec`, though it
does include some spec tests verbatim. This has the benefit of allowing tests
to be run without ruby as well as allowing the tests more granular than they
are in the official spec.

Having said that, to run the official test suite,

```bash
# This script expects node >=v14.14.0. Check version with `node --version`
git clone https://github.com/connorskees/grass --recursive
cd grass && cargo b --release
cd sass-spec && npm install
npm run sass-spec -- --impl=dart-sass --command '../target/release/grass'
```

The spec runner does not work on Windows.

The runner compares warnings and error spans exactly. To score only the CSS
output and error messages -- which is how the numbers below are measured --
pass the runner's leniency flags:

```bash
npm run sass-spec -- --impl=dart-sass --command '../target/release/grass' \
  --trim-errors --ignore-warning-diffs --ignore-error-diffs
```

Against the pinned spec revision, this fork achieves the following results:

```
2026-09-01
PASSING: 7687
FAILING: 6523
TOTAL: 14218
```

One test varies between runs (it depends on `random()`), so the passing count
moves by one either way.

The suite more than doubled between the previously pinned revision (2022-12-08,
6,905 tests, of which 6,149 passed) and the current one, and nearly all of the
new tests cover CSS Color 4. `spec/core_functions/color` alone accounts for
4,953 of the 6,523 failures -- 3,095 of them under `to_space` -- because this
fork implements only the legacy `rgb`/`hsl`/`hwb` spaces. Excluding that
subtree, the fork passes 6,205 of 7,783 tests (79.7%); the largest remaining
groups are `values/calculation` (436, the CSS math functions `round()`,
`mod()`, `rem()`, `log()` and `tan()` inside calculations) and the new CSS
`if()` function (164). The rest are largely aesthetic, relating to whitespace
around comments in expanded mode or to error messages.

## Versioning

The minimum supported rust version (MSRV) of `grass` is `1.70.0`. An increase to the MSRV will correspond with a minor version bump. The current MSRV is not a hard minimum, but future bugfix
versions of `grass` are not guaranteed to work on versions prior to this.

`grass` currently targets `dart-sass` version `1.103.1`. An increase to this number will correspond to either a minor or bugfix version bump, depending on the changes.
