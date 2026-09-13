# Releasing

`accent-sass` publishes three crates to crates.io and one package to npm. The
crates are version-locked with `=` pins, so they go out together, in dependency
order. The npm package, `@zoosky/accent-sass`, is the WebAssembly build of the
same version, and goes out after them.

`.github/scripts/release.sh` does all of it. Run it with `--dry-run` first; the
rest of this document is what it checks and why.

## Order is not optional

```
1. accent_sass_compiler   (no workspace dependencies)
2. accent-sass-macro      (depends on accent_sass_compiler)
3. accent-sass            (depends on both)
4. @zoosky/accent-sass    (npm; built from accent-sass)
```

Cargo resolves a `path` + `version` dependency against the registry when
packaging, so a crate cannot even be packaged until the one below it is live.
Running `cargo package -p accent-sass` before the other two are published fails
with `no matching package named accent_sass_compiler found`. That is expected,
not a misconfiguration.

Allow a minute between publishes for the index to update.

The npm package has no registry dependency on the crates, so it could go first.
It goes last so that a crate that fails to publish never leaves an npm version
behind that the crates do not match.

## Before you publish

1. **Gates.** Clippy runs on both the MSRV and stable, matching CI and
   `release.sh`; the MSRV alone cannot see lints added after it. The feature
   set matches CI too: `wasi-exports` is in it so the WebAssembly C ABI in
   `crates/lib/src/wasi_exports.rs` is linted and tested on the host, rather
   than only in the `wasi` job.

   ```bash
   cargo fmt --all -- --check
   cargo +1.96.1 clippy --features=macro,wasi-exports --all-targets -- -D warnings
   cargo +stable  clippy --features=macro,wasi-exports --all-targets -- -D warnings
   cargo test --features=macro,wasi-exports
   cargo audit
   ```

   `cargo audit` exits non-zero for a vulnerability and zero for a
   warning-level advisory, such as a crate that is merely unmaintained. A
   warning is worth reading before a release without being a reason to stop
   one, which is the behaviour `release.sh` relies on.

2. **Versions.** All three crates carry the same version, and two `=` pins
   reference it:

   | File | What to bump |
   |---|---|
   | `crates/compiler/Cargo.toml` | `version` |
   | `crates/accent-sass-macro/Cargo.toml` | `version`, and the `accent_sass_compiler` `=` pin |
   | `crates/lib/Cargo.toml` | `version`, and the `accent_sass_compiler` and `accent-sass-macro` pins |

   Six edits. `cargo check` then refreshes `Cargo.lock`. The npm package takes
   its version from `crates/lib/Cargo.toml`, so it needs no edit of its own.

3. **Changelog.** Move `## [Unreleased]` into a dated version heading. The
   format is [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the
   versioning is [SemVer](https://semver.org/spec/v2.0.0.html); while the major
   version is `0`, a breaking change bumps the minor.

4. **Third-party names.** `include_sass` on crates.io belongs to upstream
   `grass`; this fork's proc macro is `accent-sass-macro` for that reason. Check
   any new crate name is free before adding it.

5. **Dry run** each crate:

   ```bash
   cargo package -p accent_sass_compiler --list
   ```

   `README.md` in `crates/lib` and `crates/compiler` is a symlink to the root
   README; cargo follows it and ships the real content. If a crate has no
   in-package `README.md`, cargo publishes without one rather than failing.

   `include` patterns follow `.gitignore` rules, so each one starts with `/`
   to stay at the crate root. `release.sh` fails if a crate would package a
   file git ignores.

6. **npm.** You need `wasm-pack`, the `wasm32-unknown-unknown` target, and an
   npm login that may publish under the `@zoosky` scope:

   ```bash
   rustup target add wasm32-unknown-unknown
   npm login
   ```

   `release.sh` builds the package into `target/npm/pkg`, not wasm-pack's
   default `crates/lib/pkg`, and checks it before anything is published: the
   manifest names `@zoosky/accent-sass` at the release version, the two smoke
   scripts CI runs against the Pages build pass, and `npm pack --dry-run` lists
   the README, the LICENSE, the JavaScript, the types and the module.

## Publish

```bash
.github/scripts/release.sh
```

It prompts once, then publishes the three crates, then the npm package, then
tags. By hand, the same steps are:

```bash
cargo publish -p accent_sass_compiler
cargo publish -p accent-sass-macro
cargo publish -p accent-sass

wasm-pack build crates/lib --release --target web --scope zoosky --out-name index \
  --out-dir ../../target/npm/pkg -- --no-default-features --features wasm-exports,random
(cd target/npm/pkg && npm publish --access public)

git tag -a v0.16.0 -m "v0.16.0"
git push origin v0.16.0
```

A scoped package is private unless published with `--access public`.

### Publishing only the npm package

```bash
.github/scripts/release.sh --npm-only --dry-run
.github/scripts/release.sh --npm-only
```

For a version whose crates and tag already exist: one released before npm
joined this process, or one whose npm step failed after the crates went out. It
requires the tag, and refuses to build if the compiler sources have changed
since it, so the package is the code the crates were published from. It skips
the Rust gates and the tag, and still builds and checks the package.

npm allows unpublishing a version only within 72 hours, and never lets a
version number be reused.

## After

Bump Accent's pin. Accent depends on this fork by git revision rather than by
version (see its `Cargo.toml`), so a crates.io release does not reach it
automatically. Per Accent's rule 21, bump the pin only once the `frameworks`
job is green on the exact revision being pinned.
