# The calculation long tail

Unlocks the 3 sass-spec tests left under `spec/values/calculation` by
[01-calculation-functions.md](01-calculation-functions.md) (`zoosky/grass`
#12), under the roadmap's standard flags. Three separate defects, one of
them a real bug in the `%` operator that reaches every stylesheet, not
only the spec. Measured on 2026-09-03 against the #12 head (`4946548`),
the pinned sass-spec revision `4a9eea66`, and the dart-sass 1.103.1
binary:

```
987 runs, 984 passing, 3 failures, 0 todo, 0 ignored, 0 errors
```

The warning and error-wording differences the standard flags hide are a
separate item, [08-calculation-warnings-and-error-wording.md](08-calculation-warnings-and-error-wording.md).

## 1. Positive zero against an infinite divisor

`spec/values/calculation/mod/nan/zero_and_negative_infinity`

### Current behavior

`modulo()` in `crates/compiler/src/value/number.rs` (around line 428)
decides the infinite-divisor case by comparing the signs of the operands,
and counts **every** zero as negative:

```rust
// Zero counts as negative here, which is what Dart Sass 1.103.1 does
// (`0 % infinity` is NaN while `0 % -infinity` is 0). dart-sass master
// has since changed the positive-zero case; this matches the released
// reference implementation.
let is_negative = |n: f64| n.is_sign_negative() || n == 0.0;
```

The comment is wrong, and so is the pull request body of #12, which says
the spec test "cannot pass against dart-sass 1.103.1" because the pinned
spec "encodes a later change". The spec file has not changed since
2023-12-08 (sass-spec `de5b60b`), and the released 1.103.1 binary gives
the spec's answer. Both `%` and `mod()` are affected, since `mod()`
simplifies through the same function; two of the eight sign combinations
are wrong:

| input | dart-sass 1.103.1 | grass (#12) |
|---|---|---|
| `0 % calc(-infinity)` | `calc(NaN)` | `0` |
| `0 % calc(infinity)` | `0` | `calc(NaN)` |
| `-0 % calc(infinity)` | `calc(NaN)` | `calc(NaN)` |
| `-0 % calc(-infinity)` | `0` | `0` |
| `mod(0, -infinity)` | `calc(NaN)` | `0` |
| `mod(0, infinity)` | `0` | `calc(NaN)` |
| `mod(-0, infinity)` | `calc(NaN)` | `calc(NaN)` |
| `mod(-0, -infinity)` | `0` | `0` |
| `5 % calc(-infinity)`, `mod(5, -infinity)` | `calc(NaN)` | `calc(NaN)` |

### Reference behavior

`moduloLikeSass` in `lib/src/util/number.dart` at the 1.103.1 tag:

```dart
if (num1.isInfinite) return double.nan;
if (num2.isInfinite) {
  return num1.signIncludingZero == num2.sign ? num1 : double.nan;
}
```

`signIncludingZero` gives positive zero the sign `+1` and negative zero
`-1`. So a zero dividend keeps its own sign, and the result is the
dividend when that sign matches the divisor's, `NaN` otherwise. That is
also what the table above measures.

### Implementation instructions

1. In `modulo()`, replace the closure with a comparison of
   `n1.is_sign_negative()` and `n2.is_sign_negative()`; a positive zero
   is positive. Delete the comment that claims otherwise.
2. Add the eight cases above to `crates/lib/tests/` for both `%` and
   `mod()`, each expectation taken from the table (which is the binary's
   output), and keep the existing `math.div(1, -7 % 7)` case, which
   exercises `real_mod`, not this branch.
3. Amend the "Not done" section of #12 if it is still open, or note the
   correction in its follow-up.

## 2. A rounding strategy that arrives through interpolation

`spec/values/calculation/round/three_arguments/strategy/interpolation`

### Current behavior

```scss
a {e: round(#{"up"}, 3px, 9px)}
```

dart-sass 1.103.1 recognizes the interpolated `up` as the strategy and
simplifies to `9px`. grass emits `round(up, 3px, 9px)` unsimplified.

The defect is narrow. Interpolation elsewhere in a calculation already
matches the reference: `calc(#{"1px"} + 2px)`, `abs(#{"-1px"})`,
`min(#{"1px"}, 2px)` and `round(up, #{3px}, 9px)` all print the same in
both implementations, because an interpolated *value* is opaque text that
neither can simplify. Only the strategy keyword is special: dart-sass
matches it by text after interpolation.

`SassCalculation::round` in `crates/compiler/src/value/calculation.rs`
(around line 897) already handles a `CalculationArg::Interpolation` first
argument by calling `parse_round_strategy`, so the keyword is not
reaching it in that form. Start by finding which parse path a `round()`
call with an interpolated argument takes -- the raw-string fallback for
interpolated special functions in `crates/compiler/src/parse/value.rs`
(around line 1446) is the likely one -- and make an interpolated first
argument reach the calculation evaluator as an `Interpolation` argument.

### Testing

The four matching cases above are regression guards: add them as `test!`
cases beside the fixed one, so the fix cannot widen simplification into
territory where dart-sass keeps the text opaque.

## 3. Line noise inside an interpolated `calc()` should not parse

`spec/values/calculation/calc/error/syntax/interpolation/line_noise`

### Current behavior

```scss
a {b: calc(!{@}#$%^&*#{c}_-[+]=)}
```

dart-sass 1.103.1 rejects this with `Error: expected ")".` at column 12.
grass accepts it and prints `calc(!{@}#$%^&*c_-[+]=)`. The spec's own
comment says why: "Interpolation no longer shifts the parser into a
special mode where it allows any interpolated declaration value."

The raw-string fallback named in section 2 is the cause here too: once a
`calc()` contains interpolation, its contents are read with
`parse_interpolated_declaration_value` and emitted as text, so nothing
checks that the text is a calculation. The whitespace trimming #4 added
there is a symptom of the same shortcut.

### Implementation instructions

Parse an interpolated calculation as a calculation whose arguments may
contain interpolation nodes, the way dart-sass does, and let the
calculation parser's own syntax errors apply. That removes the fallback
rather than patching it, and it is the same change section 2 needs.
Take the error message and position from the `.hrx` file; with
`--trim-errors` only the first line must match.

## Testing

- Ground truth: `spec/values/calculation/mod.hrx`,
  `spec/values/calculation/round/three_arguments.hrx` and
  `spec/values/calculation/calc/error/syntax.hrx` at the pinned revision.
- Scoped spec run: `spec/values/calculation` (see [README.md](README.md)).
- Verify every new expectation against the dart-sass 1.103.1 binary,
  including the operator table in section 1 -- it is the case where the
  existing comment in the code was trusted over the binary.

## Acceptance criteria

- `spec/values/calculation` passes under the roadmap's standard flags:
  0 failures.
- `%` and `mod()` agree with the table in section 1 for all eight sign
  combinations, pinned in `crates/lib/tests/`.
- The `frameworks` CI job still reports no color-value differences.
