# Spec conformance roadmap

This directory holds one implementation document per work item that closes
the gap between this project and dart-sass 1.103.1, ranked by the number of
sass-spec tests each item unlocks.

**The ranking below was rebuilt on 2026-09-05 and re-measured on 2026-09-06,
then updated as item 11 landed.**
The original one was drawn up against 1,718 failures, when six large items
accounted for most of them. Those six have landed, and so have items 09 and 10;
the suite is at 298, which changed the shape of the problem rather than just its
size. The documented items are now mostly *residue*, and **134 of the 298 -- 45%
-- sit in areas no document covers at all.** No single area is deep any more:
the deepest unclaimed one is 12 failures, and 83 of the 134 are a tail of areas
holding fewer than six each. Item 11's sections 1 and 2 cut into that tail
without setting out to: one defect in a shared parser reached six areas.

## Measurement

The ranking comes from a full run of the pinned sass-spec revision
(`4a9eea66`) against the release build. Re-measured 2026-09-06 on master
(`be69f6a`) at 375 failures, and again as item 11 landed:

```
14218 runs, 13912 passing, 298 failures, 8 todo, 0 ignored, 0 errors
```

The original ranking was taken on 2026-09-02 at 12,492 passing against 1,718
failures; items 01-06 closed the difference, reaching 650 on `6d43969`. Five
pull requests since then took it to 397: #27 (CSS nesting passthrough) to 590,
#29 (the CSS `@function` rule) to 563, #30 (the rest of plain CSS) to 524, #31
(the function-name proposal) to 511, and #32 (the `consumeNewlines`
parameter) to 397. #34, which scoped `@extend` to a module's upstream closure,
took it to 375; it belongs to no item here, having come off the branch left
after #18. Item 11 took it to 369 with section 5, a bare `%` parsing as a
value (#38), then to 332 with section 4, `attr()` and the CSS `if()` joining
the special variable strings (#39), and then to 298 with sections 1 and 2, a
silent comment dropped and a quoted string's quote character kept (#40). One or two tests depend on `random()`
and move between runs.

Every count on this page comes from a macOS run. The advisory `sass-spec` CI
job runs the same flags on a Linux runner and reports two more failures: 399
on `659dce0` against 397 locally, 377 on `be69f6a` against 375. The offset
held across both commits and the 22-failure delta is identical on either
platform, so the ranking is unaffected, but the two tests behind it are
unidentified -- the job keeps only the last 40 lines of its output, which is
not enough to name them. Expect CI to read two higher than this page.

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
`--ignore-warning-diffs`, 60 with neither (re-measured 2026-09-06, unchanged
since 2026-09-03; [08](08-calculation-warnings-and-error-wording.md) records
them). Every count on this page is *with* the flags, so each is a floor rather
than the whole gap. Drop the flags when an item's acceptance criteria say so.

## Where the remaining 298 are

Ranked by failures under the standard flags, deepest first. "Kind" is the
dominant failure mode in that area, which says what the work is: *rejects
valid input* is a parser or feature gap, *different output* is a
serialization or semantics difference, *accepts invalid input* is a missing
error check.

### Unclaimed -- no document covers these

134 failures, none tracked by any item. Six areas hold six or more, 51 in
all; the other 83 are a tail of 52 areas with fewer than six each.

| Area | Failures | Kind |
|---|---|---|
| `spec/core_functions/math` | 12 | 11 different output |
| `spec/non_conformant/extend-tests` | 10 | 10 different output |
| `spec/core_functions/list` | 9 | 6 different output |
| `spec/core_functions/string` | 8 | 8 different output |
| `spec/css/custom_properties` | 6 | 6 different output |
| `spec/directives/extend` | 6 | 6 different output |

The shape has changed since the 2026-09-04 ranking. That one was led by areas
where this compiler *rejected* input dart-sass accepts, and items 09 and 10
were both cut out of it: `spec/css/plain`, `spec/css/function`,
`spec/css/style_rule`, `spec/directives/function`, `spec/directives/for`,
`spec/values/lists`, `spec/css/media` and `spec/css/supports` are now clear. What is left is mostly *different output* -- a
serialization or semantics difference, not a parser gap -- which is finer work
per failure than the last two items were. #34 then cleared 22, most of them
in areas a document already claims: 13 under `spec/directives/use`, one each
in `directives/forward`, `core_functions/meta` and
`non_conformant/extend-tests`, and six in the unclaimed tail. Item 11 has
since claimed `spec/css/functions`, which led this table at 22, and `spec/css/percent`,
whose 6 gated 35 of item 01's colour residue until section 5 cleared them. Its
sections 1 and 2 then cleared `spec/css/supports` outright and cut
`spec/css/unknown_directive` from 7 to 3 and `spec/css/moz_document` from 5 to
1, none of which any document had claimed.

### Open items

| Doc | Area | Failures | Main spec directories |
|---|---|---|---|
| [07-calculation-long-tail.md](07-calculation-long-tail.md) | What #12 left in the calculation suite: `%` and `mod()` with a signed zero against an infinite divisor, a rounding strategy arriving through interpolation, line noise inside an interpolated `calc()` | 3 | `spec/values/calculation` |
| [08-calculation-warnings-and-error-wording.md](08-calculation-warnings-and-error-wording.md) | Deprecation warnings (none exist) and error wording in calculations | 57, invisible under the standard flags | `spec/values/calculation` |
| [11-special-css-functions.md](11-special-css-functions.md) | One left of five: `type()` never taking the text path. Sections 1, 2, 4 and 5 have landed, taking all 35 colour fixtures, 18 of this area's 22, and 18 more across five areas no document claimed | 4 | `spec/css/functions` |

Item 11's 4 are what is left of the `spec/css/functions` row that led the
unclaimed table until it was written. Its 35 colour failures were part of
item 01's residue below, not additional ones, and have landed. Item 07's 3 are likewise the same three counted
under item 01 -- 07 exists to describe what 01 deliberately left. Item
08's 57 are invisible under the standard flags, so they are outside the 298
entirely and are not double-counted either; that figure was re-measured on
2026-09-06 with the flags dropped and is unchanged.

### Landed -- residue only

The counts here and in the unclaimed table above sum to 298: 164 in areas a
document claims, 134 in areas none does.

These eight are done. The counts are what remains in the areas they touched,
not open work, and they are listed so nobody mistakes a residue for a
priority.

| Doc | Landed | Residue | Where |
|---|---|---|---|
| [01-calculation-functions.md](01-calculation-functions.md) | #12 | 25 | `values/calculation` 3, `core_functions/color` 22 -- item 11 took the 35 `attr()` fixtures; 21 of what is left are double-precision differences |
| [02-css-if-function.md](02-css-if-function.md) | #13 | 1 | `spec/expressions/if` |
| [03-meta-module.md](03-meta-module.md) | #14 | 37 | `spec/core_functions/meta` |
| [04-selector-unification.md](04-selector-unification.md) | #15 | 69 | `core_functions/selector` 35, `css/selector` 34 |
| [05-comments-and-arguments.md](05-comments-and-arguments.md) | #16 | 13 | `spec/css/comment`; `spec/callable` is clear |
| [06-module-system.md](06-module-system.md) | #17, #18 | 15 | `directives/use` 8, `forward` 5, `import` 2 |
| [09-plain-css.md](09-plain-css.md) | #27, #29, #30 | 0 | `spec/css/plain` is clear; `directives/import` has 2 left, counted under 06 |
| [10-indented-newlines.md](10-indented-newlines.md) | #32 | 0 | cut across 26 areas; `directives/for`, `directives/function`, `values/lists`, `css/media` and `css/style_rule` are clear |

Item 06's residue fell from 29 to 15 through #34, which is not one of these
documents. Residue is not automatically worth chasing either, but it is worth
reading for causes: `core_functions/color`'s 57 were three defects, not
fifty-seven failures. Item 11 has taken 35 of them. Of the 22 left, 21 are
last-digit double-precision differences under `to_space` and `to_gamut`
(`59264689.52803929` against `...31`) and one is a `calc(NaN)` channel -- so
the residue is now a rounding question, not a colour-API one.

## Failure kinds

Across the whole suite the 298 failures split into (2026-09-07):

- 219 "Expected did not match output" — accent-sass produces different CSS.
- 50 "Test case should succeed but it did not" — accent-sass rejects valid input.
- 29 "Expected test to fail but it did not" — accent-sass accepts invalid input.

The order flipped on 2026-09-05. On 2026-09-04 *rejects valid input* stood at
304 and dominated; items 09 and 10 were both drawn from it, and it is now 50.
What dominates now is different output, which is a serialization or semantics
difference rather than a parser gap.

The third kind means accent-sass is systematically more lenient than dart-sass.
Closing those requires adding error checks, not features; several documents
carry a strictness section for their area. #34 cut this kind from 43 to 29 by
making a mandatory `@extend` whose target is out of scope an error. Two
fixtures moved the other way, from different output into *rejects valid
input*: `core_functions/meta/load_css/twice/load_css/different_extend` and
`directives/use/extend/scope/use_into_use_and_use_into_import_into_use`. Both
failed before and after, so the shift is a change of kind, not a regression --
diffing the two failure lists turns up no fixture that passed on `659dce0` and
fails now.

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
  cargo +1.96.1 clippy --features=macro --all-targets -- -D warnings
  cargo +stable  clippy --features=macro --all-targets -- -D warnings
  cargo test --features=macro
  ```
- One work item per branch and pull request.
