# Spec conformance roadmap

This directory holds one implementation document per work item that closes
the gap between this fork and dart-sass 1.103.1, ranked by the number of
sass-spec tests each item unlocks.

## Measurement

The ranking comes from a full run of the pinned sass-spec revision
(`4a9eea66`) against the release build. Re-measured 2026-09-04, after items
01-06 landed:

```
14218 runs, 13560 passing, 650 failures, 8 todo, 0 ignored, 0 errors
```

The original ranking was taken on 2026-09-02 at 12,492 passing against 1,718
failures. Items 01-06 are what closed the difference.

To reproduce:

```bash
git submodule update --init sass-spec
cargo build --release
cd sass-spec && npm install --no-audit --no-fund
npm run sass-spec -- --impl=dart-sass --command '../target/release/accent-sass' \
  --trim-errors --ignore-warning-diffs --ignore-error-diffs
```

Scope a run to one area by appending its spec path, for example
`spec/values/calculation`.

The two `--ignore-*` flags hide real differences: a test that only fails
on a missing deprecation warning or on the wording of an error counts as
passing. In `spec/values/calculation` on the #12 head that is 3 failures
with the flags, 25 without `--ignore-warning-diffs`, 60 with neither
(measured 2026-09-03; [08](08-calculation-warnings-and-error-wording.md)
records them). Drop the flags when an item's acceptance criteria say so.

## Work items

| Doc | Area | Status | Failing tests (2026-09-04) | Main spec directories |
|---|---|---|---|---|
| [01-calculation-functions.md](01-calculation-functions.md) | CSS math functions and constants in calculations | landed (#12) | 60 | `spec/values/calculation`, `spec/core_functions/color` |
| [02-css-if-function.md](02-css-if-function.md) | The CSS `if()` function | landed (#13) | 1 | `spec/expressions/if` |
| [03-meta-module.md](03-meta-module.md) | `sass:meta` mixin reflection and `load-css` strictness | landed (#14) | 38 | `spec/core_functions/meta` |
| [04-selector-unification.md](04-selector-unification.md) | Selector unification ordering | landed (#15) | 37 | `spec/core_functions/selector` |
| [05-comments-and-arguments.md](05-comments-and-arguments.md) | Comment positions and argument-list syntax | landed (#16) | 0 under `spec/callable` | `spec/callable`, `spec/css`, `spec/directives` |
| [06-module-system.md](06-module-system.md) | `@use`, `@forward` and `@import` edge cases | landed (#17, #18) | 42 | `spec/directives/use`, `spec/directives/forward`, `spec/directives/import` |
| [07-calculation-long-tail.md](07-calculation-long-tail.md) | What #12 leaves in the calculation suite: `%` and `mod()` with a signed zero against an infinite divisor, a rounding strategy that arrives through interpolation, line noise inside an interpolated `calc()` | open | 3 | `spec/values/calculation` |
| [08-calculation-warnings-and-error-wording.md](08-calculation-warnings-and-error-wording.md) | Deprecation warnings (none exist) and error wording in calculations; invisible under the standard flags | open | 57 hidden (not re-measured 2026-09-04) | `spec/values/calculation` |

Items 01-06 have landed, taking the suite from 12,492 passing to 13,560. No
single area dominates the remaining 650: the largest groups are
`core_functions/color` (57, mostly `calc(infinity)`/`calc(NaN)` channels),
`css/plain` (51), `core_functions/meta` (38) and `core_functions/selector`
(37), followed by a broad tail.

## Failure kinds

Across the whole suite the 650 failures split into (2026-09-04):

- 304 "Test case should succeed but it did not" — accent-sass rejects valid input.
- 294 "Expected did not match output" — accent-sass produces different CSS.
- 52 "Expected test to fail but it did not" — accent-sass accepts invalid input.

The third kind means accent-sass is systematically more lenient than dart-sass.
Closing those requires adding error checks, not features; several documents
below carry a strictness section for their area.

## Ground rules for every item

- dart-sass 1.103.1 is the reference. Verify every new or changed
  expectation against the real binary before committing it; never
  re-baseline a test to whatever the new code prints.
- Add regression tests to `crates/lib/tests/` using the `test!` and
  `error!` macros alongside the spec run.
- Run the quality gates before committing. Clippy runs on **two** toolchains
  and both gate, because pinning the lint gate to the MSRV alone left it
  unable to see any lint added after 1.85 -- sixteen findings sat in the tree
  while every job reported clean:

  ```bash
  cargo fmt --all -- --check
  cargo +1.85.0 clippy --features=macro --all-targets -- -D warnings
  cargo +stable  clippy --features=macro --all-targets -- -D warnings
  cargo test --features=macro
  ```
- One work item per branch and pull request.
