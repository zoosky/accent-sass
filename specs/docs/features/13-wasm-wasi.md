# A WASI build, and a CI job that runs it

`wasm32-wasip1`. One portable artifact that compiles Sass against real files
inside a sandbox the host controls, for anywhere a native binary is unwelcome:
a build service running untrusted themes, a plugin host, a machine that is not
one of the platforms this project ships binaries for.

**This item unlocks no sass-spec fixtures.** It is delivery work, ranked by
who wants the artifact rather than by failure count.

## What works today

Measured 2026-09-08 on `bba5497`, aarch64 macOS. Both the library and the
command-line binary compile for `wasm32-wasip1` with no source changes:

```bash
rustup target add wasm32-wasip1
cargo build --release -p accent-sass --target wasm32-wasip1 --features commandline
```

| artifact | size |
|---|---:|
| `accent-sass.wasm`, as built | 18.64 MB |
| stripped of debug info | 2.82 MB |
| gzipped | 0.89 MB |

That is the whole of the good news, and it is more than it sounds: `StdFs`
works under WASI, so `@use` and `@import` resolve against preopened
directories with no bridge, no importer callback and no synchronous-read
constraint. The browser package needs a filesystem written for it
([12](12-wasm-browser-package.md) gap 2); this one already has one.

**Nothing here has been run.** No wasm runtime was available on the machine
that measured the above, so "compiles" is verified and "works" is not. Closing
that is gap 1, and it is the substance of this item.

## 1. Nothing runs the WASI build

### Current behavior

No CI job builds for `wasm32-wasip1`, so the target can break silently between
releases. That is exactly how the browser package came to ship a module with
no compiler in it for two releases (`zoosky/accent-sass` #47).

### What it should do

A job that builds the CLI for `wasm32-wasip1` and runs the test suite through
it under wasmtime. The value is in the second half: a build that compiles is
weak evidence, and this project has a suite worth pointing at the artifact.

### Implementation instructions

- Add a job to `.github/workflows/tests.yml`, or a workflow beside
  `build_wasm.yml` if it wants a paths filter of its own.
- Install a wasmtime release, build the CLI for the target, and compile a
  fixture stylesheet with `wasmtime run --dir=. accent-sass.wasm input.scss`,
  comparing the output.
- Then point the spec runner at it. `npm run sass-spec -- --command` takes an
  arbitrary command, so a wrapper script that invokes wasmtime works without
  changing the runner. Expect the tallies to differ from the native run; treat
  any difference as a finding, not as noise, and record the numbers here.
- Decide whether the job gates or is advisory. Advisory to start is defensible,
  the way `sass-spec` and `bootstrap` are; say which in the workflow.

## 2. Path semantics under preopened directories

### Current behavior

`StdFs::canonicalize` calls `std::fs::canonicalize`, and the `Fs` trait's
default implementation returns the path unchanged. Under WASI, paths resolve
against preopened directories rather than a root, and a `..` that escapes a
preopen fails.

### What to check

- A load path outside every preopen must fail with a comprehensible error, not
  a panic or a silent miss.
- `@use "../shared/x"` from inside a preopen must behave the way the native
  build does, or the difference must be recorded here.
- Absolute paths in `--load-path` mean something different under WASI. Decide
  and document how the CLI maps them.

None of this can be settled by reading; it needs the runtime from gap 1.

## 3. Size and the profile

2.82 MB stripped is the CLI including clap. A library-only WASI build for an
embedder is smaller and is the more likely artifact for a plugin host. As with
[12](12-wasm-browser-package.md) gap 4, use a separate release profile rather
than changing the shared one, and measure rather than assume.

## Testing

The CI job is the test. Beyond it, the same fixtures the native suite uses
serve here unchanged, because the artifact is the same compiler.

## Acceptance criteria

- A CI job builds `wasm32-wasip1` and runs at least one real compile under
  wasmtime, comparing output rather than checking an exit code.
- The sass-spec suite has been run through the WASI artifact at least once and
  its tallies recorded here next to the native ones, with any difference
  explained.
- The three path questions in gap 2 are answered in this document.
- Whether the job gates is stated in the workflow.
