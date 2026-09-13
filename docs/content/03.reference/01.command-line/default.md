---
title: Command line
template: docs
lead: >-
  Every flag the accent-sass binary accepts.
menu:
  visible: true
  order: 1
description: >-
  Reference for the accent-sass command-line binary: arguments, load paths,
  output style, charset and warning flags.
---

```
accent-sass [OPTIONS] [INPUT] [OUTPUT]
```

## Arguments

| Argument | Meaning |
|---|---|
| `INPUT` | The stylesheet to compile. With `--stdin` the stylesheet comes from standard input instead, and this positional names the CSS file to write |
| `OUTPUT` | Where to write the CSS. Omit it to write to standard output. `--stdin` takes only one positional, so the two cannot be combined |

## Options

| Flag | Values | Effect |
|---|---|---|
| `-I`, `--load-path <PATH>` | a directory | Search this directory when a relative import does not resolve. Repeatable |
| `-s`, `--style <STYLE>` | `expanded`, `compressed` | Output style. Default `expanded` |
| `--stdin` | | Read the stylesheet from standard input |
| `--indented` | | Read the entry point as the indented syntax, whatever its name |
| `--check` | | Compile and write nothing. With an `OUTPUT`, verify it is up to date |
| `--no-charset` | | Never emit `@charset` or a byte-order mark, even for non-ASCII output |
| `--no-unicode` | | Restrict error messages to ASCII characters. Does not affect the CSS |
| `-q`, `--quiet` | | Silence `@warn`, `@debug` and deprecation warnings |
| `-v`, `--version` | | Print the version |
| `-h`, `--help` | | Print help |

## Notes

**Syntax follows the extension.** `.scss` is SCSS, `.sass` is the indented
syntax, `.css` is plain CSS. This applies to the entry point; imported files
always infer from their own names.

**Load paths are searched second.** An import resolves relative to the
importing file first, so adding a load path cannot change what an existing
relative import means. A path that no directory covers is not an error by
itself -- it simply never resolves an import.

**`@charset` behaviour matches the reference.** Expanded output gets a
`@charset` declaration and compressed output a byte-order mark, but only when
the stylesheet contains non-ASCII characters. `--no-charset` suppresses both.

**`--check` writes nothing.** It compiles, and then, if you name an `OUTPUT`,
compares the CSS it produced against what that file already holds. Nothing is
written either way, so the mode cannot change the answer it reports. See
[Verify a build](/guide/command-line#verify-a-build) for what to do with it.

## Exit status

| Code | Meaning |
|---|---|
| `0` | The stylesheet compiled, and matched `OUTPUT` if `--check` asked |
| `1` | It did not compile, or a file could not be read or written. The error is on standard error, with the source line and a caret |
| `2` | The command line is wrong -- an unknown flag, or a missing argument |
| `3` | `--check` found `OUTPUT` stale or missing |

`1` and `3` are deliberately different. "Your Sass is broken" and "your CSS is
out of date" call for different responses from whoever reads the log, and one
code for both makes that log lie about which happened. A missing `OUTPUT` under
`--check` is `3` rather than `1`, because it means the build has not run, not
that anything failed.

## Differences from `sass`

The flags above take the same names and values as dart-sass's command line.
Source maps and `--watch` are not implemented. `--check` is this binary's own;
dart-sass has no equivalent.

A flag neither compiler has is rejected: `accent-sass --bogus style.scss` is
`error: unexpected argument '--bogus' found`.

**Thirteen of dart-sass's own flags are accepted and then ignored.** Six of
them say so, because each would change what the program does if it worked, and
silence about that is a lie -- a `--watch` that compiles once and exits looks
exactly like a watcher that missed every change:

```
$ accent-sass --watch style.scss
Warning: --watch is not implemented, and is ignored.
```

The six are `--update`, `--watch`, `--poll`, `-i`/`--interactive`,
`--embed-sources` and `--embed-source-map`. `--quiet` does not silence them:
`-q` covers what the *stylesheet* says, and these report that the command line
asked for something it will not get.

The other seven pass without a word, because each already describes what this
binary does. It writes no source maps, so `--no-source-map` and
`--source-map-urls` ask for the status quo; it never writes an error
stylesheet, never colours its output, compiles one file per run, and has no
deprecation warnings to repeat, which covers `--no-error-css`,
`-c`/`--no-color`, `--no-stop-on-error` and `--verbose`. dart-sass ignores
`--precision` too.
