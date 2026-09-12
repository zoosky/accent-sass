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
| `INPUT` | The stylesheet to compile. Omit it with `--stdin` to read standard input |
| `OUTPUT` | Where to write the CSS. Omit it to write to standard output |

## Options

| Flag | Values | Effect |
|---|---|---|
| `-I`, `--load-path <PATH>` | a directory | Search this directory when a relative import does not resolve. Repeatable |
| `-s`, `--style <STYLE>` | `expanded`, `compressed` | Output style. Default `expanded` |
| `--stdin` | | Read the stylesheet from standard input |
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

## Exit status

| Code | Meaning |
|---|---|
| `0` | The stylesheet compiled |
| `1` | It did not. The error is on standard error, with the source line and a caret |

## Differences from `sass`

The flags above take the same names and values as dart-sass's command line.
Source maps and `--watch` are not implemented.

A flag neither compiler has is rejected: `accent-sass --bogus style.scss` is
`error: unexpected argument '--bogus' found`.

**Fourteen of dart-sass's own flags are accepted and then ignored.**
`--update`, `--no-error-css`, `--no-source-map`, `--source-map-urls`,
`--embed-sources`, `--embed-source-map`, `--watch`, `--poll`,
`--no-stop-on-error`, `-i`/`--interactive`, `--indented`, `--precision`,
`-c`/`--no-color` and `--verbose` all parse and change nothing. The compiler
reads only the flags in the table above, so `--indented` does not switch the
syntax of `--stdin` input and `--precision` does not change a number.

**Twelve of those fourteen also want a value, which dart-sass does not ask
for.** Only `--no-color` and `--verbose` are declared as switches; the rest
are declared as taking one, so passing a flag the way dart-sass accepts it
swallows the argument after it:

```
$ accent-sass --watch style.scss
error: the following required arguments were not provided:
```

The stylesheet was read as the value of `--watch`, leaving no input file.
Writing `--watch=1 style.scss` compiles the stylesheet and ignores the flag.

Treat this as a defect rather than as an interface. A script written against
dart-sass's command line fails here on the flag rather than on the missing
feature, and the wording above describes what the binary does today, not what
it should do.
