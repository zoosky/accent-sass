# A WASI build, and a CI job that runs it

`wasm32-wasip1`. One portable artifact that compiles Sass against real files
inside a sandbox the host controls, for anywhere a native binary is unwelcome:
a build service running untrusted themes, a plugin host, a machine that is not
one of the platforms this project ships binaries for.

**This item unlocks no sass-spec fixtures.** It is delivery work, ranked by
who wants the artifact rather than by failure count.

**Gaps 1 and 2 are closed** by `zoosky/accent-sass` #49. The artifact has been
run, the CI job runs it on every push and pull request, and the path questions
have answers taken from a runtime rather than from reading. **Gap 3 is closed**
by `zoosky/accent-sass` #50: there is a library artifact, it is measured, and
CI calls it. Gap 4, the options the C ABI does not take, is open.

## What works

Measured 2026-09-08 and 2026-09-09 on `c3329ac`, aarch64 macOS, wasmtime
28.0.0 and 48.0.1 (both, identical results). The library and the command-line binary compile for
`wasm32-wasip1` with no source changes:

```bash
rustup target add wasm32-wasip1
cargo build --release -p accent-sass --target wasm32-wasip1 --features commandline
```

| artifact | size |
|---|---:|
| `accent-sass.wasm`, as built | 18.64 MiB |
| stripped of debug info | 2.82 MiB |
| gzipped | 0.89 MiB |

The target has a second shape. The command module above is what a host runs
like a process; the library module is what a host instantiates once and calls
into, and it is smaller -- 1.31 MiB, 0.44 MiB gzipped. Gap 3 has the build
command, the full comparison and the ABI.

`StdFs` works under WASI, so `@use` and `@import` resolve against preopened
directories with no bridge, no importer callback and no synchronous-read
constraint. The browser package needs a filesystem written for it
([12](12-wasm-browser-package.md) gap 2); this one already has one.

A compile under `wasmtime run --dir=.` produces output identical to the native
build's, byte for byte, on every input tried.

## 1. Nothing ran the WASI build -- closed

### What was wrong

