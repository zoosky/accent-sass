# Spec conformance roadmap

This directory holds one implementation document per work item that closes
the gap between this project and dart-sass, ranked by the number of
sass-spec tests each item unlocks. The reference is 1.104.0 since
[item 25](25-baseline-before-dart-sass-1-104.md); the ranking below was drawn
up against 1.103.1.

It also holds a second kind of item, added 2026-09-08: **delivery items**,
which unlock no fixtures at all. Items 12, 13 and 14 are the WebAssembly
targets. They are listed under [Delivery items](#delivery-items----no-spec-impact)
and are deliberately kept out of the ranking, which counts failures and would
read them as worthless.

**The ranking below was rebuilt on 2026-09-10, by cause rather than by
area, and re-measured on 2026-09-11.** The original one was drawn up against
1,718 failures, when six large items accounted for most of them. Those six
have landed, and so have items 09, 10, 11, 16, 17, 20, 21 and 23, and now 07
and 24; the suite is at 173, which changed the shape of the problem rather
than just its size.

The 2026-09-05 rebuild reported 134 failures in areas no document covered and
ranked them by *area*, which said where the failures were and nothing about
what caused them. Every one of those has now been read. There are 128 of them
at 284, and they group into 21 causes: the deepest is worth 18 failures across
five areas, the nine deepest cover 89 of the 128, and the rest is a tail of 12
that item 24 records one by one. Items 15 to 24 are that reading. Several of
those items hold more than one defect -- item 17 is five -- so 21 is the count
of things worth queueing, not of lines to change. Items 16, 17, 20, 21, 23 and
24 have since landed, and 40 of the unclaimed failures are left: 39 in items
15, 18, 19 and 22, and item 23's residue, `css/font-face/bubble/empty`.

Counting by cause reorders the work. `spec/core_functions/math` looked like the
deepest unclaimed area at 12 failures; it is five unrelated defects, four of
them a line or two. What actually leads the ranking is a rule about combinators
that shows up in five different areas and in no single area looks big.

## Measurement

The ranking comes from a full run of the pinned sass-spec revision against
the release build. Measured 2026-09-11 on master (`f55ace41`), against
sass-spec `b39c32768`:

```
14266 runs, 14085 passing, 173 failures, 8 todo, 0 ignored, 0 errors
```

The 173 splits 133 in the areas items 01 to 11 claim and 40 in the areas
items 15 to 24 were drawn from.

Item 25 moved the pin from `4a9eea66` to `b39c32768`, the first revision
with dart-sass 1.104.0's expectations, which added 48 runs. The step is
measured rather than inferred. `e8868a5e`, the last master before the move,
measured 175 at the old pin against 1.103.1, so #61 to #82 took the suite
from 234 to 175. Every fixture failing now also failed there; the 48 new
fixtures all pass, and so do two old ones,
`core_functions/color/hwb/four_args/blackness/degenerate/negative_infinity`
and `values/calculation/mod/nan/zero_and_negative_infinity`, which is how
item 25 took it from 175 to 173.

**This page records one measurement and does not keep a running total.** Put
what an item closed in that item's own document, which opens with it; a total
here is edited by every branch that lands, so it conflicts once per branch and
is wrong between one merge and the next full run. It was, briefly: this
paragraph read "the suite is at 277" on a master that measured 243. Rebuild
the page in one deliberate pass after a batch lands, against a fresh run
rather than by arithmetic, which is what this revision does.

The 2026-09-10 revisions were that pass, at 243 and then 234. This revision
is the next, at 173.

Marking the table below has the same hazard in a milder form. A row and the
row under it are one hunk to a three-way merge, so two items landing on
separate branches conflict even though they edited different rows. Mark them
in the same pass as the re-measurement.

The ranking itself was drawn up against the 284 failures master carried
before items 16, 17, 20 and 23 landed. That run's failure list was split by area and
read fixture by fixture; the causes in items 15 to 24 come from that reading,
and each was checked against dart-sass 1.103.1 before being written down. The
per-item "Failures" column below is what each item was worth against that run
-- the size of the job, not a live count -- and the areas table further down
is re-measured at 173. The 284 split 156 claimed against 128 unclaimed; the
earlier figure of 134 was taken at 294 and is not directly comparable, #43 and
#52 having closed fixtures in unclaimed areas without belonging to any item.

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
not enough to name them. Expect CI to read two higher than this page. The
offset still held on 2026-09-11: 177 against 175 on `e8868a5e`, and 175
against 173 on `f55ace41`.

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
`spec/values/calculation` that is 0 failures with the flags, 22 without
`--ignore-warning-diffs`, and 22 with neither `--ignore-*` flag but
`--trim-errors` kept (re-measured 2026-09-11 on
`f55ace41`; it was 3, 25 and 60 on 2026-09-06).
[08](08-calculation-warnings-and-error-wording.md) records them. Every count
on this page is *with* the flags, so each is a floor rather than the whole
gap. Drop the flags when an item's acceptance criteria say so.

## Where the remaining 173 are

Ranked by the failures each item unlocks, deepest first. Items 15 to 24 were
written on 2026-09-10 and between them accounted for all 128 failures that no
document covered. Items 16, 17, 20, 21, 23 and 24 have since landed and have
moved to the table at the end of this page, and so has item 07. The
"Failures" column is what each item was worth against the 284 run: the size
of the job, not a live count. The "Left" column is measured on `f55ace41`.

### Open items

| Doc | Cause | Failures | Left | Main spec directories |
|---|---|---|---|---|
| [15-bogus-combinators.md](15-bogus-combinators.md) | A selector whose combinators cannot match is printed rather than dropped, and may act as an extender | 18 | 18 | `non_conformant/extend-tests`, `directives/extend`, `non_conformant/{scss,sass}` |
| [18-loud-comment-fidelity.md](18-loud-comment-fidelity.md) | A comment loses the line it was written on, and a continuation line loses the output indentation | 9, plus 1 of item 05's residue | 9, plus the 1 | `libsass-closed-issues`, `non_conformant`, `css/keyframes` |
| [19-indented-syntax-gaps.md](19-indented-syntax-gaps.md) | `.sass` parse gaps left after item 10: comments spanning lines, `@import` lists, a bare `@at-root`. Section 2, brackets spanning lines, was closed by #68 | 9, plus 1 of item 05's residue | 6, plus the 1 | `expressions/comments`, `non_conformant/sass`, `directives/at_root` |
| [22-custom-property-raw-text.md](22-custom-property-raw-text.md) | A custom property's value is folded onto one line instead of being reindented | 6 | 6 | `css/custom_properties` |
| [08-calculation-warnings-and-error-wording.md](08-calculation-warnings-and-error-wording.md) | Deprecation warnings (none exist); the error wording is done | 22, invisible under the standard flags | 22, all missing warnings | `spec/values/calculation` |

Items 15 to 24 are not disjoint from the residue the landed items left: items
18 and 19 each close one failure in `spec/css/comment`, which item 05 counts
as its residue. Those two were counted in the 284 run's 156 claimed rather
than its 128 unclaimed, so no number on this page double-counts them.

"Claimed" is by area, which is a rough measure.
`spec/values/calculation/calc/operator/var/calculation` sat in an area two
documents claim and belonged to neither; item 24 recorded it outside its count
of 39, and #79 closed it. Ten failures are like it now: they sit in areas that
landed items claim, and no document gives their cause. Eight are in
`spec/css/comment`, and item 05 lists them as its residue, saying none has
been read for a cause: five loud comments in the indented syntax
(`block/loud/sass/content_after_close/{loud_comment,silent_comment}`,
`block/loud/sass/trailing_whitespace` and
`error/loud/sass/content_after_close/{multi_line,single_line}`) and three
`sourcemap` comments (`between_loads`, `sourcemappingurl` and `sourceurl`).
The other two are in no document: `directives/import/css/unquoted`, in item
06's area, and `expressions/if/syntax/newline/in_css_function`, in item 02's.
All ten are counted as residue below.

### What the areas look like now

The 173 by area, for anyone who wants to scope a run, measured on
`f55ace41`. Areas a document already claims are marked. Since the 234 run,
`css/selector` fell from 32 to 25 and `core_functions/selector` from 34 to 25
while item 24 landed, `core_functions/color` lost its one failure that was not
a rounding difference to item 25, and "everything else" fell from 55 to 18.
No area grew.

| Area | Failures | Claimed by |
|---|---:|---|
| `spec/core_functions/meta` | 37 | 03 |
| `spec/core_functions/selector` | 25 | 04 |
| `spec/css/selector` | 25 | 04 |
| `spec/core_functions/color` | 21 | 01 |
| `spec/css/comment` | 10 | 05, 18, 19 |
| `spec/non_conformant/extend-tests` | 9 | 15 |
| `spec/directives/use` | 8 | 06 |
| `spec/css/custom_properties` | 6 | 22 |
| `spec/directives/forward` | 5 | 06 |
| `spec/directives/extend` | 5 | 15 |
| `spec/expressions/comments` | 4 | 19 |
| everything else | 18 | 15, 18 and 19, plus one failure each of the residue of 02, 06 and 23 |

By kind, the 173 are 143 "Expected did not match output", 14 "Test case
should succeed but it did not" and 16 "Expected test to fail but it did not";
[Failure kinds](#failure-kinds) has the history.

Two residue areas point at an open item. 24 of `css/selector`'s 25 failures
are under `css/selector/combinator/`, and 20 of `core_functions/selector`'s 25
have `combinator` in their path. That is item 15's subject, but item 15 names
none of them, and nobody has checked whether its fix reaches them.

Item 07 has landed in full and moved to the table below: #83, item 25, closed
its section 1, the last, and `spec/values/calculation` is clear under the
standard flags. Item 08's failures are invisible under the standard flags, so
they are outside the 173 entirely. Its **gap 2, the error wording, is
closed** by `zoosky/accent-sass` #52: 36 of its 57 were error text, and the
`spec/values/calculation` area went from 60 to 24 under `--trim-errors` alone.
It is 22 now, and every one of those is a missing deprecation warning: the
same run with `--ignore-warning-diffs` added has none. What remains under item
08 is gap 1, the deprecation warnings, which needs a warning facility the
compiler does not have.

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
spec suite through it stood at 285 failures against the native build's 284 at
the time -- one fixture, a last-digit floating-point difference on a colour
far outside any gamut, where dart-sass does not match the fixture either. The
one-fixture gap has not been re-measured since 2026-09-10.

The pipeline behind item 12 shipped a module with no compiler in it for two
releases: `wasm-exports` is not a default feature and the job never asked for
it, so wasm-bindgen exported nothing and the linker dropped everything.
`zoosky/accent-sass` #47 fixed the build, made the job run the package it
builds, and put it on pull requests. That is the reason item 13's acceptance
criteria insist on *running* the artifact: for a target nothing exercises, a
green build says very little.

### Reference moves -- no ranking impact

Moving to a new dart-sass release changes what the suite expects rather than
closing a gap the ranking counts, so these are kept out of it as well.

| Doc | What it is | State |
|---|---|---|
| [25-baseline-before-dart-sass-1-104.md](25-baseline-before-dart-sass-1-104.md) | 1.103.1 to 1.104.0: negative zero prints as `-0`, and degenerate colour channels become `0`. Also records the baseline measured before the move | landed |

### Landed -- residue only

The residues here sum to 134, re-measured on `f55ace41`: the 133 left in the
areas items 01 to 11 claim, plus the 1 that items 16, 17, 20, 21, 23 and 24
left behind in areas drawn from the 284 run's unclaimed 128. The 39 still open
there belong to items 15, 18, 19 and 22.

These sixteen are done. The counts are what remains in the areas they
touched, not open work, and they are listed so nobody mistakes a residue for
a priority.

| Doc | Landed | Residue | Where |
|---|---|---|---|
| [01-calculation-functions.md](01-calculation-functions.md) | #12 | 21 | `core_functions/color` 21, all double-precision differences; `values/calculation` is clear. Item 11 took the 35 `attr()` fixtures |
| [02-css-if-function.md](02-css-if-function.md) | #13 | 1 | `spec/expressions/if` |
| [03-meta-module.md](03-meta-module.md) | #14 | 37 | `spec/core_functions/meta` |
| [04-selector-unification.md](04-selector-unification.md) | #15 | 50 | `core_functions/selector` 25, `css/selector` 25 -- item 16 took three, and 16 more went while item 24 landed |
| [05-comments-and-arguments.md](05-comments-and-arguments.md) | #16 | 10 | `spec/css/comment`; `spec/callable` is clear. Items 18 and 19 name causes for 2 of the 10 |
| [06-module-system.md](06-module-system.md) | #17, #18 | 14 | `directives/use` 8, `forward` 5, `import` 1 -- item 16 took one |
| [07-calculation-long-tail.md](07-calculation-long-tail.md) | #83 for section 1; sections 2 and 3 by other work | 0 | `spec/values/calculation` is clear |
| [09-plain-css.md](09-plain-css.md) | #27, #29, #30 | 0 | `spec/css/plain` is clear; `directives/import` has 1 left, counted under 06 |
| [10-indented-newlines.md](10-indented-newlines.md) | #32 | 0 | cut across 26 areas; `directives/for`, `directives/function`, `values/lists`, `css/media` and `css/style_rule` are clear |
| [11-special-css-functions.md](11-special-css-functions.md) | #38, #39, #40, #41 | 0 | `css/functions`, `css/percent` and `css/supports` are clear; its 35 colour fixtures were part of item 01's residue |
| [16-top-level-parent-selector.md](16-top-level-parent-selector.md) | #54 | 0 | `libsass/base-level-parent` and `libsass/parent-selector` are clear. Worth 18 rather than the 14 counted for it: it also reached `css/selector`, `core_functions/selector` and `directives/import`, which items 04 and 06 claim |
| [17-math-module.md](17-math-module.md) | #55 | 0 | `core_functions/math` is clear |
| [20-string-split.md](20-string-split.md) | #57 | 0 | `core_functions/string` is clear; #64 closed `split/private_use_character` with item 24's escaping fix |
| [23-font-face-bubbling.md](23-font-face-bubbling.md) | #56 | 1 | `css/font-face`: `bubble/empty`, which also needs item 18's section 1 |
| [21-empty-map-as-list.md](21-empty-map-as-list.md) | #58 | 0 | `core_functions/list` is clear; #72 closed `join/error/named` under item 24. Worth 9 rather than the 8 counted for it: an arglist is a list too, which also closed one of item 24's four arglist fixtures |
| [24-unclaimed-tail.md](24-unclaimed-tail.md) | #61 to #68, #70 to #79 | 0 | All 39 fixtures pass, and so does the calc fixture it kept outside them. Its own document still lists eight causes as open; all eight are closed |

Item 06's residue fell from 29 to 15 through #34, which is not one of these
documents. Residue is not automatically worth chasing either, but it is worth
reading for causes: `core_functions/color`'s 57 were three defects, not
fifty-seven failures. Item 11 has taken 35 of them, and item 25 took the
`calc(NaN)` channel. All 21 left are last-digit double-precision differences
under `to_space` and `to_gamut` (`59264689.52803929` against `...31`), so the
residue is a rounding question, not a colour-API one.

## Failure kinds

Across the whole suite the 173 failures split into (2026-09-11, `f55ace41`):

- 143 "Expected did not match output" — accent-sass produces different CSS.
- 14 "Test case should succeed but it did not" — accent-sass rejects valid input.
- 16 "Expected test to fail but it did not" — accent-sass accepts invalid input.

At 234 the split was 179, 28 and 27.

The order flipped on 2026-09-05. On 2026-09-04 *rejects valid input* stood at
304 and dominated; items 09 and 10 were both drawn from it, and it is now 14.
What dominates now is different output, which is a serialization or semantics
difference rather than a parser gap. Item 16 took 18 of the 48 that column
held at 284, and it was a restriction to delete rather than a feature to
write.

The third kind means accent-sass is systematically more lenient than dart-sass.
Closing those requires adding error checks, not features; several documents
carry a strictness section for their area, and item 24's section 1
collected eight that belonged to no area in particular, which #68 and #72
closed. #34 cut this kind from 43 to 29 by
making a mandatory `@extend` whose target is out of scope an error. Two
fixtures moved the other way, from different output into *rejects valid
input*: `core_functions/meta/load_css/twice/load_css/different_extend` and
`directives/use/extend/scope/use_into_use_and_use_into_import_into_use`. Both
failed before and after, so the shift is a change of kind, not a regression --
diffing the two failure lists turns up no fixture that passed on `659dce0` and
fails now.

## Ground rules for every item

- dart-sass 1.104.0 is the reference (1.103.1 before item 25). Verify
  every new or changed expectation against the real binary before
  committing it; never re-baseline a test to whatever the new code prints.
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
