# Spec conformance roadmap

This directory holds one implementation document per work item that closes
the gap between this project and dart-sass 1.103.1, ranked by the number of
sass-spec tests each item unlocks.

**The ranking below was rebuilt on 2026-09-04.** The original one was drawn up
against 1,718 failures, when six large items accounted for most of them. Those
six have landed and the suite is at 650, which changed the shape of the problem
rather than just its size: the documented items are now mostly *residue*, and
**425 of the 650 -- 65% -- sit in areas no document covers at all.** A roadmap
that lists only the eight existing documents would point a contributor at the
smaller half.

## Measurement

The ranking comes from a full run of the pinned sass-spec revision
(`4a9eea66`) against the release build. Re-measured 2026-09-04, after items
01-06 landed:

```
14218 runs, 13560 passing, 650 failures, 8 todo, 0 ignored, 0 errors
```

Measured on `6d43969`. The original ranking was taken on 2026-09-02 at 12,492
passing against 1,718 failures; items 01-06 closed the difference. CI reports
13,558 on the same tree -- one or two tests depend on `random()` and move
between runs.

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

The two `--ignore-*` flags hide real differences: a test that only fails on a
missing deprecation warning or on the wording of an error counts as passing. In
`spec/values/calculation` that is 3 failures with the flags, 25 without
`--ignore-warning-diffs`, 60 with neither (measured 2026-09-03 on the #12 head;
[08](08-calculation-warnings-and-error-wording.md) records them). Every count on
this page is *with* the flags, so each is a floor rather than the whole gap.
Drop the flags when an item's acceptance criteria say so.

## Where the remaining 650 are

Ranked by failures under the standard flags, deepest first. "Kind" is the
dominant failure mode in that area, which says what the work is: *rejects
valid input* is a parser or feature gap, *different output* is a
serialization or semantics difference, *accepts invalid input* is a missing
error check.

### Unclaimed -- no document covers these

This is where the work is: 425 failures, none tracked by any item. The ten
deepest areas are 192 of them; the other 233 are a long tail of areas with
fewer than eleven each.

`spec/css/plain` used to lead this table with 51. It is now item
[09](09-plain-css.md) and sits under "Open items" below with 24 left, so the
unclaimed total is 374.

| Area | Failures | Kind | Note |
|---|---|---|---|
| `spec/css/functions` | 24 | 20 different output | |
| `spec/css/supports` | 21 | 15 rejects valid input | |
| `spec/css/style_rule` | 17 | 9 rejects valid input | |
| `spec/css/function` | 16 | 15 rejects valid input | |
| `spec/directives/function` | 15 | 12 rejects valid input | |
| `spec/values/lists` | 13 | 13 rejects valid input | |
| `spec/core_functions/math` | 12 | 11 different output | |
| `spec/directives/for` | 12 | 12 rejects valid input | |
| `spec/css/media` | 11 | 10 rejects valid input | |

The `css/*` rows together are about 90 failures and mostly one theme: input
that dart-sass accepts and this compiler does not -- the same theme as item
09. `spec/css/function` and `spec/directives/function` are the CSS `@function`
rule, which is also what most of item 09's remainder is; those 31 and item
09's 8 are one feature and are best done together.

### Open items

| Doc | Area | Failures | Main spec directories |
|---|---|---|---|
| [07-calculation-long-tail.md](07-calculation-long-tail.md) | What #12 left in the calculation suite: `%` and `mod()` with a signed zero against an infinite divisor, a rounding strategy arriving through interpolation, line noise inside an interpolated `calc()` | 3 | `spec/values/calculation` |
| [08-calculation-warnings-and-error-wording.md](08-calculation-warnings-and-error-wording.md) | Deprecation warnings (none exist) and error wording in calculations | 57, invisible under the standard flags | `spec/values/calculation` |
| [09-plain-css.md](09-plain-css.md) | Plain CSS. Nesting passthrough landed in #27 and closed 27 of the original 51; what is left is the CSS `@function` rule, whitespace in `@import ... supports(..)`, `if()` in plain CSS, and `//` in a plain CSS value | 24 | `spec/css/plain` |

Item 07's 3 failures are the same three counted in item 01's residue below,
not additional ones -- 07 exists to describe what 01 deliberately left. Item
08's 57 are invisible under the standard flags, so they are outside the 650
entirely and are not double-counted either; that figure has not been
re-measured since 2026-09-03 and needs a run with the flags dropped.

### Landed -- residue only

The counts here and in the unclaimed table above sum to 650: 225 in areas a
document claims, 425 in areas none does. Both figures predate item 09, which
moved 51 out of the unclaimed column and closed 27 of them.

These six are done. The counts are what remains in the areas they touched,
not open work, and they are listed so nobody mistakes a residue for a
priority.

| Doc | Landed | Residue | Where |
|---|---|---|---|
| [01-calculation-functions.md](01-calculation-functions.md) | #12 | 60 | `values/calculation` 3, `core_functions/color` 57 -- mostly `calc(infinity)`/`calc(NaN)` channels |
| [02-css-if-function.md](02-css-if-function.md) | #13 | 1 | `spec/expressions/if` |
| [03-meta-module.md](03-meta-module.md) | #14 | 38 | `spec/core_functions/meta` |
| [04-selector-unification.md](04-selector-unification.md) | #15 | 71 | `core_functions/selector` 37, `css/selector` 34 |
| [05-comments-and-arguments.md](05-comments-and-arguments.md) | #16 | 13 | `spec/css/comment`; `spec/callable` is clear |
| [06-module-system.md](06-module-system.md) | #17, #18 | 42 | `directives/use` 24, `forward` 13, `import` 5 |

Residue is not automatically worth chasing. `core_functions/color`'s 57 are
mostly one cause -- calculation keywords passed as a channel -- so they are
better read as one defect than fifty-seven.

## Failure kinds

Across the whole suite the 650 failures split into (2026-09-04):

- 304 "Test case should succeed but it did not" — accent-sass rejects valid input.
- 294 "Expected did not match output" — accent-sass produces different CSS.
- 52 "Expected test to fail but it did not" — accent-sass accepts invalid input.

The first kind dominates, and it concentrates in `spec/css/*`: this compiler
rejects input dart-sass accepts. That is why plain CSS led the unclaimed table
when the ranking was drawn up, and why item 09 came out of it.

The third kind means accent-sass is systematically more lenient than dart-sass.
Closing those requires adding error checks, not features; several documents
carry a strictness section for their area.

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
