# Deprecation warnings and error wording in calculations

Unlocks 57 sass-spec tests under `spec/values/calculation` that the
roadmap's standard flags never count: 22 fail only on a missing
deprecation warning (`--ignore-warning-diffs`) and 35 only on the wording
of an error (`--ignore-error-diffs`). Measured on 2026-09-03 against the
`zoosky/accent-sass` #12 head (`4946548`), the pinned sass-spec revision
`4a9eea66`, and the dart-sass 1.103.1 binary:

| flags | failures |
|---|---:|
| `--trim-errors --ignore-warning-diffs --ignore-error-diffs` (standard) | 3 |
| `--trim-errors --ignore-error-diffs` | 25 |
| `--trim-errors` | 60 |

The 3 are [07-calculation-long-tail.md](07-calculation-long-tail.md).
This document is the other 57. They are zero in the published tally, so
they rank last by the roadmap's own measure; they are recorded because
the warnings are the mechanism by which a user learns that a stylesheet
will stop compiling on Dart Sass 2.0 or 3.0, and Accent's users write
against current Dart Sass.

**Gap 2 is closed** by `zoosky/accent-sass` #52. Re-measured 2026-09-09 on
`f198518`, the same area and flags:

| flags | before | after |
|---|---:|---:|
| standard | 2 | 2 |
| `--trim-errors --ignore-error-diffs` | 24 | 24 |
| `--trim-errors` | 60 | 24 |

The 36 the gap describes all pass. What is left under `--trim-errors` alone
is gap 1's 22 warnings and item 07's 2. The counts here are one lower than the
2026-09-03 figures above because item 07's third test has since started
passing, and one higher for gap 2 -- 36 rather than 35 -- for the same reason.

Gap 1, the deprecation warnings, is open.

## Gap 1: no deprecation warnings at all

### Current behavior

accent-sass emits no deprecation warnings. The compiler crate contains no
`DEPRECATION WARNING` string, and none of the five deprecations the
calculation tests expect is implemented anywhere. Every one of the 22
warning failures has the same diff: dart-sass prints a warning on
standard error, accent-sass prints nothing.

### Reference behavior

dart-sass 1.103.1 prints these in the calculation tests, each with the
deprecation's id in brackets, a `See https://sass-lang.com/d/<id>` line,
and the source snippet:

| id | count | when |
|---|---:|---|
| `global-builtin` | 5 | a global built-in function is called at all: "Global built-in functions are deprecated and will be removed in Dart Sass 3.0.0." (`abs/sass_script`, `round/one_argument/sass_script`, `round/one_argument/calc_unsafe_in_binary_operator`, `calc/no_operator/function/min`, `calc/no_operator/function/max`) |
| `global-builtin` | 8 | `min()` or `max()` mixes a unitless number with a real unit: "In future versions of Sass, max() will be interpreted as the CSS max() calculation. This doesn't allow unitless numbers to be mixed with numbers with units. If you want to use the Sass function, call math.max() instead." (the `unitless_and_real` groups) |
| `global-builtin` | 2 | `round()` gets a number with units and no step: "This requires an explicit modulus when rounding numbers with units." (`round/one_argument/preserves_units`, `preserves_single_unit`) |
| `global-builtin` | 1 | `abs()` mixes units: `abs/preserves_single_unit` |
| `slash-div` | 4 | `/` is division beside `abs()`, `min()`, `max()` or `round()`: "Using / for division outside of calc() is deprecated and will be removed in Dart Sass 2.0.0." with a `math.div(...)` recommendation |
| `abs-percent` | 1 | `abs(-7.5%)`: "Passing percentage units to the global abs() function is deprecated." |
| `if-function` | 1 | the Sass `if()` syntax: "deprecated in favor of the modern CSS if()" (`calc/no_operator/function/if`) |

These are the only deprecations the calculation area exercises. The
full dart-sass set is larger (the `sass-spec` runner's
`--ignore-warning-diffs` exists because most implementations lag it),
and a general warning facility is the right shape: the `slash-div`
warning alone fires on every legacy `/` division in every stylesheet.

### Implementation instructions

