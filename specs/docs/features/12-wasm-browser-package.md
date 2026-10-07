# A browser package with a real API

`wasm32-unknown-unknown` plus wasm-bindgen, packaged for npm. For a
playground, documentation demos and client-side theme editing: anything that
compiles Sass where there is no filesystem and no process.

**This item unlocks no sass-spec fixtures.** It is delivery work, not
conformance work, and it is ranked by who wants the artifact rather than by
failure count. Nothing here changes what the compiler accepts or prints.

## Status: closed

Gaps 1, 2 and 3 landed in #90 and shipped: the package is on npm as
`accent-sass` since 0.16.1 (#110, #114), with the [browser
guide](../../../docs/content/02.guide/04.browser/default.md), the
[JavaScript API reference](../../../docs/content/03.reference/03.javascript-api/default.md)
and the demo. Gap 4, size, was measured on 2026-10-07 and settled without a
change: every size lever buys its bytes with compile time, and the shipped
profile is the right point for a compiler that runs while someone edits.
[Section 4](#4-size) has the table.

Measured 2026-09-12 on `feature/wasm-browser-api`, aarch64 macOS, wasm-pack
0.13.1, node 24.

The package exposes `compileString(source, options)` and
`compile(path, options)` beside the original `from_string`. Options are
`style`, `syntax`, `loadPaths`, `files`, `url`, `charset`, `alertAscii`,
`quiet` and `logger`, named after dart-sass where dart-sass has a name.
Imports resolve against a caller-supplied file map, errors arrive as an
`Error` carrying `message`, `formatted`, `file`, `line` and `column`, and
`index.d.ts` declares the surface with real types rather than `any`.

The filesystem is [`MemoryFs`](../../../crates/compiler/src/memory_fs.rs), a
public type rather than something private to the binding. That is what makes
it testable: the bindings only exist on `wasm32-unknown-unknown`, so
`cargo test` cannot reach them, but it can reach `MemoryFs` through
`crates/lib/tests/memory_fs.rs`. `from_string_with_file_name` became public
in the same change, because `url` needs it.

| | before | after |
|---|---:|---:|
| module | 1.67 MB, 0.60 MB gzipped | 1.69 MB, 0.60 MB gzipped |
| exports | `from_string` | `compileString`, `compile`, `from_string` |

What it costs to resolve imports is 24 KB of module. Two checks gate it:
`.github/scripts/wasm-api-smoke.mjs` covers the API surface, and
`.github/scripts/demo-check.mjs` compiles Bulma and USWDS through the built
package.

Gap 4, size, is measured in section 4.

### What it made possible

`docs/demo` compiles Bulma 1.0.4 and USWDS 3.13.0 from source in a browser
tab, and is published by `.github/workflows/pages.yml`. Measured in Chrome on
the same machine, against the native binary:

| | files | native | in the browser | output |
|---|---:|---:|---:|---:|
| Bulma | 78 | 0.35 s | 2.5 s | 21,562 lines |
| USWDS | 605 | 1.54 s | 6.6 s | 33,685 lines |

The earlier "roughly native speed" reading came from a single synthetic
stylesheet, where the WebAssembly build ran at 1.09x the native wall time. A
real framework is several times slower than that, so the synthetic number
should not be quoted for framework work.

## What works today

The pipeline exists and, since `zoosky/accent-sass` #47, is honest about what
it ships. Measured 2026-09-08 on `bba5497`, aarch64 macOS, wasm-pack 0.13.1,
node 24:

| build | wasm | `from_string` |
|---|---:|---|
| default features | 0.22 MB | absent -- nothing exported, every symbol dead code |
| `--no-default-features --features wasm-exports,random` | 2.20 MB stripped, 1.53 MB after `wasm-opt` | present |

The `Build WebAssembly` job asks for the right features, runs
`.github/scripts/wasm-smoke.mjs` against the package it just built, and runs on
pull requests that can change the module. A stylesheet compiles under node:
`calc(#{$c} - 0.5rem)` gives `calc(2.5rem - 0.5rem)`.

Three properties make the rest of this item tractable. The compiler uses no
time, thread or process APIs. `Fs` is a three-method trait with a defaulted
fourth. `Options` already carries every knob a JS API would want.

## 1. The binding is one function with default options

### Current behavior

`crates/compiler/src/lib.rs`:

```rust
#[cfg(feature = "wasm-exports")]
#[wasm_bindgen(js_name = from_string)]
pub fn from_string_js(input: String) -> std::result::Result<String, String> {
    from_string(input, &Options::default()).map_err(|e| e.to_string())
}
```

Inherited from upstream. There is no way to choose an output style, say the
input is indented syntax, silence a warning, or resolve an `@use`. A caller
that needs any of those cannot use the package at all.

### What it should expose

`Options` already has the fields: `style`, `input_syntax`, `load_paths`,
`allows_charset`, `unicode_error_messages`, `quiet`, plus `fs`, `logger` and
`custom_fns`. The first six map directly onto a JS options object. `fs` is
gap 2, `logger` is a callback, and `custom_fns` is out of scope here -- a Sass
function implemented in JS would have to cross the boundary on every call.

### Implementation instructions

- Take an options argument as a `JsValue` and deserialize it, or declare a
  `#[wasm_bindgen] pub struct` with setters. Prefer the struct: it gives
  TypeScript users real types in `index.d.ts` instead of `any`.
- Keep `from_string(input)` working with no options argument. It is the
  published surface, such as it is.
- Name the options the way dart-sass names them in its own JavaScript API
  wherever the two overlap, so the packages stay interchangeable. Check the
  reference before writing the names down rather than assuming them; the
  roadmap's ground rule about verifying against dart-sass applies to its API
  surface as much as to its output.
- **[13](13-wasm-wasi.md) gap 4 settled these names first**, for the WASI C
  ABI, against the same reference. Take them from there rather than deriving
  them again: `style`, `syntax`, `loadPaths`, `charset`, `alertAscii`, and
  `quiet` for the one knob dart-sass's JavaScript API has no name for. Two
  traps that gap already walked into are worth inheriting -- dart-sass's
  `Syntax` spells the indented value `indented`, not `sass`, and `alertAscii`
  is the inverse of this crate's `unicode_error_messages`.
- `Options` borrows (`Options<'a>`), and wasm-bindgen structs cannot carry a
  lifetime. Build the `Options` inside the exported function from an owned
  configuration struct.

## 2. Imports need a filesystem the browser can supply

### Current behavior

`Options::default()` uses `StdFs`, which calls `std::fs`. On
`wasm32-unknown-unknown` those calls compile and then fail at runtime, so
`@use` and `@import` cannot resolve. Only single-file stylesheets work.

### What it should do

`Fs` is the whole seam:

```rust
pub trait Fs: std::fmt::Debug {
    fn is_dir(&self, path: &Path) -> bool;
    fn is_file(&self, path: &Path) -> bool;
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> { .. }
}
```

Implement it over a JS value -- a `Map` of path to source, or an object with
`isFile`/`read` methods -- and pass it through `Options::fs`.

### The constraint that decides the design

**`Fs::read` is synchronous.** A JS importer therefore cannot `fetch`, cannot
`await`, and cannot touch the File System Access API. The caller must have
every dependency in memory before it calls the compiler. That is a real
limitation, and it should be stated in the package README rather than
discovered: an editor loads the theme's files into a `Map` first, then
compiles.

Making imports async would mean an async compiler, which is a rewrite of the
evaluator, not a binding change. Do not start down that path under this item.

## 3. Errors are flattened to a string

### Current behavior

`.map_err(|e| e.to_string())` throws a JS string carrying the whole
pretty-printed block -- message, span, source line and caret. An editor that
wants to underline the offending characters has to parse it back out.

### What it should do

`PublicSassErrorKind::ParseError` already carries `message` and a `SpanLoc`
with file, line and column, and the crate documents the message text as
unstable. Throw an object with those as fields, and keep the formatted block
on it as well so nothing regresses for callers printing it whole.

## 4. Size

Settled 2026-10-07, on master `04378ea0`, aarch64 macOS, rustc stable,
wasm-pack 0.13.1, wasm-bindgen 0.2.128, binaryen 132, node 24.9. Each
variant is the same source built with a different profile, run through
`wasm-bindgen` and, where marked, `wasm-opt`, then timed with
`.github/scripts/demo-check.mjs` compiling the demo's Bulma 1.0.4 and USWDS
3.13.0 bundles. Times are the median-looking of two or three runs; they vary
by a few percent.

| Build | Module | Gzipped | Bulma | USWDS |
|---|---:|---:|---:|---:|
| shipped: `opt-level = 3`, `wasm-opt -O` | 1.73 MB | 637 KB | 0.52 s | 2.41 s |
| `opt-level = "s"` | 1.52 MB | 500 KB | 0.67 s | 3.40 s |
| `opt-level = "s"`, `wasm-opt -O` | 1.41 MB | 531 KB | 0.65 s | 3.35 s |
| `opt-level = "z"` (the `small` profile) | 1.28 MB | 418 KB | 0.83 s | 4.70 s |
| `opt-level = "z"`, `wasm-opt -O` | 1.19 MB | 451 KB | 0.79 s | 4.58 s |
| `opt-level = "z"`, `wasm-opt -Oz` | 1.17 MB | 452 KB | 0.76 s | 4.26 s |

What the table says:

- The `small` profile the WASI artifact uses takes a third off the download,
  637 KB to 418 KB gzipped, and makes USWDS take 1.9 times as long. `s` sits
  in between on both axes. The levers trade bytes for time at a steady rate;
  there is no free point.
- `wasm-opt` shrinks the raw module and *grows* the gzipped one for the
  size-optimised builds, and `-Oz` is no better than `-O` after gzip. The item's
  guess that `-Oz` was the cheap lever was wrong, and so was the guess about
  `panic = "abort"`: the shared release profile already has it, with `lto`
  and `codegen-units = 1`, so the only lever left was `opt-level`.
- binaryen 132 refuses the module without `--enable-bulk-memory` and the other
  features rustc emits for this target; wasm-pack passes them, a bare
  `wasm-opt` call does not.

The shipped profile stays. The download is paid once per version and
cached; the compile time is paid on every keystroke in an editor and on every
demo run, and the demo already takes 6.6 seconds for USWDS in a browser. If a
host ever wants the smaller module, the mechanism needs no new tooling:
`CARGO_PROFILE_RELEASE_OPT_LEVEL=z` on the `wasm-pack build` line in
`release.sh` and `pages.yml` produces the 1.28 MB module (measured), and
`[package.metadata.wasm-pack.profile.release] wasm-opt = false` keeps
`wasm-pack` from growing it again. Brotli was not measured; the tool was not
on the machine.

## Testing

`.github/scripts/wasm-smoke.mjs` is the floor: it proves the binding works at
all, and it must keep passing. Add a node test file beside it that exercises
each option, an `@use` through the JS filesystem, and an error's structured
fields. Run it in the same job.

## Acceptance criteria

- The package exposes output style, input syntax, load paths and quiet, plus a
  filesystem, and each has a test.
- `@use` and `@import` resolve through a JS-supplied filesystem.
- A compile error reaches JS as an object with message, file, line and column,
  and the formatted block is still available.
- `index.d.ts` declares the surface with real types, not `any`.
- The smoke test still passes, and the new tests run in the same CI job.
- The package README states the synchronous-filesystem constraint.
