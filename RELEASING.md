# Releasing

`accent-sass` publishes three crates. They are version-locked with `=` pins, so
they go out together, in dependency order.

## Order is not optional

```
1. accent_sass_compiler   (no workspace dependencies)
2. accent-sass-macro      (depends on accent_sass_compiler)
3. accent-sass            (depends on both)
```

Cargo resolves a `path` + `version` dependency against the registry when
packaging, so a crate cannot even be packaged until the one below it is live.
Running `cargo package -p accent-sass` before the other two are published fails
with `no matching package named accent_sass_compiler found`. That is expected,
not a misconfiguration.

Allow a minute between publishes for the index to update.

## Before you publish

1. **Gates.** Clippy runs on both the MSRV and stable, matching CI and
   `release.sh`; the MSRV alone cannot see lints added after it:

   ```bash
   cargo fmt --all -- --check
   cargo +1.96.1 clippy --features=macro --all-targets -- -D warnings
   cargo +stable  clippy --features=macro --all-targets -- -D warnings
   cargo test --features=macro
   ```

2. **Versions.** All three crates carry the same version, and two `=` pins
   reference it:

   | File | What to bump |
   |---|---|
   | `crates/compiler/Cargo.toml` | `version` |
   | `crates/accent-sass-macro/Cargo.toml` | `version`, and the `accent_sass_compiler` `=` pin |
   | `crates/lib/Cargo.toml` | `version`, and the `accent_sass_compiler` and `accent-sass-macro` pins |

   Six edits. `cargo check` then refreshes `Cargo.lock`.

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

## Publish

```bash
cargo publish -p accent_sass_compiler
cargo publish -p accent-sass-macro
cargo publish -p accent-sass
```

Then tag:

```bash
git tag -a v0.14.0 -m "v0.14.0"
git push origin v0.14.0
```

## After

Bump Accent's pin. Accent depends on this fork by git revision rather than by
version (see its `Cargo.toml`), so a crates.io release does not reach it
automatically. Per Accent's rule 21, bump the pin only once the `frameworks`
job is green on the exact revision being pinned.
