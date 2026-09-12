# Command line: check mode, and the flags that swallow their neighbour

**A delivery item. It closes no sass-spec fixture.**

The binary already has a clap command line, and has had one since the fork
began: `crates/lib/src/main.rs`, behind the default `commandline` feature.
What it cannot do is *verify* anything. It compiles a stylesheet and writes
CSS, and there is no way to ask "does this still compile" or "is the CSS I
committed still what the Sass produces" without writing a file and diffing it
by hand.

This item adds `--check`, and repairs three defects found while measuring the
existing command line. Two of the three are older than the check mode and
would be worth fixing on their own.

## The constraint that shapes the design

**The binary must stay a drop-in for `sass`.** Three things drive it by that
contract:

- the sass-spec runner, as `--command '../target/release/accent-sass'`,
- `.github/scripts/frameworks.sh`, which calls it positionally with `-I`,
- `.github/scripts/wasi-smoke.sh` and `wasi-sass.sh`, under wasmtime.

So verification cannot be a `verify` subcommand: adding one would make
`accent-sass style.scss` ambiguous with a subcommand name and put the whole
corpus at risk. `--check` is a flag, and the positional grammar does not
change.

## What was measured

Run on 2026-09-12 against `master` at `2e38d7e1`, with the debug build.

### Defect 1: ten flags take a value, so they eat the file name

Ten arguments are declared with neither an action nor `num_args`. clap 4
defaults to `ArgAction::Set`, which takes a value, so each one consumes the
argument after it:

```
$ accent-sass --indented in.sass
error: the following required arguments were not provided:
  <INPUT>

$ accent-sass --stdin --indented
error: a value is required for '--indented <INDENTED>' but none was supplied
```

`in.sass` was read as the *value of* `--indented`, leaving no input file.
Exit status 2. The ten are `--indented`, `--update`, `--no-error-css`,
`--no-source-map`, `--embed-sources`, `--embed-source-map`, `--watch`,
`--poll`, `--no-stop-on-error` and `-i`/`--interactive`.

This is worse than an unimplemented flag. A script written against dart-sass's
command line does not fail on the missing feature, it fails on the argument
parse, and the error names the input file rather than the flag that ate it.

### Defect 2: you cannot compile the indented syntax from stdin

`--indented` is not wired to anything. `Options::input_syntax` exists and
overrides extension inference at both entry points
(`crates/compiler/src/lib.rs:141` and `:213`), and nothing passes it.

For a file this does not matter, because the extension decides:
`accent-sass in.sass` already compiles as indented. Standard input has no
extension, so `--stdin` is the one input route where the syntax cannot be
inferred, and it is exactly the route the flag exists to serve.

### Defect 3: a failed compile truncates the output file

The output file is opened with `.truncate(true)` *before* the compile runs, so
the compile's own failure destroys the previous result:

```
$ accent-sass good.scss out.css   # out.css holds valid CSS
$ accent-sass bad.scss out.css
Error: Expected expression.
$ wc -c < out.css
0
```

A nonexistent input does the same. Any build that writes over its last output
loses it the moment a stylesheet stops compiling, which is the moment you most
want the last good file.

## What `--check` does

`--check` compiles and writes nothing. What it verifies depends on whether you
name an output file:

| Command | What it checks |
|---|---|
| `accent-sass --check app.scss` | the stylesheet compiles |
| `accent-sass --check app.scss app.css` | it compiles, **and** `app.css` is what it compiles to |

The second is the one a CI job wants: it answers "is the committed CSS current"
without a build step, a temporary file or a `diff` invocation.

On a mismatch it names the first differing line and prints both versions of
it. It does not diff the whole file. The mode answers *is this current*, and
one line is enough to separate a stale build from a wrong one; a full diff is
what `diff` is for.

### Exit status

| Code | Meaning |
|---|---|
| 0 | The stylesheet compiled, and matched the output file if one was named |
| 1 | It did not compile, or a file could not be read or written |
| 2 | The command line is wrong. clap's own code, unchanged |
| 3 | It compiled, and the output file is stale or missing |

**1 and 3 are deliberately different.** "Your Sass is broken" and "your CSS is
out of date" call for different responses from whoever reads the CI log, and
a check mode that collapses them makes the log lie about which happened. 2 is
clap's, so 3 is the first free code.

A missing output file is 3 rather than 1: under `--check` it means the build
has not run, not that anything failed. A file that exists and cannot be read
is 1, because that is a real I/O failure.

## What the repair does, flag by flag

All ten become real switches, which is defect 1 closed. What happens after
parsing splits three ways.

**`--indented` is implemented**, and stops being hidden. It sets
`InputSyntax::Sass` for the entry point, which is dart-sass's rule -- a file
reached through `@use`, `@import` or `@forward` still infers its own syntax
from its own name.

**Six flags warn, and are ignored.** `--update`, `--watch`, `--poll`,
`-i`/`--interactive`, `--embed-sources` and `--embed-source-map` would each
change what the program does if they worked. Silence about those is a lie:
a `--watch` that compiles once and exits looks exactly like a watcher that
missed every change. Each prints `Warning: --watch is not implemented, and is
ignored.` to standard error.

`--quiet` does not silence these. `-q` is about what the *stylesheet* says --
`@warn`, `@debug`, deprecations -- and these are about the command line being
wrong, which is not something the stylesheet can ask to have hidden.

**Seven flags stay silent, and are ignored.** `--no-error-css`,
`--no-source-map`, `--source-map-urls`, `--no-stop-on-error`, `--precision`,
`-c`/`--no-color` and `--verbose` each already describe what this binary does.
It writes no source maps, so `--no-source-map` is a request for the status
quo; it never writes an error stylesheet, never colours its output, compiles
one file per run, and has no deprecation warnings to repeat. dart-sass ignores
`--precision` too. Warning about a flag that asks for the behaviour you are
already getting is noise.

## What this deliberately does not do

- **Source maps.** Still unimplemented. Three of the silent flags and two of
  the warning ones are theirs, and they stay accepted so a dart-sass command
  line keeps parsing.
- **`--watch`.** Still unimplemented; it now says so.
- **A whole-file diff under `--check`.** See above.
- **An atomic write.** The output file is opened only after the compile
  succeeds, which is defect 3 closed, but a write that fails part way through
  still leaves a partial file. Writing to a temporary file and renaming would
  close that too; it is a different failure, far rarer, and out of scope here.

## Acceptance criteria

1. None of the ten flags consumes the argument after it. `accent-sass --watch
   style.scss` compiles `style.scss`.
2. `accent-sass --stdin --indented` compiles the indented syntax from standard
   input.
3. A failed compile leaves an existing output file byte-for-byte unchanged.
4. `--check` writes no file, under any combination of arguments.
5. `--check` exits 0 on a current output file, 3 on a stale or missing one,
   and 1 on a stylesheet that does not compile.
6. The sass-spec and `frameworks` jobs are unaffected. Neither passes any flag
   this item changes: the runner passes `--precision` and the input file, and
   `frameworks.sh` passes `-I`.

## Verification

`crates/lib/tests/command-line.rs` drives the built binary through
`CARGO_BIN_EXE_accent-sass` and covers all six criteria. It is gated on the
`commandline` feature, so a `--no-default-features` build still compiles.

The test file is the first in this crate to run the binary as a process. The
rest of `crates/lib/tests/` calls the library through the `test!` and `error!`
macros, which cannot see argument parsing, an exit status or a file left on
disk -- the three things every defect above lives in.
