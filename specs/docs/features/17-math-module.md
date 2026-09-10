# The math module

12 sass-spec failures in `spec/core_functions/math`, the deepest single
area no document claims. They are five independent causes, none of them
large, and four of the five are a line or two.

Measured 2026-09-10 against master (`31361d7`), the pinned sass-spec
revision `4a9eea66`, and dart-sass 1.103.1 run as `npx -y sass@1.103.1`:

```
cd sass-spec && npm run sass-spec -- --impl=dart-sass \
  --command '../target/release/accent-sass' --trim-errors \
  --ignore-warning-diffs --ignore-error-diffs spec/core_functions/math
```

| Section | Cause | Failures |
|---|---|---:|
| 1 | A fuzzy `is_zero()` stands in for an exact zero | 6 |
| 2 | `clamp()` compares in the wrong order | 3 |
| 3 | `clamp()` misses one unit mismatch | 1 |
| 4 | `$min-number` is the smallest normal, not the smallest double | 1 |
| 5 | `unit()` leaves a multi-unit denominator unbracketed | 1 |

## 1. A fuzzy `is_zero()` stands in for an exact zero

`asin/{zero_fuzzy,negative_zero_fuzzy}`, `atan/{zero_fuzzy,negative_zero_fuzzy}`,
`log/zero_fuzzy`, `log/base/zero_fuzzy`

`Number::is_zero` in `crates/compiler/src/value/number.rs:228` is
`fuzzy_equals(self.0, 0.0)`, which is true for anything within 1e-11 of
zero. That is right for Sass equality, and wrong as a guard in front of a
function that has a real value near zero:

| input | accent-sass | dart-sass 1.103.1 |
|---|---|---|
| `math.asin(0.000000000001)` | `0deg` | `0.0000000001deg` |
| `math.atan(0.000000000001)` | `0deg` | `0.0000000001deg` |
| `math.log(0.000000000001)` | `calc(-infinity)` | `-27.6310211159` |
| `math.log(2, 0.000000000001)` | `0` | `-0.025085833` |

`asin` at `builtin/modules/math.rs:340` and `atan` at line 365 each return
`0deg` when `number.is_zero()`; `log` at line 189 returns `-infinity` when
the argument is fuzzily zero and `0` when the base is. In every case the
value is 1e-12, twelve orders of magnitude from where the function is
degenerate, and the ordinary path gives the right answer.

The serializer is not implicated: it already rounds 5.7296e-11 to
`0.0000000001`, which is what dart-sass prints, so removing the guards is
enough.

Replace the fuzzy checks with exact `== 0.0` comparisons in these three
functions. Then fix `acos` at line 299, which has the same defect through
the neighbouring `is_one()` and no fixture that catches it:
`math.acos(0.999999999999)` prints `0deg` here and `0.0000810276deg` in
dart-sass 1.103.1. Check `atan2` too.

## 2. `clamp()` compares in the wrong order

`clamp/min_greater_than_max`, `clamp/preserves_units/{min,max}`

dart-sass resolves clamp as a fixed ladder: if `min >= max` return `min`;
if `number <= min` return `min`; if `number >= max` return `max`; otherwise
return `number`. The bounds win every tie, and an inverted range collapses
to `min`.

`clamp` in `crates/compiler/src/builtin/modules/math.rs:28` has no
`min >= max` case at all, and returns `number` on a tie:

```rust
match min.cmp(&number, span, BinaryOp::LessThan)? {
    Some(Ordering::Greater) => return Ok(min),
    Some(Ordering::Equal) => return Ok(number),
    ...
```

| input | accent-sass | dart-sass 1.103.1 |
|---|---|---|
| `math.clamp(1, 2, 0)` | `0` | `1` |
| `math.clamp(180deg, 0.5turn, 360deg)` | `0.5turn` | `180deg` |
| `math.clamp(180deg, 1turn, 360deg)` | `1turn` | `360deg` |

The unit rows are why the tie matters: `0.5turn` and `180deg` are equal, so
which operand you return decides the unit that gets printed. Returning the
bound rather than the number is what "preserves units" means in the fixture
names.

## 3. `clamp()` misses one unit mismatch

`clamp/error/some_unitless/min_and_number`

`math.clamp(0, 1, 2px)` must be an error and is not. The three checks in
`clamp` compare `$min` against `$number` and `$min` against `$max`, so a
unitless `$min` and `$number` with a `$max` that has a unit falls through
all of them. The other five `some_unitless` fixtures pass only because
`--ignore-error-diffs` hides the wording; dart-sass words all six as
`$max: 2px and $min: 0 have incompatible units (one has units and the other
doesn't).`, naming the two arguments that disagree. Fixing the wording as
well as the missing case is the honest version of this section, and costs
nothing extra.

## 4. `$min-number` is the smallest normal, not the smallest double

`variables/min_number`

`math.rs:500` defines `$min-number` as `f64::MIN_POSITIVE`. Rust's
`MIN_POSITIVE` is the smallest *normal* double, 2.2250738585072014e-308.
Dart's `double.minPositive`, which the reference uses, is the smallest
subnormal, 5e-324. The names match and the values do not.

```scss
a {b: math.$min-number * 1e300 * 1e39}
```

prints `22250738585072010000000000000000` here and `4940656458412465` in
dart-sass. Use `f64::from_bits(1)`, and say in a comment why
`f64::MIN_POSITIVE` is wrong, because it is the obvious thing to reach for
again later.

## 5. `unit()` leaves a multi-unit denominator unbracketed

`unit/numerator_and_denominator/multiple`

`Display for Unit` in `crates/compiler/src/unit/mod.rs:342` brackets a
denominator when the numerator is empty and there is more than one unit
below the line (`(px*em*rad)^-1`), but the numerator-and-denominator arm
never brackets:

```rust
} else {
    write!(f, "{}/{}", numer_rendered, denom_rendered)
}
```

`math.unit(1px * 1em / 1rad / 1s)` prints `"px*em/rad*s"` here and
`"px*em/(rad*s)"` in dart-sass. Bracket `denom_rendered` when
`denom.len() > 1`, mirroring the arm two lines above.

This `Display` is also what error messages use, so a scoped run of
`spec/values` is worth doing after the change, not just `spec/core_functions`.

## Testing

- Ground truth: `spec/core_functions/math/{asin,atan,log,clamp,unit,variables}.hrx`
  at the pinned revision.
- Scoped spec runs: `spec/core_functions/math`, then `spec/values` for
  section 5 and the whole suite for section 1, since `is_zero` is used
  widely and only these three call sites should change.
- Add regression tests to `crates/lib/tests/` with `test!` and `error!`:
  one per row of the two tables above, plus `math.clamp(0, 1, 2px)` and
  `math.acos` near zero.
- Verify every new expectation against dart-sass 1.103.1 with
  `npx -y sass@1.103.1`, never bare `npx sass`.

## Acceptance criteria

- `spec/core_functions/math` passes: 0 failures, down from 12.
- The whole-suite "Expected test to fail but it did not" count drops from
  28 to 27 through section 3, and nothing else moves.
- No other area regresses. Sections 1 and 5 touch shared code, so the
  measurement that matters is the whole-suite count, not the scoped one.
