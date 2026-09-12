---
title: JavaScript API
template: docs
lead: >-
  The WebAssembly build's exports, options, results and errors.
menu:
  visible: true
  order: 3
description: >-
  Reference for the accent-sass browser package: compileString, compile,
  CompileOptions, CompileResult and the error shape.
---

Built with `wasm-pack` for the `web` target; see
[Install](/guide/install#the-browser-package). The package ships an
`index.d.ts` with real types.

Option names follow dart-sass's JavaScript API wherever the two have a name for
the same knob, so a caller that knows one knows the other.

## Exports

| Export | Signature |
|---|---|
| `compileString` | `(source: string, options?: CompileOptions) => CompileResult` |
| `compile` | `(path: string, options?: CompileOptions) => CompileResult` |
| `from_string` | `(input: string) => string` |
| `default` / `initSync` | wasm-bindgen's initialisers |

`compile` takes the entry point from `options.files` and ignores `options.url`,
because the path already names the entry.

`from_string` is the package's original export, kept so existing callers keep
working. It has no options and no filesystem, so it cannot resolve `@use`, and
it throws the formatted error block as a **string** rather than an object.

## `CompileOptions`

| Option | Type | Default | Effect |
|---|---|---|---|
| `files` | `Record<string,string>` or `Map<string,string>` | `{}` | The stylesheet tree imports resolve against |
| `loadPaths` | `string[]` | `[]` | Searched when a relative import does not resolve |
| `url` | `string` | `"stdin"` | Virtual path of the source; relative imports resolve against its directory, and its extension sets the syntax |
| `style` | `"expanded"` \| `"compressed"` | `"expanded"` | Output style |
| `syntax` | `"scss"` \| `"indented"` \| `"css"` | from `url` | Syntax of the entry point only |
| `charset` | `boolean` | `true` | Emit `@charset` or a byte-order mark for non-ASCII output |
| `alertAscii` | `boolean` | `false` | Restrict error messages to ASCII. The inverse of the Rust API's `unicode_error_messages` |
| `quiet` | `boolean` | `false` | Silence `@warn` and `@debug` |
| `logger` | `(event: SassLogEvent) => void` | none | Called for each `@warn` and `@debug` |

`"sass"` is accepted as a synonym for `"indented"`, because that is what the
file extension is called. dart-sass spells it `indented`.

Paths in `files` are virtual and normalized: `a/b.scss` and `./a/b.scss` name
the same file.

A malformed option throws a `TypeError` rather than failing later as a
confusing Sass error. An array passed as `files` is rejected for that reason.

## `CompileResult`

| Field | Type | Meaning |
|---|---|---|
| `css` | `string` | The compiled CSS |
| `loadedUrls` | `string[]` | The files the compile read, in order |

`loadedUrls` counts **reads**, not distinct files, so a stylesheet consulted
twice appears twice. De-duplicate if you want a file count.

## `SassLogEvent`

| Field | Type |
|---|---|
| `type` | `"warn"` or `"debug"` |
| `message` | `string` |
| `file` | `string` |
| `line` | `number`, 1-based |
| `column` | `number`, 1-based |

An exception thrown by the logger is swallowed: a warning must not be able to
fail a compile that would otherwise have succeeded.

## Errors

A failed compile throws a real `Error`, so `instanceof Error` holds and a stack
trace survives.

| Property | Meaning |
|---|---|
| `message` | The Sass message alone, with no span or source context |
| `formatted` | The full block the command line prints, with source line and caret |
| `file` | The file the error is in |
| `line` | 1-based |
| `column` | 1-based |

A missing entry point or a non-UTF-8 file has no span; `file` is empty and
`line` and `column` are `0`, so a caller never has to branch on shape.

## The synchronous filesystem

`Fs::read` returns bytes directly, with nothing to await. An importer therefore
cannot `fetch`, cannot `await` and cannot use the File System Access API: every
file a compile might touch must be in `files` before the call.

Making imports asynchronous would mean an asynchronous evaluator, which is a
rewrite rather than a binding change.

## Size

| Build | wasm | gzipped |
|---|---:|---:|
| `--features wasm-exports,random` | 1.69 MB | 0.60 MB |
| Without `wasm-exports` | 0.22 MB | — |

The second row is a module with **no compiler in it**: without the feature,
wasm-bindgen exports nothing and every symbol is dead code.
