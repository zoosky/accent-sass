# Spec conformance roadmap

This directory holds one implementation document per work item that closes
the gap between this project and dart-sass 1.103.1, ranked by the number of
sass-spec tests each item unlocks.

**The ranking below was rebuilt on 2026-09-05.** The original one was drawn up
against 1,718 failures, when six large items accounted for most of them. Those
six have landed, and so have items 09 and 10; the suite is at 397, which
changed the shape of the problem rather than just its size. The documented
items are now mostly *residue*, and **187 of the 397 -- 47% -- sit in areas no
document covers at all.** No single area is deep any more: the largest
unclaimed one is 22 failures, and 94 of the 187 are a tail of areas with fewer
than six each.

## Measurement

The ranking comes from a full run of the pinned sass-spec revision
(`4a9eea66`) against the release build. Re-measured 2026-09-05, on the item 10
head:

```
14218 runs, 13813 passing, 397 failures, 8 todo, 0 ignored, 0 errors
```

The original ranking was taken on 2026-09-02 at 12,492 passing against 1,718
failures; items 01-06 closed the difference, reaching 650 on `6d43969`. Five
pull requests since then took it to 397: #27 (CSS nesting passthrough) to 590,
#29 (the CSS `@function` rule) to 563, #30 (the rest of plain CSS) to 524, #31
(the function-name proposal) to 511, and #32 (the `consumeNewlines`
parameter) to 397. One or two tests depend on `random()` and move between
runs.

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

## Where the remaining 397 are

Ranked by failures under the standard flags, deepest first. "Kind" is the
dominant failure mode in that area, which says what the work is: *rejects
valid input* is a parser or feature gap, *different output* is a
serialization or semantics difference, *accepts invalid input* is a missing
error check.

### Unclaimed -- no document covers these

187 failures, none tracked by any item. The ten deepest areas are 93 of them;
the other 94 are a tail of 59 areas with fewer than six each.

| Area | Failures | Kind |
|---|---|---|
| `spec/css/functions` | 22 | 18 different output |
| `spec/core_functions/math` | 12 | 11 different output |
| `spec/non_conformant/extend-tests` | 11 | 11 different output |
| `spec/core_functions/list` | 9 | 6 different output |
| `spec/core_functions/string` | 8 | 8 different output |
| `spec/css/unknown_directive` | 7 | 7 different output |
| `spec/css/custom_properties` | 6 | 6 different output |
| `spec/css/percent` | 6 | 6 rejects valid input |
| `spec/css/supports` | 6 | 6 different output |
| `spec/directives/extend` | 6 | 6 different output |

The shape has changed since the 2026-09-04 ranking. That one was led by areas
where this compiler *rejected* input dart-sass accepts, and items 09 and 10
were both cut out of it: `spec/css/plain`, `spec/css/function`,
`spec/css/style_rule`, `spec/directives/function`, `spec/directives/for`,
`spec/values/lists` and `spec/css/media` are now clear, and `spec/css/supports`
went from 21 to 6. What is left is mostly *different output* -- a
serialization or semantics difference, not a parser gap -- which is finer work
per failure than the last two items were.

### Open items

| Doc | Area | Failures | Main spec directories |
|---|---|---|---|
| [07-calculation-long-tail.md](07-calculation-long-tail.md) | What #12 left in the calculation suite: `%` and `mod()` with a signed zero against an infinite divisor, a rounding strategy arriving through interpolation, line noise inside an interpolated `calc()` | 3 | `spec/values/calculation` |
| [08-calculation-warnings-and-error-wording.md](08-calculation-warnings-and-error-wording.md) | Deprecation warnings (none exist) and error wording in calculations | 57, invisible under the standard flags | `spec/values/calculation` |

Item 07's 3 failures are the same three counted in item 01's residue below,
not additional ones -- 07 exists to describe what 01 deliberately left. Item
08's 57 are invisible under the standard flags, so they are outside the 397
entirely and are not double-counted either; that figure has not been
re-measured since 2026-09-03 and needs a run with the flags dropped.

### Landed -- residue only

The counts here and in the unclaimed table above sum to 397: 210 in areas a
document claims, 187 in areas none does.

These eight are done. The counts are what remains in the areas they touched,
not open work, and they are listed so nobody mistakes a residue for a
priority.

| Doc | Landed | Residue | Where |
|---|---|---|---|
| [01-calculation-functions.md](01-calculation-functions.md) | #12 | 60 | `values/calculation` 3, `core_functions/color` 57 -- mostly `calc(infinity)`/`calc(NaN)` channels |
| [02-css-if-function.md](02-css-if-function.md) | #13 | 1 | `spec/expressions/if` |
| [03-meta-module.md](03-meta-module.md) | #14 | 38 | `spec/core_functions/meta` |
| [04-selector-unification.md](04-selector-unification.md) | #15 | 69 | `core_functions/selector` 35, `css/selector` 34 |
| [05-comments-and-arguments.md](05-comments-and-arguments.md) | #16 | 13 | `spec/css/comment`; `spec/callable` is clear |
| [06-module-system.md](06-module-system.md) | #17, #18 | 29 | `directives/use` 21, `forward` 6, `import` 2 |
| [09-plain-css.md](09-plain-css.md) | #27, #29, #30 | 0 | `spec/css/plain` is clear; `directives/import` has 2 left, counted under 06 |
| [10-indented-newlines.md](10-indented-newlines.md) | #32 | 0 | cut across 26 areas; `directives/for`, `directives/function`, `values/lists`, `css/media` and `css/style_rule` are clear |

Residue is not automatically worth chasing. `core_functions/color`'s 57 are
mostly one cause -- calculation keywords passed as a channel -- so they are
better read as one defect than fifty-seven.

## Failure kinds

Across the whole suite the 397 failures split into (2026-09-05):

- 261 "Expected did not match output" — accent-sass produces different CSS.
- 93 "Test case should succeed but it did not" — accent-sass rejects valid input.
- 43 "Expected test to fail but it did not" — accent-sass accepts invalid input.

The order has flipped. On 2026-09-04 the first kind was 304 and dominated;
items 09 and 10 were both drawn from it, and it is now 93. What dominates now
is different output, which is a serialization or semantics difference rather
than a parser gap.

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
