---
title: Command line
template: docs
lead: >-
  Compiling files, reading stdin, and resolving imports from load paths.
menu:
  visible: true
  order: 2
description: >-
  Use the accent-sass binary to compile stylesheets, pipe from stdin, add load
  paths, and produce compressed output.
---

## Compile a file

```sh
accent-sass input.scss            # to stdout
accent-sass input.scss output.css # to a file
```

The syntax comes from the extension: `.scss` is SCSS, `.sass` is the indented
syntax, and `.css` is plain CSS. Files reached through `@use`, `@import` and
`@forward` always infer their own syntax from their own names, which is
dart-sass's rule.

## Read from stdin

```sh
echo 'a { b: calc(1rem - 2px); }' | accent-sass --stdin
```

Standard input has no extension to infer a syntax from, so it is read as SCSS.
Add `--indented` for the indented syntax:

```sh
printf 'a\n  color: red\n' | accent-sass --stdin --indented
```

## Verify a build

`--check` compiles and writes nothing. What it verifies depends on whether you
name an output file:

```sh
accent-sass --check app.scss          # does it still compile?
accent-sass --check app.scss app.css  # is app.css what it compiles to?
```

The second is the one a CI job wants. It answers whether the CSS you committed
is still what the Sass produces, without a build step, a temporary file or a
`diff`:

```sh
accent-sass --check -I node_modules app.scss app.css
case $? in
  0) ;;
  3) echo "app.css is stale; run the build" >&2; exit 1 ;;
  *) echo "app.scss does not compile" >&2; exit 1 ;;
esac
```

Test for `3` rather than for any non-zero status. `|| echo "stale"` would
report a stylesheet that does not compile as a stale file, which is the
confusion the two codes exist to prevent.

On a mismatch it names the first differing line and prints both versions of it,
then exits `3`. A stylesheet that does not compile exits `1` instead, so a log
tells you which of the two happened:

```
$ accent-sass --check app.scss app.css
app.css is out of date.
  first difference on line 2:
    on disk:    color: blue;
    compiled:   color: red;
```

## Resolve imports from a load path

`-I` (or `--load-path`) adds a directory to search when a relative import does
not resolve. Pass it more than once for more than one directory:

```sh
accent-sass -I node_modules -I vendor/scss app.scss
```

Imports resolve relative to the importing file first, and only then against
load paths, so adding a load path cannot silently change what an existing
relative import means.

This is how you compile a framework installed with npm:

```sh
npm install bulma@1.0.4
printf '@use "bulma/sass";\n' > app.scss
accent-sass -I node_modules app.scss > app.css
```

> [!NOTE]
> Do not name the entry file after the module it loads. `@use "bulma"` inside a
> file called `bulma.scss` resolves to the file itself, because relative
> resolution runs before the load path. You get `Module loop: this module is
> already being loaded`, or, if a `bulma.css` happens to exist beside it, a
> silent empty result. dart-sass behaves the same way.

## Compress the output

```sh
accent-sass --style compressed app.scss > app.min.css
```

Compressed output is valid, equivalent CSS, but it is currently
under-minified next to dart-sass -- see
[Compatibility](/reference/compatibility#compressed-output). If byte-for-byte
minification matters, run a dedicated minifier over the expanded output.

## Silence warnings

Frameworks warn. USWDS emits about 12 KB of `@warn` on every compile, which
buries anything you actually want to read:

```sh
accent-sass -q -I node_modules/@uswds/uswds/packages app.scss > app.css
```

`-q` stops `@warn`, `@debug` and deprecation warnings from reaching the logger.

## Exit status

The binary exits `0` on success and `1` on a compile error, printing the error
to standard error with the source line and a caret under the offending
characters. A wrong command line is `2`, and a stale file under `--check` is
`3`.

[Every flag](/reference/command-line) is in the reference.
