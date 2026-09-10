# Spec conformance roadmap

This directory holds one implementation document per work item that closes
the gap between this project and dart-sass 1.103.1, ranked by the number of
sass-spec tests each item unlocks.

It also holds a second kind of item, added 2026-09-08: **delivery items**,
which unlock no fixtures at all. Items 12, 13 and 14 are the WebAssembly
targets. They are listed under [Delivery items](#delivery-items----no-spec-impact)
and are deliberately kept out of the ranking, which counts failures and would
read them as worthless.

**The ranking below was rebuilt on 2026-09-10, by cause rather than by
area.** The original one was drawn up against 1,718 failures, when six large
items accounted for most of them. Those six have landed, and so have items 09,
10 and 11; the suite is at 284, which changed the shape of the problem rather
than just its size.

The 2026-09-05 rebuild reported 134 failures in areas no document covered and
ranked them by *area*, which said where the failures were and nothing about
what caused them. Every one of those has now been read. There are 128 of them
at 284, and they group into 21 causes: the deepest is worth 18 failures across
five areas, the nine deepest cover 89 of the 128, and the rest is a tail of 12
that item 24 records one by one. Items 15 to 24 are that reading. Several of
those items hold more than one defect -- item 17 is five -- so 21 is the count
of things worth queueing, not of lines to change.

Counting by cause reorders the work. `spec/core_functions/math` looked like the
deepest unclaimed area at 12 failures; it is five unrelated defects, four of
them a line or two. What actually leads the ranking is a rule about combinators
that shows up in five different areas and in no single area looks big.

## Measurement

The ranking comes from a full run of the pinned sass-spec revision
(`4a9eea66`) against the release build. Re-measured 2026-09-10 on master
(`31361d7`):

```
14218 runs, 13926 passing, 284 failures, 8 todo, 0 ignored, 0 errors
```

Every count on this page is from that run, so the per-area numbers below are
current rather than carried over from the 294 run. The failure list it produced
was split by area and read fixture by fixture; the causes in items 15 to 24 come
from that reading, and each was checked against dart-sass 1.103.1 before being
written down.

The 284 splits 156 in areas a document claims and 128 in areas none does. The
earlier figure of 134 was taken at 294 and is not directly comparable: #43 and
#52 both closed fixtures in unclaimed areas without belonging to any item.

**Item 16 has since landed and the suite is at 266.** Every count below is
still the 284 measurement, so subtract item 16's eighteen when reading them:
fourteen from the areas its row names, and four from areas items 04 and 06
claim (`css/selector` 34 to 32, `core_functions/selector` 35 to 34,
`directives/import` 2 to 1).

The original ranking was taken on 2026-09-02 at 12,492 passing against 1,718
failures; items 01-06 closed the difference, reaching 650 on `6d43969`. Five
pull requests since then took it to 397: #27 (CSS nesting passthrough) to 590,
#29 (the CSS `@function` rule) to 563, #30 (the rest of plain CSS) to 524, #31
(the function-name proposal) to 511, and #32 (the `consumeNewlines`
parameter) to 397. #34, which scoped `@extend` to a module's upstream closure,
took it to 375; it belongs to no item here, having come off the branch left
after #18. Item 11 took it to 369 with section 5, a bare `%` parsing as a
value (#38), then to 332 with section 4, `attr()` and the CSS `if()` joining
the special variable strings (#39), then to 298 with sections 1 and 2, a silent
comment dropped and a quoted string's quote character kept (#40), and then to
294 with section 3, `type()` taking the text path (#41). One or two tests depend on `random()`
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

## Where the remaining 284 are

Ranked by the failures each item unlocks, deepest first. Items 15 to 24 were
written on 2026-09-10 and between them account for all 128 failures that no
document covered; nothing in the suite is unread now.

### Open items

| Doc | Cause | Failures | Main spec directories |
|---|---|---|---|
| [15-bogus-combinators.md](15-bogus-combinators.md) | A selector whose combinators cannot match is printed rather than dropped, and may act as an extender | 18 | `non_conformant/extend-tests`, `directives/extend`, `non_conformant/{scss,sass}` |
| [16-top-level-parent-selector.md](16-top-level-parent-selector.md) | `&` at the top level is an error here and passes through in dart-sass, which dropped the restriction -- **landed**, and worth 18 rather than 14 | 14 | `libsass/base-level-parent`, `libsass-closed-issues` |
| [17-math-module.md](17-math-module.md) | Five defects: a fuzzy zero, `clamp()`'s comparison ladder, one missing unit check, `$min-number`, `unit()`'s brackets | 12 | `core_functions/math` |
| [18-loud-comment-fidelity.md](18-loud-comment-fidelity.md) | A comment loses the line it was written on, and a continuation line loses the output indentation | 9, plus 1 of item 05's residue | `libsass-closed-issues`, `non_conformant`, `css/keyframes` |
| [19-indented-syntax-gaps.md](19-indented-syntax-gaps.md) | Four `.sass` parse gaps left after item 10: comments and brackets spanning lines, `@import` lists, a bare `@at-root` | 9, plus 1 of item 05's residue | `expressions/comments`, `parser/indentation`, `directives/at_root` |
| [20-string-split.md](20-string-split.md) | `string.split` mishandles an empty separator, an empty string and quotedness | 8 | `core_functions/string` |
| [21-empty-map-as-list.md](21-empty-map-as-list.md) | A map is not interchangeable with its list of pairs, and the empty map is not the empty list | 8 | `core_functions/list` |
| [22-custom-property-raw-text.md](22-custom-property-raw-text.md) | A custom property's value is folded onto one line instead of being reindented | 6 | `css/custom_properties` |
| [23-font-face-bubbling.md](23-font-face-bubbling.md) | A nested `@font-face` gets a copy of the parent selector, which dart-sass exempts it from | 5 | `css/font-face` |
| [24-unclaimed-tail.md](24-unclaimed-tail.md) | Twelve causes, none worth its own document: strictness checks, arglist separators, `@extend` result sets and nine more | 39 | scattered |
| [07-calculation-long-tail.md](07-calculation-long-tail.md) | What #12 left in the calculation suite: `mod()` with a signed zero against an infinite divisor. Its sections 2 and 3 now pass | 1 | `spec/values/calculation` |
| [08-calculation-warnings-and-error-wording.md](08-calculation-warnings-and-error-wording.md) | Deprecation warnings (none exist); the error wording is done | 22, invisible under the standard flags | `spec/values/calculation` |

Items 15 to 24 are not disjoint from the residue the landed items left: items
18 and 19 each close one failure in `spec/css/comment`, which item 05 counts
as its residue. Those two are counted in the 156 claimed, not in the 128, so
no number on this page double-counts them.

"Claimed" is by area, and one fixture shows why that is a rough measure:
`spec/values/calculation/calc/operator/var/calculation` sits in an area two
documents claim and belongs to neither -- `calc(1 + (var(--c)))` loses its
inner parentheses, which is neither item 07's long tail nor item 08's wording.
Item 24 records it outside its count of 39.

### What the areas look like now

The same 284 by area, for anyone who wants to scope a run. Areas a document
already claims are marked.

| Area | Failures | Claimed by |
|---|---:|---|
| `spec/core_functions/meta` | 37 | 03 |
| `spec/core_functions/selector` | 35 | 04 |
| `spec/css/selector` | 34 | 04 |
| `spec/core_functions/color` | 22 | 01 |
| `spec/core_functions/math` | 12 | 17 |
| `spec/non_conformant/extend-tests` | 10 | 15, 24 |
| `spec/css/comment` | 10 | 05, 18, 19, 24 |
| `spec/core_functions/list` | 9 | 21, 24 |
| `spec/directives/use` | 8 | 06 |
| `spec/core_functions/string` | 8 | 20 |
| `spec/directives/extend` | 6 | 15, 24 |
| `spec/css/custom_properties` | 6 | 22 |
| `spec/non_conformant/scss` | 5 | 15, 18, 24 |
| `spec/directives/forward` | 5 | 06 |
| `spec/css/font-face` | 5 | 23 |
| everything else | 72 | mostly 16, 19 and 24 |

The failure *kind* has settled where item 11 left it. Across the whole suite
the 284 split into 208 "Expected did not match output", 48 "Test case should
succeed but it did not" and 28 "Expected test to fail but it did not". The
middle column is the one items 09 and 10 were drawn from; item 16 has since
taken 18 of its 48, leaving 30.

Item 11 has landed in full and moved to the table below. Item 07's 1 is also
counted under item 01 -- 07 exists to describe what 01 deliberately left, and
two of its three sections now pass. Item 08's failures are invisible under the
standard flags, so they are outside the 284 entirely and are not double-counted
either. Its **gap 2, the error wording, is
closed** by `zoosky/accent-sass` #52: 36 of its 57 were error text, and the
`spec/values/calculation` area is now at 24 under `--trim-errors` alone
against 60 before. What remains under item 08 is gap 1, the deprecation
warnings, which needs a warning facility the compiler does not have.

Gap 2 turned out to be more than wording. Eleven of its tests were a
case-sensitivity difference in unit names -- dart-sass prints `1Q` where this
compiler printed `1q` -- which is a CSS output bug the standard flags never
saw. The measurement is in that item; the lesson is that a family of failures
grouped by their symptom can hide a cause of a different kind.

Until 2026-09-10 both open items were calculation work, so the tracked queue
was one area deep while 128 failures sat outside it unread. Items 15 to 24
close that gap: the queue is now the whole suite.

### Delivery items -- no spec impact

Packaging and target support. None of these changes what the compiler accepts
or prints, so none of them moves a fixture; they are ranked by who wants the
artifact. Kept apart from the table above so a reader looking for conformance
work is not sent to them, and so a reader looking for the WebAssembly story
finds it recorded rather than folklore.

| Doc | What it is | State |
|---|---|---|
| [12-wasm-browser-package.md](12-wasm-browser-package.md) | `wasm32-unknown-unknown` for npm: options, a JS-supplied filesystem, structured errors, size | open |
| [13-wasm-wasi.md](13-wasm-wasi.md) | `wasm32-wasip1`, and a CI job that runs the artifact rather than only building it | closed: gaps 1 and 2 by #49, gap 3 by #50, gap 4 by #51 |
| [14-wasm-component-model.md](14-wasm-component-model.md) | `wasm32-wasip2` and a WIT interface for Accent's plugin runtime | recorded, not queued -- build it only when one of its triggers fires |

Item 13 has since been measured end to end: the WASI artifact runs, and the
spec suite through it stands at 285 failures against the native build's 284 --
one fixture, a last-digit floating-point difference on a colour far outside
any gamut, where dart-sass does not match the fixture either.

The pipeline behind item 12 shipped a module with no compiler in it for two
releases: `wasm-exports` is not a default feature and the job never asked for
it, so wasm-bindgen exported nothing and the linker dropped everything.
`zoosky/accent-sass` #47 fixed the build, made the job run the package it
builds, and put it on pull requests. That is the reason item 13's acceptance
criteria insist on *running* the artifact: for a target nothing exercises, a
green build says very little.

### Landed -- residue only

The counts here sum to 156, the share of the 284 in areas a document already
claims; items 15 to 24 hold the other 128.

These nine are done. The counts are what remains in the areas they touched,
not open work, and they are listed so nobody mistakes a residue for a
priority.

| Doc | Landed | Residue | Where |
|---|---|---|---|
| [01-calculation-functions.md](01-calculation-functions.md) | #12 | 24 | `values/calculation` 2, `core_functions/color` 22 -- item 11 took the 35 `attr()` fixtures; 21 of what is left are double-precision differences |
| [02-css-if-function.md](02-css-if-function.md) | #13 | 1 | `spec/expressions/if` |
| [03-meta-module.md](03-meta-module.md) | #14 | 37 | `spec/core_functions/meta` |
| [04-selector-unification.md](04-selector-unification.md) | #15 | 69 | `core_functions/selector` 35, `css/selector` 34 |
| [05-comments-and-arguments.md](05-comments-and-arguments.md) | #16 | 10 | `spec/css/comment`; `spec/callable` is clear. Items 18 and 19 name causes for 2 of the 10 |
| [06-module-system.md](06-module-system.md) | #17, #18 | 15 | `directives/use` 8, `forward` 5, `import` 2 |
| [09-plain-css.md](09-plain-css.md) | #27, #29, #30 | 0 | `spec/css/plain` is clear; `directives/import` has 2 left, counted under 06 |
| [10-indented-newlines.md](10-indented-newlines.md) | #32 | 0 | cut across 26 areas; `directives/for`, `directives/function`, `values/lists`, `css/media` and `css/style_rule` are clear |
| [11-special-css-functions.md](11-special-css-functions.md) | #38, #39, #40, #41 | 0 | `css/functions`, `css/percent` and `css/supports` are clear; its 35 colour fixtures were part of item 01's residue |

Item 06's residue fell from 29 to 15 through #34, which is not one of these
documents. Residue is not automatically worth chasing either, but it is worth
reading for causes: `core_functions/color`'s 57 were three defects, not
fifty-seven failures. Item 11 has taken 35 of them. Of the 22 left, 21 are
last-digit double-precision differences under `to_space` and `to_gamut`
(`59264689.52803929` against `...31`) and one is a `calc(NaN)` channel -- so
the residue is now a rounding question, not a colour-API one.

## Failure kinds

Across the whole suite the 284 failures split into (2026-09-10):

- 208 "Expected did not match output" — accent-sass produces different CSS.
- 48 "Test case should succeed but it did not" — accent-sass rejects valid input.
- 28 "Expected test to fail but it did not" — accent-sass accepts invalid input.

The order flipped on 2026-09-05. On 2026-09-04 *rejects valid input* stood at
304 and dominated; items 09 and 10 were both drawn from it, and it is now 48.
What dominates now is different output, which is a serialization or semantics
difference rather than a parser gap. Item 16 is the largest thing left in the
middle column, at 14 of the 48, and it is a restriction to delete rather than a
feature to write.

The third kind means accent-sass is systematically more lenient than dart-sass.
Closing those requires adding error checks, not features; several documents
carry a strictness section for their area, and item 24's section 1 collects
eight that belong to no area in particular. #34 cut this kind from 43 to 29 by
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
