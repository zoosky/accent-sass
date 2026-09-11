# Moving the reference to dart-sass 1.104.0

This item moves the reference implementation from dart-sass 1.103.1 to
1.104.0. It records the baseline taken before the move, what the release
changes, and what following it took. It is listed under "Reference moves" in
the [README](README.md) rather than in the ranking, which counts failures
against 1.103.1. It does close one failure the ranking counts, the last in
[item 07](07-calculation-long-tail.md), described below.

## Baseline before the move

Measured 2026-09-11 on master `e8868a5e` (#82 merged), CI run 34596558170,
against the native dart-sass 1.103.1 linux-x64 binary:

| Check | Result |
|---|---|
| `frameworks`: Bulma, Pico, Foundation, USWDS | 5, 0, 0 and 903 differing lines; none colour-bearing |
| `frameworks`: Foundation function probe | 0 differing across 106 results |
| `sass-spec`, pinned revision `4a9eea66f` | 14,218 runs, 14,033 passing, 177 failures, 8 todo |

The same spec run on macOS reports 175, two fewer, which is the offset the
README's measurement section describes. Compare CI with CI and macOS with
macOS.

To reproduce the framework counts locally, use the native dart-sass release
binary for your platform. It reproduces CI's counts exactly; `npx sass`, the
JavaScript build, reported 9, 0, 44 and 917 on the same commit.

## What 1.104.0 changes

The [release notes](https://github.com/sass/dart-sass/releases/tag/1.104.0)
list two changes, and the only library commit between the two tags is
`0ea3ef0c8`, "Implement degenerate colors" (sass/dart-sass#2840):

1. **Negative zero is serialized as `-0`** instead of `0`, so it keeps its
   sign when used in a CSS calculation. Only exact negative zero is affected;
   a value that merely rounds to zero still prints `0`.
2. **Colours convert degenerate channels to `0`**, as CSS Color 4 specifies:
   `NaN` and negative zero in any channel, and positive or negative infinity
   in a polar hue. dart-sass does this in `SassColor.forSpaceInternal` only.

The same commit also rewrote `signIncludingZero` to use a cross-platform
identity check. That changes nothing for the native binary, which is the
reference: 1.103.1 and 1.104.0 give the same result for every sign of zero
against an infinite divisor. It fixed the JavaScript build, which `npx sass`
runs. `npx sass@1.103.1` counted positive zero as negative, so `0 % infinity`
was `NaN` and `0 % -infinity` was `0` there. Review of this item's pull
request ran through `npx` and first read that as a 1.104.0 change.

## Measured before any change

- `frameworks.sh` against the native 1.104.0 binary gave the same counts as
  against 1.103.1: nothing either change touches appears in the corpus.
- sass-spec upstream `b39c32768`, the first revision with 1.104.0's
  expectations, against the unchanged binary: 175 failures became 223 locally,
  48 new and none fixed. 27 were degenerate colours; 21 were negative zero,
  from `math.sin`, `asin`, `atan`, `atan2`, `tan`, `sqrt`, `pow` and `log`,
  `values/numbers/negative_zero` and `non_conformant/misc/negative_numbers`.

## What following it took

The two release-note changes are the first three items below. The rest
surfaced when a probe of about 125 declarations was
compared against the 1.104.0 binary, from the spec, or in review: each is a
place where this compiler
produced a sign or a `NaN` differently from Dart, which printing `-0` or
normalizing a channel made visible.

- The serializer's `write_float` and `Number::to_string` print exact
  negative zero as `-0`.
- `Color::for_space` ports `_normalizeLinear` and the new `_normalizeHue`
  rule. A zero hue is now `0` even when a negative saturation or chroma would
  have rotated it by 180 degrees, as in dart-sass.
- `Color::rgb_internal` is the unnormalizing constructor dart-sass calls
  `SassColor.rgbInternal`. Legacy rgb built by `rgb()`, `color.change()`, the
  legacy `mix()` and `invert()`, and hex literals goes through it, so
  `color.change(#123456, $red: -0)` keeps a red of `-0`. Conversion to another
  space normalizes.
- `math.round`, `math.ceil`, `math.floor`, `fuzzy_round` and the calculation
  `round()` never return `-0`: Dart computes each as an integer, which has no
  negative zero.
- `clamp_like_css` orders `-0` below `0`, as Dart's `clamp` does through
  `compareTo`, so `rgb(-0, 10, 20)` has a red of `0` and a `-0` alpha clamps
  to `0`.
- The rgb-to-hsl and rgb-to-hwb conversion propagates `NaN` through `max` and
  `min`, as Dart's `math.max` and `math.min` do. Rust's skip it, which made a
  `NaN` red print `hsl(0, 24.6376811594%, 27.0588235294%)` where dart-sass
  prints `hsl(0, 0%, 0%)`.
- Sass's `%`, and the calculation `mod()` that shares it, count only negative
  zero as negative when the divisor is infinite. This compiler counted every
  zero as negative, which matched the JavaScript build of 1.103.1 but not the
  native binary. That was [item 07](07-calculation-long-tail.md)'s section 1,
  and its fixture, `values/calculation/mod/nan/zero_and_negative_infinity`,
  passes now. Review found it.
- The legacy channel accessors `red()`, `green()` and `blue()` return an
  integer, as Dart's `SassColor.red` does through `round()`, so a `-0`
  channel that `color.change()` keeps reads back as `0`. Review found this
  too: until negative zero printed as `-0`, the difference was invisible.
- `math.asin` and `math.atan` keep the sign of a zero argument.
- `math.log` with a base divides the two natural logarithms, as dart-sass
  does. Its special case for a base of zero returned `0`, which was wrong for
  `math.log(0, 0)` and `math.log(-2, 0)` under 1.103.1 as well; both are
  `NaN`.

Nine existing tests were updated, each with a comment naming what it now
follows. Eight expected 1.103.1's output: two in `color_css4.rs`, two in
`color_hsl.rs`, one in `color_interpolation.rs`, one in `number.rs` and two
in `math-module.rs`. The ninth, `zero_mod_infinity_is_nan` in `modulo.rs`,
expected the JavaScript build's answer for `0 % infinity`; both native
binaries give `0`, and it is now `zero_mod_infinity_keeps_dividend`. New regression tests are in `negative-zero.rs` and
`degenerate-colors.rs`. Every expectation was checked against the 1.104.0
binary.

## After

- sass-spec pinned to `b39c32768`: 14,266 runs, 14,085 passing, 173 failures
  locally. Against the failure list at the old pin, no fixture newly fails.
  All 48 of 1.104.0's new failures pass, and so do two that failed at the old
  pin too:
  `core_functions/color/hwb/four_args/blackness/degenerate/negative_infinity`
  and `values/calculation/mod/nan/zero_and_negative_infinity`. Expect CI to
  read two higher.
- `frameworks.sh` against the native 1.104.0 binary: 5, 0, 0 and 903
  differing lines, none colour-bearing, and 0 across the 106 Foundation
  function results. Unchanged from the baseline.
- The probe used to find the items above matches 1.104.0 on every
  declaration, in expanded and compressed output.

## What moved with it

- The `bootstrap` and `frameworks` CI jobs install the 1.104.0 binary.
- The `sass-spec` submodule is pinned to `b39c32768`.
- `README.md`, `CLAUDE.md`, the Foundation probe's header and
  `CHANGELOG.md` name 1.104.0.

Code comments that say a routine was ported from, or verified against,
dart-sass 1.103.1 are left as they are. They record what was compared, and
none of those routines changed in 1.104.0.