No CI job built for `wasm32-wasip1`, so the target could break silently between
releases. That is exactly how the browser package came to ship a module with
no compiler in it for two releases (`zoosky/accent-sass` #47).

### What shipped

A `wasi` job in `.github/workflows/tests.yml`: it installs a pinned wasmtime,
builds the CLI for the target, and runs `.github/scripts/wasi-smoke.sh`, which
compiles three stylesheets under the runtime and compares the output. **The job
gates.** It is fast and deterministic, and the target has no other coverage at
all -- nothing else in the workflow would notice a filesystem call that starts
failing under WASI, because every one of them compiles either way.

The third check is the one worth having. It compiles a stylesheet whose
`@use` reaches outside every preopen, and asserts the compiler's own error:

```
Error: Can't find stylesheet to import.
```

Not a panic, not a silent miss. That the sandbox denies the read *and says so*
is the property the whole target is for, so it is pinned rather than assumed.

## 2. Path semantics under preopened directories -- closed

Three questions, each answered by running it. wasmtime 28.0.0 and 48.0.1 agree.

**A load path outside every preopen fails comprehensibly.** `@use
"../shared/x"` under `--dir=site`, where the preopen does not cover
`../shared`, gives `Can't find stylesheet to import.` with the source span --
the same error a missing file gives natively. The sandbox is reported as a
missing file, which is what it looks like from inside.

**`..` inside a preopen behaves as it does natively.** The same stylesheet
under `--dir=.`, where the preopen covers both directories, resolves and
compiles. The boundary is the preopen, not the `..`.

**An absolute `--load-path` needs a preopen mapped onto it.** A bare host path
fails, because the guest has no `/Users/...`:

```
Error: Can't find stylesheet to import.
```

Preopening the host root does not rescue it. `--dir=/` and `--dir=/::/` both
leave the run failing, and less legibly -- a raw `No such file or directory
(os error 44)` instead of the compiler's message. What works is mapping the
directory and passing the guest name:

```bash
wasmtime run --dir=. --dir=/host/shared::/shared module.wasm --load-path=/shared input.scss
```

The alias itself is free: `--dir=/host/shared::/host/shared` works too. An
earlier draft of this document claimed a deep alias failed where a short one
worked. **That was wrong** -- it came from a probe with a second variable in
it, and re-running both forms on wasmtime 28.0.0 and 48.0.1 shows they behave
identically. The rule an embedder has to follow is only the first sentence:
map a preopen onto every load path, under whatever name you like.

## 3. Size and the profile -- closed

### What was wrong

Every figure recorded for this target described the command module, so every
one of them included clap and an argument parser no embedder ever calls.
Nothing said what a plugin host would actually carry, and nothing built such a
thing. The workspace even had a `small` profile already, inherited from
upstream, that no build used.

### What shipped

**A library artifact with something in it.** A `wasi-exports` feature adds a C
ABI in `crates/lib/src/wasi_exports.rs` -- `accent_sass_alloc`,
`accent_sass_dealloc`, `accent_sass_compile_string`,
`accent_sass_compile_path`, `accent_sass_result_free` -- which turns the
`cdylib` into a reactor: instantiated once, called many times, no process
startup per compile.

The exports are what make the number mean anything. A `cdylib` that exports
nothing is dead code to the linker, and that is exactly how the browser
package once measured 0.22 MB while containing no compiler at all
([12](12-wasm-browser-package.md), "What works today"). Measuring a
library-only build without pinning the compiler into it would have repeated
that mistake with a smaller number.

```bash
cargo build --profile small -p accent-sass --target wasm32-wasip1 \
  --no-default-features --features wasi-exports,random
```

**The `small` profile, not a change to `release`.** `opt-level = 'z'`, fat
LTO, one codegen unit, `panic = 'abort'`, `strip = true`. It stays separate
because `opt-level = 'z'` costs native compile speed for a win only a module
cares about, which is what [12](12-wasm-browser-package.md) gap 4 asks for.

**A host that calls it, in CI.** `.github/scripts/wasi-lib-smoke.mjs`
instantiates the module under Node's WASI and drives the ABI: two compiles in
one instance, `@use` resolved through a preopened directory, an import that
leaves a narrower preopen, a compile error, and bytes that are not UTF-8.
wasmtime runs the command module in the same job but cannot run this one --
its CLI has no way to write bytes into the guest's memory, and every call here
needs to.

The memory handling is tested natively as well, in the crate's own unit tests.
A leaked result or a mismatched free is invisible to a smoke test that
compiles once and exits; it shows up in a host as an instance that grows.
`wasi-exports` is therefore in the feature set the gating jobs use.

### Measured

2026-09-09, aarch64 macOS, rustc 1.97.0-nightly (14196dbfa 2026-04-12). Sizes
are MiB; "stripped" is `RUSTFLAGS=-C strip=debuginfo`, which is what the
`small` profile does at link time; gzip is `gzip -9`.

| module | profile | size | gzipped |
|---|---|---:|---:|
| command, `--features commandline` | `release` | 18.64 | 4.15 |
| command | `release`, stripped | 2.82 | 0.89 |
| command | `small` | 1.39 | 0.50 |
| library, `--no-default-features --features wasi-exports,random` | `release` | 16.41 | 3.58 |
| library | `release`, stripped | 2.42 | 0.75 |
| library | `small` | **1.31** | **0.44** |
| library | `small`, then `wasm-opt -Oz` | 1.10 | 0.43 |

The command module's stripped release row reproduces 2026-09-08's figures to
the byte, so the two sessions' numbers can be compared. The toolchain barely
matters here either: the same library build on 1.96.1, the MSRV the `wasi` job
pins, is 1,374,929 bytes against nightly's 1,376,528.

**The profile is the lever; the interface is not.** Dropping the command-line
interface saves 0.40 MiB against the stripped release build, and 0.08 MiB --
5% -- against the `small` one. Changing the profile saves 1.11 MiB, 46%. The
premise this gap was written on, that a library-only build is the smaller
artifact, is true and nearly beside the point. Take the library build because
a host wants exports rather than a process, not because of its size.

**Compression flattens what is left.** The two `small` modules are 0.06 MiB
apart gzipped.

**`wasm-opt -Oz` buys 0.21 MiB, and 0.01 MiB after gzip.** It is worth running
if the host loads the module uncompressed, and hard to justify otherwise, so
it is not in CI. Pass the feature flags explicitly if you do run it: binaryen
112 rejects the module by default (`Bulk memory operations require bulk
memory [--enable-bulk-memory]`), and `--all-features` produced a module that
Node then refused to compile (`invalid value type 0x0`). This worked, and the
smoke test passes against its output:

```bash
wasm-opt -Oz --enable-bulk-memory --enable-sign-ext --enable-mutable-globals \
  --enable-nontrapping-float-to-int --enable-multivalue --enable-reference-types \
  accent_sass.wasm -o accent_sass.opt.wasm
```

### What was not measured

`random` is on in every library figure, so the module carries `rand` and the
`random()` and `unique-id()` builtins, exactly as the command module does.
Turning it off was not measured. Nor was brotli, for want of the tool -- the
same omission [12](12-wasm-browser-package.md) gap 4 records.

## 4. The C ABI compiles with default options -- open

`accent_sass_compile_string` and `accent_sass_compile_path` build
`Options::default()`, so a host cannot choose an output style, declare the
input to be the indented syntax, add a load path or silence a warning. It gets
expanded CSS or a formatted error.

This is the gap the browser binding has ([12](12-wasm-browser-package.md) gap
1) in the other target's clothes, and the fix has the same shape: an owned
configuration the host fills in before the call. Under WASI it is the smaller
job of the two -- scalars and string offsets in linear memory, with no
wasm-bindgen and no JavaScript types to agree on -- and it should follow
whatever names item 12 settles on, so the two artifacts do not diverge.

Do it when an embedder asks for it.

## The spec suite under WASI

Run once in full, 2026-09-09, against the pinned sass-spec revision
`4a9eea66` and the same flags the roadmap uses everywhere else:

```
14218 runs, 13925 passing, 285 failures, 8 todo, 0 ignored, 0 errors
```

One more failure than the native build's 284, and the lists were diffed rather
than the totals compared: **one fixture added, none removed.**

The one is `spec/core_functions/color/to_space/oklch/lab/out_of_range/far`,
which converts `oklch(10% 999999 0deg)` -- a chroma far outside any gamut --
into lab, and lands on numbers around 7.7e16:

| | first channel |
|---|---|
| fixture, and the native build | `76838084903189984` |
| the WASI build | `76838084903190000` |
| dart-sass 1.103.1 | `76838084903189980` |

Last-digit floating-point, from a different `libm` compiled into the module
than the host's. Note the third row: dart-sass does not match the fixture here
either, so this input has no stable answer across implementations, let alone
across targets. It is not worth chasing, and it is the *only* difference the
whole suite finds between the two targets.

### The trap in measuring this

The first attempt reported 328 failures, and 44 of them were the harness, not
the compiler. The wrapper preopened only the test's own directory, so a fixture
starting `@use '../test-hue' as *` escaped the sandbox and failed with the
error gap 2 describes -- correct sandbox behaviour, wrong conclusion. The tell
was the shape of the diff: 44 added and none removed, which is what a harness
fault looks like. Extracting one of them and running it directly gave output
identical to the native build's.

The wrapper now rewrites the entry point to a path under the mapped spec root,
so a relative import resolves inside a preopen instead of escaping one. Anyone
pointing a sandboxed compiler at a suite of relative imports will meet this;
the fix is to give the sandbox the tree, not the leaf.

It also refuses to run rather than falling back. If no `--load-path` covers the
working directory -- a different runner, a different argument order -- the
wrapper exits 78 with a message instead of passing the cwd-relative path
through, because the fallback produces exactly the phantom failures above and
they read as a compiler that cannot find a file.


`.github/scripts/wasi-sass.sh` presents the module to the spec runner as if it
were a native binary, so the measurement can be re-taken:

```bash
WASMTIME=/path/to/wasmtime \
WASM=target/wasm32-wasip1/release/accent-sass.wasm \
npm run sass-spec -- --impl=dart-sass --command ../.github/scripts/wasi-sass.sh \
  --trim-errors --ignore-warning-diffs --ignore-error-diffs
```

It is not in CI. A full run takes about twenty-three minutes against three for
the native build, because each of the 14,218 tests pays wasmtime's startup: a
single compile costs 0.085s under the runtime against 0.005s native, and the
difference is module load rather than execution.

## Acceptance criteria

- [x] A CI job builds `wasm32-wasip1` and runs at least one real compile under
      wasmtime, comparing output rather than checking an exit code.
- [x] The sass-spec suite has been run through the WASI artifact and its
      tallies recorded here next to the native ones, with any difference
      explained.
- [x] The three path questions in gap 2 are answered in this document.
- [x] Whether the job gates is stated in the workflow.
- [x] Gap 3: a library-only profile, measured. The comparison is in gap 3, and
      the `wasi` job builds the artifact, calls its exports and prints its size
      on every run.
