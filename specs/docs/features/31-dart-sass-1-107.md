# Moving the reference to dart-sass 1.107.0

This item moves the reference implementation from dart-sass 1.105.2 to
1.107.0 and the sass-spec pin from `530e761ec` to `1e2e5ddaf`. It spans three
releases: 1.106.0 and 1.106.1 on 2026-10-21, and 1.107.0 on 2026-10-28. Like
[item 30](30-dart-sass-1-105.md) it is listed under "Reference moves" in the
[README](README.md). Unlike item 30, the move and the work it calls for land
in one pull request.

## Baseline before the move

Measured 2026-10-30 on master `a3a6dc6c`, macOS, release build, with CI's
flags (`--trim-errors --ignore-warning-diffs --ignore-error-diffs`):

| sass-spec revision | Runs | Failures |
|---|---|---|
| `530e761ec`, the old pin | 14,355 | 100 |
| `1e2e5ddaf`, sass-spec main | 14,400 | 125 |

The move adds 45 runs and 25 failures, all from 1.106.0, and fixes none.
`1e2e5ddaf` is the commit that adds 1.107.0's tests, and nothing has landed on
sass-spec main after it.

Against the 1.107.0 binary, Bulma, Pico, USWDS and the 106 Foundation function
results compiled byte-identically. Foundation differed on one line, its
`@media screen and (min-width: 0\0)` IE hack.

## What the three releases change

From `gh api repos/sass/dart-sass/compare/<old>...<new>` for each pair of
tags:

1. **1.106.0** is the escape release. The language change is `91fd60be1`,
   "Only normalize escapes at the expression level" (sass/dart-sass#2880),
   with sass-spec `58295762b`. It also merges `2ea4fc07b`, "Fix more issues
   found by the fuzzer" (sass/dart-sass#2883):
   - `a05866cae`: interpolation that contains `// */` no longer crashes.
   - `34c8e6b1f`: an indented first statement after the first line no
     longer crashes the indented parser.
   - `09e4024c4`: a rest parameter that repeats an earlier name is an error.
   - `56608242a`: `FileSpan.trimLeft()` and `trimRight()` are more accurate.
     This affects spans only.

   The sass-spec tests for the first two already passed here.
2. **1.106.1** fixes two dart-sass crashes. An attribute selector with an
   empty value crashed its new serializer (`e79f8a7d8`). This compiler never
   crashed on it, and sass-spec `fd1b9497e` passes. A number with an
   empty-string unit (`b10205ed8`) is reachable only through dart-sass's own
   APIs.
3. **1.107.0** lets `--fatal-deprecation` exclude deprecations, and adds the
   same to the JavaScript, Dart and embedded APIs (`455cbc050`). This
   compiler's command line has no `--fatal-deprecation`. Its sass-spec tests,
   in `1e2e5ddaf`, live in `js-api-spec`, which the `sass-spec` job does not
   run.

### The escape change

Before 1.106.0, the parser normalized an escape into its written form as it
read an identifier. Now an identifier the parser reads holds its contents
with escapes resolved: `.\61` and `.a` name the same class. Writing CSS
escapes them again, through dart-sass's `toCssIdentifier`. The text of a
statement keeps escapes exactly as written: at-rule names and values,
property names, custom property values, keyframe names and selectors before
they are parsed. Only SassScript expressions normalize them.

What a stylesheet sees:

- In an expression, an escape of U+0000, of a surrogate or of a code point
  beyond U+10FFFF is U+FFFD. Before, it was an error or `\0 `. The exact
  identifier `\0` stays `\0 `, because CSS hacks use it. So Foundation's
  `0\0` is now `0` followed by U+FFFD.
- `\10FFFF` is the highest valid code point, no longer U+FFFD.
- A keyword or function name matches with its escapes resolved:
  `!\69 mportant`, `true \61 nd false`, `\75 rl(` in `@import`,
  `:nth-child(\6f dd)` and `@\6d edia` mean what they spell.
- `@f\6Fo`, `--\61: b`, `\63 olor: red` and `@keyframes \61` keep their
  escapes in the output. Before, each was normalized.
- A unit that would read back as an exponent is written with an escaped `e`:
  `1e1e1` prints as `10\65 1`.
- An escaped `e` does not start an exponent in a keyframe selector.
- An underscore written as an escape normalizes like `_` in a variable,
  function or mixin name. This compiler already did.
- An attribute value that is not an identifier written as is gets quotes:
  `[a=b\ c]` prints `[a="b c"]`.

## What this item changed

- **The reference move.** The `bootstrap` and `frameworks` jobs install
  1.107.0. The `sass-spec` submodule is pinned to `1e2e5ddaf`. `README.md`,
  `CLAUDE.md`, the documentation site and this directory's README name
  1.107.0.
- **The escape model**, ported from dart-sass's `parser.dart`,
  `stylesheet.dart` and `util/string.dart`:
  - `parse_identifier` resolves escapes, and two new helpers let keyword
    checks see through an escape: `peek_char_or_escape` and
    `read_char_or_escape`.
  - `parse_interpolated_identifier` and `parse_interpolated_declaration_value`
    take an `expression` flag, set at each call site as dart-sass sets it.
    Only the hash expression, `identifierLike`, calculation identifiers, the
    CSS `if()` condition and special-function arguments are expressions.
  - `utils/strings.rs` holds three conversions:
    - `to_css_identifier`, which is `toCssIdentifier`. Selector names, the
      media type and units are written through it.
    - `resolve_css_escapes`, which is `resolveCssEscapes`. Keyword checks use
      it through `Interpolation::as_plain_identifier`, as dart-sass uses
      `asPlainIdentifier`.
    - `character_to_escape`, which is `characterToEscape`.
  - The two checks dart-sass left on the raw text stay raw. Those are the
    media query keywords and `@supports not`. That is why `@media screen \61
    nd (color)` is an error in 1.107.0 and here.
- **The rest.** The serializer escapes exponent-like units. The keyframe
  selector parser takes only a literal `e`. A repeated parameter, rest or
  not, is "Duplicate parameter.", as dart-sass words it.
- **Two fixes the port brought along.** `[|b=c]`, an attribute in no
  namespace, parses. That closed `css/selector/attribute/empty_namespace`,
  which failed before the move. And a named color keeps the text it was
  written in: `blu\65` stays `blu\65`. Two tests in `str-escape.rs` had
  expected `blue`. 1.105.2 printed `blu\65` as well, so those expectations
  were wrong before this item.

The 18 test expectations that changed were each checked against the 1.107.0
release binary. The 71 tests in `crates/lib/tests/escapes.rs` take their
expectations from its output.

## Measurement

Measured 2026-10-30 on macOS, as for the baseline:

| Revision | Runs | Failures |
|---|---|---|
| before, `a3a6dc6c` at `1e2e5ddaf` | 14,400 | 125 |
| after | 14,400 | 99 |

26 fixtures pass that did not: the 25 new ones and `empty_namespace`. None
fails that passed. The failures split into 77 different output, 12 rejecting
valid input and 10 accepting invalid input. Before, the split was 77, 13 and
10.

A probe corpus of 479 escape cases was compiled with both engines. It covers
expressions, units, strings, selectors, at-rules, declarations, plain CSS and
the indented syntax. Before the port, 133 cases differed from 1.107.0, and
1.105.2 itself differed from 1.107.0 on 121. Afterwards, 2 differ, and both
differed from 1.105.2 as well. The corpus was a scratch script; the 71 tests
in `escapes.rs` are drawn from it.

`.github/scripts/frameworks.sh` against the 1.107.0 binary: Bulma, Pico,
Foundation, USWDS and the Foundation function results compile
byte-identically, and so does Bootstrap 5.0.2. Against master, only
Foundation's IE hack line changes. Accent's four theme entry points compile
byte-identically on master and on this branch. Their escapes are all inside
quoted `content` strings, which this release leaves alone.

Compile times, as the median of seven runs against master's release build,
are within noise: Bulma -0.4%, Foundation -1.2%, USWDS +0.7%. The new
`Interpolation::as_plain_identifier` and `to_css_identifier` return borrowed
text for an identifier with no escapes, so the common case allocates nothing
new.

## Deviations that remain

- **A dart-sass crash.** In an expression, a backslash at the very end of
  the input escapes U+FFFD. dart-sass 1.107.0's check for the `\0` hack then
  reads two characters past it and fails with a `RangeError`. `a { color:
  [\` and `@if \` both crash it. This compiler reports the error dart-sass
  would reach without the crash. Comments next to the two tests in
  `error.rs` and `if.rs` say so.

The probe corpus also found four gaps that predate the move. Each differs
from 1.105.2 as well:

- `"\7f"` writes U+007F as is. dart-sass escapes it as `\7f`.
- A quoted string in a custom property, or in any other unevaluated
  declaration value, is written from its value. dart-sass keeps the token as
  written, through `interpolatedStringToken`. So `--a: "\201C"` prints
  `"“"` here and `"\201C"` in dart-sass.
- `calc(\61 + 1px)` compiles here. dart-sass rejects it because the
  escape's trailing space belongs to the identifier, which leaves the `+`
  without whitespace before it.
- `:nth-child(2n + \31)`, an escaped digit after the `+`, is an error in
  both. Through `selector.parse`, this compiler reports `expected
  selector.` where dart-sass reports `Expected a number.`. Message wording
  is outside the parity rule.

## Not done

- No release. When one carries this item, it is a minor version: output
  changes for escaped identifiers, which dart-sass itself flags as
  "potentially breaking".
- The four older gaps above.
- The crash is not reported upstream; there is no upstream contribution
  obligation.
