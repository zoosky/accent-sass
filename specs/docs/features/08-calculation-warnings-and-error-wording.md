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

## Gap 2: error wording

### Current behavior

35 tests fail only because accent-sass's error message differs from
dart-sass's on the first line, which is what `--trim-errors` compares.
Six families:

| count | dart-sass | accent-sass | tests |
|---:|---|---|---|
| 17 | `This operation can't be used in a calculation.` | `This expression can't be used in a calculation.` (16) and `expected "+", "-", "*", "/", ",", or ")".` (1) | every `<function>/error/sass_script`, `calc-size/error/sass_script`, `calc/error/syntax/unknown_operator` |
| 11 | `1Q and 1deg are incompatible.` | `1q and 1deg are incompatible.` | `calc/error/known_incompatible/length/q/*` |
| 2 | `Expected expression.` | `Expected number, variable, function, or calculation.` | `calc/error/syntax/trailing_operator`, `double_operator` |
| 2 | `Value (1 2 3) can't be used in a calculation.` | `Value 1 2 3 can't be used in a calculation.` | `calc/error/value/variable/list`, `function/list` |
| 1 | `This expression can't be used in a calculation.` | `Expected digit.` | `calc/error/syntax/leading_operator` |
| 1 | `Expected identifier.` | `Expected number, variable, function, or calculation.` | `calc/error/syntax/hash` |
| 1 | `Rest arguments can't be used with calculations.` | `This expression can't be used in a calculation.` | `clamp/error/syntax/rest` |

### Implementation instructions

1. dart-sass distinguishes an *operation* that cannot be simplified
   into a calculation (a Sass-script binary expression such as
   `sqrt(1 + 2px)` where the operands are incompatible) from an
   *expression* that has no place in one. Match that split at the site
   that produces `This expression can't be used in a calculation.`
2. The `Q` unit keeps its canonical uppercase spelling in messages; the
   fix is in unit display, not in the calculation code, and it should
   be checked against every other place a unit name is printed.
3. A list value is inspected with its parentheses in this message
   (`(1 2 3)`); use the same inspection dart-sass's `inspect()` gives a
   space-separated list in an error.
4. The remaining four are parser-position messages; take each from its
   `.hrx` file and match the position as well as the text, since a
   later `--trim-errors` run still checks the first line.

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

- `spec/values/calculation` passes with `--trim-errors` alone, apart
  from the three tests in 07 until that item lands.
- The warning facility is documented in the crate's README with the
  ids it implements, and Accent's `styling` feature says whether they
  are shown.
- No change to the standard-flags tally or to the `frameworks` job.
