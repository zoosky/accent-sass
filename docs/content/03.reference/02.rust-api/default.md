---
title: Rust API
template: docs
lead: >-
  Entry points, options, the replaceable traits, and Cargo features.
menu:
  visible: true
  order: 2
description: >-
  Reference for the accent-sass Rust API: from_string, from_path, Options, the
  Fs and Logger traits, MemoryFs, and the crate's Cargo features.
---

Generated documentation is on [docs.rs](https://docs.rs/accent-sass). This page
is the shape of the API rather than its rustdoc.

## Entry points

| Function | Compiles |
|---|---|
| `from_string(input, &Options) -> Result<String>` | A string, named `stdin` |
| `from_string_with_file_name(input, file_name, &Options) -> Result<String>` | A string, named as though it were that file |
| `from_path(path, &Options) -> Result<String>` | A file, read through `Options::fs` |

`from_string_with_file_name` matters for two reasons: relative imports resolve
against the name's directory, and the syntax is inferred from its extension. An
editor compiling an unsaved buffer wants both.

`Result<T>` is `std::result::Result<T, Box<Error>>`.

## Options {#options}

`Options` is a consuming builder; every method returns it. `Options::default()`
is expanded output, `StdFs`, `StdLogger`, no load paths, charset on, Unicode
error messages on, warnings on, and syntax inferred from the file name.

| Method | Type | Default | Effect |
|---|---|---|---|
| `fs` | `&dyn Fs` | `StdFs` | Where imports are read from |
| `logger` | `&dyn Logger` | `StdLogger` | Where `@warn` and `@debug` go |
| `style` | `OutputStyle` | `Expanded` | `Expanded` or `Compressed` |
| `load_path` | `impl AsRef<Path>` | none | Append one load path |
| `load_paths` | `&[impl AsRef<Path>]` | none | Append several |
| `input_syntax` | `InputSyntax` | inferred | `Scss`, `Sass` or `Css`, for the entry point only |
| `allows_charset` | `bool` | `true` | Emit `@charset` or a byte-order mark when output is non-ASCII |
| `unicode_error_messages` | `bool` | `true` | Use non-ASCII characters in error messages |
| `quiet` | `bool` | `false` | Silence `@warn`, `@debug` and deprecations |

## Traits

### `Fs`

Where the compiler looks for imported files.

```rust
pub trait Fs: std::fmt::Debug {
    fn is_dir(&self, path: &Path) -> bool;
    fn is_file(&self, path: &Path) -> bool;
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> { /* identity */ }
}
```

`read` is **synchronous**, which is the constraint that shapes every embedder:
an importer cannot await, so everything a compile might touch must already be
reachable.

`canonicalize` decides module identity. The compiler uses its result as the key
of the module cache, so an implementation that lets two spellings of one path
survive will execute that module twice.

Three implementations ship:

| Type | Reads from |
|---|---|
| `StdFs` | `std::fs`. The default |
| `MemoryFs` | An in-memory map of path to contents |
| `NullFs` | Nothing. Every import fails to resolve |

### `MemoryFs`

| Method | Does |
|---|---|
| `new()` | An empty filesystem |
| `insert(path, contents)` | Add or replace a file. Paths are normalized, so `./a.scss` and `a.scss` are one entry |
| `len()`, `is_empty()` | How many files are stored |
| `loaded_paths()` | The paths read so far, in order, one entry per read |
| `clear_loaded_paths()` | Forget that record, to scope it to one compile |

Directories are implied by the files in them: inserting `a/b/c.scss` makes `a`
and `a/b` report `true` from `is_dir`, which is what the resolver needs to find
`a/b/_index.scss`.

### `Logger`

```rust
pub trait Logger: Debug {
    fn debug(&self, location: SpanLoc, message: &str);
    fn warn(&self, location: SpanLoc, message: &str);
}
```

`StdLogger` writes to standard error; `NullLogger` discards. `Options::quiet`
stops events before they reach the logger at all.

## Errors

`Error` (the crate's `SassError`) implements `Display`, which produces the
formatted block the command line prints. `Error::kind` consumes it and returns
`ErrorKind`:

| Variant | Carries |
|---|---|
| `ParseError` | `message`, a `SpanLoc`, and whether Unicode messages are allowed |
| `IoError` | The entry point could not be read |
| `FromUtf8Error` | A file was not valid UTF-8 |

Imports that cannot be found are `ParseError`s pointing at the `@use` or
`@import`, not `IoError`s.

`SpanLoc` comes from the `codemap` crate, which the crate re-exports. Its lines
and columns are **0-based**; the formatted block and every editor count from
one.

## The `include!` macro

```rust
static CSS: &str = accent_sass::include!("../static/_index.scss");
```

Requires the `macro` feature. Compiles at build time with default options
except output style, which is compressed. Tracked with `include_str!` so
incremental rebuilds notice a changed partial; the `nightly` feature uses
`proc_macro::tracked_path` instead, which is more robust.

## Cargo features {#cargo-features}

| Feature | Default | Effect |
|---|---|---|
| `commandline` | yes | Build the binary, using `clap` |
| `random` | yes | `math.random()`, `random()`, `string.unique-id()`, `unique-id()` |
| `macro` | no | The `accent_sass::include!` macro |
| `nightly` | no | Let `include!` use `proc_macro::tracked_path` |
| `wasm-exports` | no | The JavaScript API for a `wasm32-unknown-unknown` build |
| `wasi-exports` | no | A C ABI for embedding a `wasm32-wasip1` module in a host |

Turning off `random` removes four Sass functions from the build. A stylesheet
that calls one then fails to compile, which is a compile error rather than a
missing feature, so turn it off deliberately.

## The WASI C ABI

`wasi-exports` exposes a C ABI for `wasm32-wasip1`, for a plugin host that
instantiates the compiler once and compiles many stylesheets without paying
process startup for each. Strings cross as UTF-8 in linear memory: allocate,
write, call, read three 32-bit words (status, pointer, length), free. An
options handle carries the same knobs as `Options` and is reusable.

Both release profiles set `panic = "abort"`, so a panic reaches the host as a
trap and leaves the instance unusable. A host compiling untrusted input should
be ready to discard the instance.