1. Add a deprecation channel to the compiler's warning output, keyed by
   dart-sass's deprecation ids, so the text, the `See` line and the
   snippet render the way dart-sass renders them. The spec compares the
   warning text exactly (modulo the runner's normalization), so take
   every message from the `.hrx` files.
2. Emit `global-builtin` from the global function lookup, with the
   three message variants above chosen by what the call did: a plain
   call, a `min`/`max`/`abs` that mixed units, or a `round` that took a
   number with units and no step. #12 already routes these names
   through the Sass function when the calculation cannot express the
   call; the warning belongs at that point.
3. Emit `slash-div` where `/` is treated as division outside a
   calculation, with the recommendation line built from the two
   operands' source text.
4. Emit `abs-percent` from the global `abs()` when the argument is a
   percentage, and `if-function` from the legacy `if()`.
5. Decide, and document, whether the warnings are on by default in the
   library API and in Accent's `styling` feature. dart-sass prints them
   by default and offers `--silence-deprecation`/`--fatal-deprecation`;
   Accent's users will see them in `accent build` output.

## Gap 2: error wording -- closed

### What was wrong

36 tests failed only because accent-sass's error message differed from
dart-sass's on the first line, which is what `--trim-errors` compares. Seven
families:

| count | dart-sass | accent-sass | tests |
|---:|---|---|---|
| 17 | `This operation can't be used in a calculation.` | `This expression can't be used in a calculation.` (16) and `expected "+", "-", "*", "/", ",", or ")".` (1) | every `<function>/error/sass_script`, `calc-size/error/sass_script`, `calc/error/syntax/unknown_operator` |
| 11 | `1Q and 1deg are incompatible.` | `1q and 1deg are incompatible.` | `calc/error/known_incompatible/length/q/*` |
| 2 | `Expected expression.` | `Expected number, variable, function, or calculation.` | `calc/error/syntax/trailing_operator`, `double_operator` |
| 2 | `Value (1 2 3) can't be used in a calculation.` | `Value 1 2 3 can't be used in a calculation.` | `calc/error/value/variable/list`, `function/list` |
| 1 | `This expression can't be used in a calculation.` | `Expected digit.` | `calc/error/syntax/leading_operator` |
| 1 | `Expected identifier.` | `Expected number, variable, function, or calculation.` | `calc/error/syntax/hash` |
| 1 | `expected ")".` | `Expected number, variable, function, or calculation.` | `calc/error/syntax/interpolation/line_noise` |
| 1 | `Rest arguments can't be used with calculations.` | `This expression can't be used in a calculation.` | `clamp/error/syntax/rest` |

### The `Q` family was not an error-wording problem

**This document's instruction 2 was wrong.** It said the `Q` unit "keeps its
canonical uppercase spelling in messages" and that the fix was in unit
display. It is neither. Probing dart-sass 1.103.1 directly:

| input | dart-sass |
|---|---|
| `a { b: 1Q; c: 1q; d: 1PX }` | `1Q`, `1q`, `1PX` -- the source spelling, unchanged |
| `math.div(1q, 1mm)` | `0.25` |
| `math.div(1Q, 1mm)` | `calc(1Q / 1mm)` -- no conversion |
| `math.div(1kHz, 1Hz)` | `1000` |
| `math.div(1kHz, 1hz)` | `calc(1kHz / 1hz)` |
| `1Q + 1q` | `Error: 1Q and 1q have incompatible units.` |

**Unit names are case-sensitive in dart-sass**, which CSS is not. Only the
canonical spelling is a known unit; every other casing is an unknown unit that
never converts and prints back as written. accent-sass lowercased every unit
at parse time, so `Q` became the quarter-millimetre unit and printed as `q` --
a CSS output bug that no fixture under the standard flags catches, hiding
behind eleven that looked like message wording.

There is one exception, and it has to be kept: the check that decides whether
`calc(1Q + 1deg)` is an error lowercases first. Measured, again directly:
`calc(1Q + 1mm)` compiles, `calc(1Q + 1deg)` and `calc(1HZ + 1deg)` do not,
and `calc(1foo + 1deg)` does, an unknown name being compatible with anything.
So `Q` converts like an unknown unit and counts as a length for that one
question. `Unit::ignoring_case` is that exception and says so.

### What shipped

**A case-sensitive unit table.** One table of spellings, lowercase-keyed;
`Unit::from` accepts a name only when it matches the canonical casing exactly,
and `Unit::ignoring_case` is the single caller that ignores case, used by the
known-compatibility check alone.

**An operation is not an expression.** dart-sass parses a calculation with its
ordinary expression parser and validates afterwards, which is why the two
messages exist. A CSS math function whose arguments are not calculation syntax
reaches the evaluator as an ordinary call, so `disallowed_in_calculation` walks
the argument tree there: it descends through the four calculation operators
and through parentheses, reports any other binary operator as an *operation*
at the operator, and anything else as an *expression*. `sqrt(7 % 3 + "a")`
reports the `%` and `sqrt("a" + 1)` reports the string, both checked against
dart-sass.

**The parser reports position, not a menu.** `Expected number, variable,
function, or calculation.` appears nowhere in the spec suite -- it is a stale
dart-sass message this fork inherited from upstream. What replaces it depends
on where the operand was wanted, because dart-sass's parser is committed by
then or is not: at the start of an argument nothing is committed, so `calc(,)`
is a malformed call (`expected ")".`), while an operator has promised an
operand, so `calc(1px *)` is a missing one (`Expected expression.`). That is
`OperandPosition`. A `#` reports `Expected identifier.` in either position,
because it begins an interpolation or a hex colour and the parser is already
reading a name.

**Values that parse and are then refused.** A quoted string, `()`, a hex
colour and a unary `/` are expressions a calculation cannot hold rather than
parse failures, so they are parsed and rejected as such: `calc("a")`,
`calc(())`, `calc((((())))`, `calc(#fff)`, `calc(/ 1px)` and `calc(/1px)` all
say so now.

A `#` is the fiddly one, because what follows decides whether dart-sass got
far enough to have an expression at all. `calc(#)`, `calc(# )`, `calc(#,)` and
`calc(#-)` are missing identifiers; `#fff`, `#zzz` and `#\65` are expressions;
a digit run is a hex colour of 3, 4, 6 or 8 digits, so `#123` and `#12345678`
are expressions while `#1`, `#12`, `#12345` and `#1234567` want another hex
digit. All twenty cases were taken from dart-sass 1.103.1.

**A bare list is parenthesised in the value error**, and only there:
`Value (1 2 3)` for `$a: 1 2 3` against `Value [1 2 3]` for a bracketed one.
A one-element space list counts -- `list.append((), 1px)` gives `Value (1px)`
-- while a one-element comma list is exempt, because `inspect` already writes
it as `(1px,)`.

**How the argument list is written outranks what is in it.** A calculation
takes neither keyword nor rest arguments, and dart-sass says which before it
looks at what they would expand to: `sqrt($x: 7 % 3)` is refused for the `$x:`
and never reaches the `%`. Keyword outranks rest, and a `$map...` counts as a
rest argument.

### Measured

Full suite, standard flags: **284 before and after**. The failure *lists* were
diffed rather than the totals compared: **37 fixed, none added.** The 37th is
`spec/css/plain/error/expression/calculation/line_noise`, outside this item --
the same parser defect reached the plain CSS path.

The `frameworks` corpus is unaffected: bulma 9, pico 0, foundation 44, uswds
917 differing lines, 0 colour-bearing, identical to master's run on the same
machine. The unit-case change was the one to watch there, since it alters what
a stylesheet writing `1PX` compiles to.

### What this deliberately did not fix

Three divergences found while probing, none covered by a fixture, all still
open:

| input | dart-sass | accent-sass |
|---|---|---|
| `calc(1px and 2px)` | `This operation can't be used in a calculation.` | `calc(1px and 2px)` |
| `calc(not 1px)` | `This expression can't be used in a calculation.` | `calc(not 1px)` |
| `sqrt(7 and 3)` | `This operation can't be used in a calculation.` | `sqrt(7 and 3)` |
| `calc(1px & 2px)` | `This expression can't be used in a calculation.` | `expected ")".` |

The word operators read as identifiers, so the argument becomes a
space-separated list and is carried through to the output; `&` is dart-sass's
parent selector, an expression a calculation cannot hold. Fixing either means
changing what the calculation parser accepts, not what it says when it
refuses, so it is a behaviour change with no test behind it. Recorded here
rather than attempted.

One difference is in the caret rather than the message. dart-sass underlines
the operator in `This operation can't be used in a calculation.`; on the
evaluator's path this underlines the whole operation, because the expression
parser keeps its operators on a stack without their spans and `BinaryOpExpr`
carries only the merged one. Giving it the narrower caret means threading an
operator span through that parser, which no fixture asks for -- `--trim-errors`
compares the first line. The parser's own path already points at the operator.

## Testing

- Ground truth: `spec/values/calculation` at the pinned revision, run
  with `--trim-errors --ignore-error-diffs` for Gap 1 and with
  `--trim-errors` alone for both gaps (see [README.md](README.md) for
  the standard invocation to start from).
- Verify every warning and error text against the dart-sass 1.103.1
  binary before pinning it; the `.hrx` files are derived from that
  binary, and the binary is the tie-breaker when they disagree.
- Add `test!`/`error!` cases to `crates/lib/tests/` for one instance of
  each family above.

## Acceptance criteria

- [ ] `spec/values/calculation` passes with `--trim-errors` alone, apart
      from the tests in 07 until that item lands. Gap 2's half is done: the
      area is at 24 under that flag, all of them gap 1's warnings and 07's
      two.
- [ ] The warning facility is documented in the crate's README with the
      ids it implements, and Accent's `styling` feature says whether they
      are shown. Gap 1's, so still open.
- [x] No change to the standard-flags tally or to the `frameworks` job. Both
      measured for gap 2: 284 either way with no fixture added or removed, and
      the framework diffs identical to master's.
