# CSS math functions and constants in calculations

Unlocks about 600 sass-spec tests: all 436 failures under
`spec/values/calculation` plus roughly 160 of the 176 failures under
`spec/core_functions/color`, which are otherwise-working color functions
that receive `calc(infinity)` or `calc(NaN)` as a channel and die in the
parser. This is the largest single item on the roadmap.

## Current behavior

The calculation engine recognizes only four functions. `CalculationName`
in `crates/compiler/src/value/calculation.rs:41` has the variants `Calc`,
`Min`, `Max` and `Clamp`, and the parser dispatches on exactly those names
in `crates/compiler/src/parse/value.rs:1748-1779`.

Everything else falls through to plain function parsing, so
`round(up, 1.5px)`, `mod(10, 3)` or `hypot(3px, 4px)` inside a value are
not treated as calculations, and the calc-context constants are rejected
outright:

```
Error: Expected "(" or ".".
  ,
1 | a {b: rgb(calc(infinity), 0, 0, 0.5)}
  |                        ^
```

## Reference behavior

dart-sass 1.103.1 treats the full CSS math function set as calculations
and simplifies them at compile time when the arguments allow it:

- Stepped value functions: `round()` (with the `nearest`, `up`, `down`
  and `to-zero` strategies), `mod()`, `rem()`.
- Trigonometric functions: `sin()`, `cos()`, `tan()`, `asin()`, `acos()`,
  `atan()`, `atan2()`.
- Exponential functions: `pow()`, `sqrt()`, `hypot()`, `log()`, `exp()`.
- Sign-related functions: `abs()`, `sign()`.
- Calc constants, case-insensitive, valid only inside a calculation
  context: `infinity`, `-infinity`, `NaN`, `e`, `pi`.

Degenerate results survive as values: `calc(infinity)` is a number whose
value is infinite, it can flow into color channel arguments, and the
color functions clamp or propagate it the same way dart-sass does.

## Implementation instructions

1. Extend `CalculationName` in `crates/compiler/src/value/calculation.rs`
   with one variant per function above, including its `Display` and
   serializer arm (`write_calculation_name` in
   `crates/compiler/src/serializer.rs:305`).
2. Extend the name dispatch in `crates/compiler/src/parse/value.rs`
   (around line 1748) so those names parse as calculations. Note the
   argument shapes differ: `round()` takes an optional leading strategy
   keyword, `atan2()`, `pow()`, `mod()` and `rem()` take exactly two
   arguments, `hypot()` is variadic, the rest take one.
3. Add the calc constants to the calculation value parser. They are
   identifiers, matched case-insensitively, and only valid inside a
   calculation; outside one, `infinity` stays an ordinary identifier.
   The spec has explicit case tests (`InFiNiTy`), so preserve the source
   casing when a calculation is emitted unsimplified.
4. Implement simplification in `SassCalculation`
   (`crates/compiler/src/value/calculation.rs`) mirroring the existing
   `min`/`max`/`clamp` logic: when every argument is a compatible number,
   compute the result; otherwise emit the calculation as text. Pay
   attention to unit rules — the trigonometric functions accept an angle
   or a unitless number, `sign()` and `abs()` preserve the argument's
   unit, `atan2()` and the stepped functions require compatible units,
   and `log()`/`exp()` require unitless arguments. Derive the exact
   error messages and edge cases from the spec tests, not from memory.
5. Make degenerate numbers (infinities and NaN) first-class in `Number`
   (`crates/compiler/src/value/number.rs`) wherever they are not already:
   arithmetic, comparison, clamping in color channels, and serialization
   (an infinite number serializes as `calc(infinity)` in a CSS context).
6. Re-run the color suite afterwards; the ~160 `calc(infinity)` color
   failures should disappear without touching the color code. Whatever
   remains in `spec/core_functions/color` is a genuine color bug —
   investigate separately.

## Testing

- Ground truth: `spec/values/calculation/` (per-function `.hrx` files;
  `round/` and `calc/` are directories) and `spec/core_functions/color/`.
- Scoped spec run: append `spec/values/calculation` to the runner
  invocation in [README.md](README.md).
- Add `test!`/`error!` cases to `crates/lib/tests/` covering each new
  function's simplified and unsimplified forms, the constants, and at
  least one degenerate-channel color case. Verify each expectation
  against the dart-sass 1.103.1 binary first.

## Acceptance criteria

- `spec/values/calculation` passes apart from at most a handful of
  error-span differences.
- The `calc(infinity)`-driven failures under `spec/core_functions/color`
  are gone.
- The `frameworks` CI job still reports no color-value differences.
