---
title: Rust library
template: docs
lead: >-
  Compiling from a string or a path, supplying your own filesystem, and baking
  CSS into your binary.
menu:
  visible: true
  order: 3
description: >-
  Use accent-sass from Rust: from_string, from_path, Options, a custom Fs
  implementation, MemoryFs, and the include! macro.
---

## Compile a string

```rust
use accent_sass::{Options, from_string};

let css = from_string(
    "a { b { color: &; } }".to_owned(),
    &Options::default(),
)?;

assert_eq!(css, "a b {\n  color: a b;\n}\n");
# Ok::<(), Box<accent_sass::Error>>(())
```

`from_string` names its input `stdin`, so relative imports resolve from the
current directory's root. To compile a string as though it were a file
somewhere -- which an editor compiling an unsaved buffer wants -- use
`from_string_with_file_name`.

## Compile a file

```rust
use accent_sass::{Options, from_path};

let css = from_path("styles/app.scss", &Options::default())?;
# Ok::<(), Box<accent_sass::Error>>(())
```

## Options

`Options` is a builder, and every call returns it:

```rust
use accent_sass::{InputSyntax, Options, OutputStyle, from_path};

let options = Options::default()
    .style(OutputStyle::Compressed)
    .load_path("vendor/scss")
    .quiet(true)
    .input_syntax(InputSyntax::Scss);

let css = from_path("styles/app.scss", &options)?;
# Ok::<(), Box<accent_sass::Error>>(())
```

[The full list](/reference/rust-api#options) is in the reference.

## Supply your own filesystem

`Options::fs` takes any `Fs` implementation, so the compiler never has to touch
the disk. This is the seam that makes the browser build possible, and it is
equally useful to a program that already holds its stylesheets -- a CMS with
themes in a database, an editor with unsaved buffers, a test that does not want
a temporary directory.

`MemoryFs` is a ready-made one:

```rust
use accent_sass::{MemoryFs, Options, from_path};

let mut fs = MemoryFs::new();
fs.insert("theme/_colors.scss", "$brand: #bada55;");
fs.insert("theme/index.scss", "@use \"colors\";\na { color: colors.$brand; }");

let css = from_path("theme/index.scss", &Options::default().fs(&fs))?;
assert_eq!(css, "a {\n  color: #bada55;\n}\n");
# Ok::<(), Box<accent_sass::Error>>(())
```

`MemoryFs::loaded_paths` reports what a compile actually read, in order, which
is what you want for cache invalidation.

> [!IMPORTANT]
> `Fs::read` is synchronous. An importer cannot `await`, so every file a
> compile might touch has to be in the filesystem **before** the compile
> starts. There is no way to fetch a dependency at the moment it is imported.

## Capture warnings

`Options::logger` takes a `Logger`, which receives `@warn` and `@debug` with
their source locations. `StdLogger` writes to standard error, `NullLogger`
discards; implement the trait to route them into your own diagnostics.

## Compile Sass at build time {#compile-sass-at-build-time}

The `macro` feature compiles a stylesheet into your binary during
`cargo build`:

```toml
[dependencies]
accent-sass = { version = "0.15.0", features = ["macro"] }
```

```rust
static CSS: &str = accent_sass::include!("../static/_index.scss");
```

The macro tracks the files it reads with `include_str!`, so editing a partial
triggers a rebuild. Output is compressed and the options are not configurable;
if you need control, compile at runtime instead.

## Errors

Every entry point returns `Result<String, Box<Error>>`. `Display` gives the
formatted block the command line prints -- message, span, source line and
caret. `Error::kind` gives the parts: the message on its own, and a `SpanLoc`
with file, line and column.

Error message wording is explicitly unstable. Do not match on it.
