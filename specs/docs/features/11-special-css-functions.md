# Special CSS functions

Unlocks the 22 sass-spec tests under `spec/css/functions`, the deepest area
no document claimed, the 6 under `spec/css/percent`, and 35 under
`spec/core_functions/color`. **Sections 1, 2, 4 and 5 have landed**, taking
those 41 and 18 of the 22; section 3 is the 4 that remain.

Sections 1 and 2 reached much further than this document predicted: 34
fixtures, not the 16 counted here, because the same two defects live in
`almost_any_value` and so ran through `spec/css/supports`,
`spec/css/unknown_directive`, `spec/css/moz_document` and three libsass
issues as well.
Measured 2026-09-06 against master (`1c3608e`), the pinned sass-spec revision
`4a9eea66`, and dart-sass 1.103.1 run as `npx -y sass@1.103.1`:

```
120 runs, 98 passing, 22 failures, 0 todo, 0 ignored, 0 errors
```

A *special function* is one whose arguments are not Sass expressions:
`element()`, `expression()`, `progid:...()`, `url()`, `type()`, and any
vendor-prefixed `calc()`. Their contents are read as text and printed back
nearly verbatim. Unprefixed `calc()` is not among them -- it parses as a real
calculation, which is why `CALC(0)` prints `0` while `-a-calc(0)` prints
itself. Sections 1 to 3 are that text going wrong: text that should have
been dropped, text that should have been preserved, and a function that never
took the text path at all. Sections 4 and 5 are the neighbouring question of
which unevaluated strings a value position accepts, which is where the
colour functions fail.

Sections 1 to 3 are independent, except that section 2 has to precede
section 3 for two of section 3's tests to pass. Sections 4 and 5 are worth
63 tests, but only together: the colour fixtures need the value to parse
*and* the parsed value to be accepted as a channel, so either section alone
leaves all 35 failing.

| Section | Defect | Tests in the named area | Colour fixtures |
|---|---|---:|---:|
| 1 | A silent comment is copied into the output -- **landed** | 8 | -- |
| 2 | A quoted string is re-quoted -- **landed** | 8 | -- |
| 3 | `type()` is not a special function | 4 | -- |
| 4 | `attr()` and `if()` are not special variable strings -- **landed**, #39 | 2 | 35 |
| 5 | A bare `%` is not a value -- **landed**, #38 | 6 | 35, with 4 |

Sections 1 and 2 reached further than the table says, which the landing
confirmed: 34 fixtures across six areas rather than the 16 counted here.

## 1. A silent comment inside a special function is copied through -- landed

`spec/css/functions/special/comment/{calc,element,expression,progid}/{after_open_paren,before_close_paren}/silent`

### Current behavior

`parse_interpolated_declaration_value` in
`crates/compiler/src/parse/stylesheet.rs` (the `'/'` arm, around line 2219)
recognizes `/*` and copies the loud comment's raw text, which is correct.
Anything else beginning with `/` falls to the `else` branch, which writes the
slash and moves on -- so `//` and the comment text after it land in the
output:

```scss
a {
  b: -a-calc(//
    c);
}
```

```css
a {
  b: -a-calc(// c);
}
```

### Reference behavior

dart-sass drops the comment and keeps the newline that ends it, which the
existing whitespace handling already collapses to a single space:

```css
a {
  b: -a-calc( c);
}
```

The second shape in each pair puts the comment before the closing paren.
`-a-calc(c //\n    )` becomes `-a-calc(c  )`: one space from before the
comment, one from the newline. Three variants were checked against the
binary -- text after `//`, no indentation on the next line, and two spaces
before the comment -- and all three produce exactly one space per newline.
A newline with no comment at all already agrees; `-a-calc(x\n    y)` prints
`-a-calc(x y)` in both engines today.

### Implementation instructions

In the `'/'` arm, treat `//` the way the surrounding parser treats a silent
comment: consume to the end of the line and write nothing. Do not consume
the newline itself -- it is what produces the space, and consuming it makes
`before_close_paren` print `-a-calc(c )` with one space instead of two.

Check the indented syntax before you finish. In `.sass` a newline can end
the statement, which the `consume_newlines` parameter governs; the eight
failing tests are all `.scss`, so confirm no `.sass` test that passes today
starts failing.

## 2. A quoted string inside a special function is re-quoted -- landed

`spec/css/functions/special/prefixed/{lowercase,uppercase}/{calc,element,expression,progid}/punctuation`

### Current behavior

The `'"' | '\''` arm of the same function (line 2211) parses the string and
calls `as_interpolation(false)`. That picks a quote character with
`best_quote` rather than keeping the source's, so a single-quoted string
comes back double-quoted:

| input | accent-sass | dart-sass 1.103.1 |
|---|---|---|
| `-a-calc('x')` | `-a-calc("x")` | `-a-calc('x')` |
| `-a-calc('')` | `-a-calc("")` | `-a-calc('')` |
| `-a-calc('x"y')` | `-a-calc('x"y')` | `-a-calc('x"y')` |
| `unknown('x')` | `unknown("x")` | `unknown("x")` |

The last row is the control: outside a special function both engines
normalize, so this is not a general quoting difference. The eight failing
fixtures are one line of punctuation that happens to contain `""''`, which
accent-sass prints as `""""`.

### Reference behavior

Inside a special function, dart-sass preserves the quote character the
source used. Interpolation inside the string is still evaluated:
`-a-calc('a#{1+1}b')` prints `-a-calc('a2b')`, single quotes kept.

### Implementation instructions

Emit the string with its original quote character. `as_interpolation` takes
an `is_static` flag, but that flag only reaches `quote_inner_text`;
`best_quote` runs either way, so passing `true` alone does not fix this.
Either thread the source's quote character through, or capture the string's
raw text in this arm the way the loud-comment branch does.

Both call sites that pass `false` are in `stylesheet.rs` (lines 2215 and
2939). `crates/compiler/src/parse/css.rs:158` already passes `true` and is a
different path.

Fix the second one, at line 2939, in the same change. It is `almost_any_value`,
which parses unknown at-rule values, and it carries both this defect and
section 1's:

| input | accent-sass | dart-sass 1.103.1 |
|---|---|---|
| `@asdf 'foo bar baz';` | `@asdf "foo bar baz";` | `@asdf 'foo bar baz';` |
| `@a b //` | `@a b //;` | `@a b;` |

That is `spec/css/unknown_directive/value_interpolation` for the quoting and
`comment/{children,no_children}/after_value/silent` for the comment -- three
of that area's seven failures, from the same two fixes. The area is
unclaimed, so nothing else covers them.

### What sections 1 and 2 needed beyond the instructions above

Four things the instructions did not anticipate, each established against
dart-sass 1.103.1:

- **A custom property keeps its `//`.** Its value is raw CSS, where `//` is
  two slashes, so `--x: // ;` prints as written. dart-sass carries a
  `silentComments` parameter for exactly this;
  `parse_interpolated_declaration_value` now does too, and the custom-property
  call sites pass `false`. Without it the change broke `--btn-font-family: //`.
- **The quoted string has to be re-emitted as static text.** Passing
  `is_static: true` keeps an escaped `\#{` escaped, because the text is
  re-parsed. With `false` it came back as a live interpolation.
- **`url()` and `url-prefix()` hold a raw URL**, so the `//` in `http://` is
  not a comment. Both parsers handled only `url`, which turned
  `@-moz-document url-prefix(http://x)` into a truncated value once comments
  started being dropped, and `element(url-prefix(http://x))` into an error.
  dart-sass matches both names case-sensitively, so `URL(` is an ordinary
  function call whose `//` *is* a comment. The two branches are easy to fix
  one at a time and then diverge: spec fixtures cover only the at-rule side.
- **The calc whitespace trim is for the unprefixed name only.** `-a-calc( x )`
  keeps its spaces in dart-sass, while an interpolated `calc( #{x} )` is
  trimmed. The trim was keyed on the unvendored name, which ate the space the
  two silent-comment fixtures expect.

### A divergence these sections did not close

`@a domain(http://x);` prints `@a domain(http:;` here and errors with
`expected ")"` in dart-sass, which tracks the unclosed paren. The shape is not
new -- before the comment was dropped this printed `@a domain(http://x);`,
which dart-sass also rejects -- so it stays in the "accepts invalid input"
column either way. No fixture in the pinned revision covers it, and closing it
means balancing parens in `almost_any_value`.

## 3. `type()` is not treated as a special function

`spec/css/functions/special/unprefixed/lowercase/type/punctuation`,
`spec/css/functions/special/unprefixed/uppercase/type/{punctuation,number,interpolation}`

### Current behavior

`try_parse_special_function` in `crates/compiler/src/parse/value.rs` (line
1461) matches `calc`, `element`, `expression`, `progid` and `url`. `type` is
absent, so `type(...)` parses as an ordinary unknown function call. Two
consequences:

- A body that is not a Sass expression is a parse error. `type(@#$%^&*(...))`
  fails with `expected ")"`, where dart-sass prints it back verbatim.
- The name keeps its source case. `TYPE(0)` stays `TYPE(0)`; dart-sass
  prints `type(0)`.

`type(0)` and `type(#{0})` pass today by coincidence: they are valid Sass
expressions that happen to serialize identically.

### Reference behavior

dart-sass treats bare `type` as special and prints the unvendored, lowercased
name, as it does for the others:

| input | accent-sass | dart-sass 1.103.1 |
|---|---|---|
| `TYPE(0)` | `TYPE(0)` | `type(0)` |
| `TYPE(#{0})` | `TYPE(0)` | `type(0)` |
| `TYPE('x')` | `TYPE("x")` | `type('x')` |
| `-a-type('x')` | `-a-type("x")` | `-a-type("x")` |

The last row matters: a vendor prefix does **not** make `type` special. The
existing arm matches on `unvendor(name)`, so adding `type` there would make
`-a-type('x')` special too and print `-a-type('x')`, which is wrong. Match
the unnormalized name instead.

### Implementation instructions

Add a `type` case that matches the name before unvendoring, then reuse the
`calc | element | expression` body -- scan `(`, read the contents with
`parse_interpolated_declaration_value`, expect `)`. The lowercasing comes
free: `try_parse_special_function` is already called with the lowercased
name (`value.rs:1278`).

The two `type/punctuation` tests also need section 2; the body they pass is
the same `""''` line. Landing this section alone moves them from an error to
a wrong-output failure.

## 4. `attr()` and `if()` are not special variable strings -- landed in #39

`spec/css/functions/special_variable/{attr,if}`, and 35 fixtures under
`spec/core_functions/color`

Landed once section 5 cleared the parse error in front of it. Adding the two
prefixes to both predicates took the suite from 369 failures to 332: all 35
colour fixtures and both tests here, with nothing newly failing and the
"accepts invalid input" count unmoved at 29.

The 2 tests here are `rgb(attr(c))` and `rgb(if(css(): c))`. The section
below on what it does not unlock is kept because it records why the 35 sat
behind section 5, which the diff does not show.

### Current behavior

Two predicates decide whether an unevaluated string may stand where a number
is expected:

- `is_special_function` in `crates/compiler/src/utils/mod.rs:36` accepts
  `calc(`, `var(`, `env(`, `min(`, `max(` and `clamp(`.
- `Value::is_var` in `crates/compiler/src/value/mod.rs:336` accepts `var(`
  alone, and governs the forms where one argument stands for several
  channels.

Neither knows `attr(` or `if(`, so a colour function rejects them:

```
a {b: rgb(attr(c))}
Error: $channels: Expected red channel to be a number, was attr(c).
```

### Reference behavior

dart-sass accepts both, in every channel position and in the one-argument
form, and prints the call back unevaluated:

| input | dart-sass 1.103.1 |
|---|---|
| `rgb(attr(c))` | `rgb(attr(c))` |
| `rgb(if(css(): c))` | `rgb(if(css(): c))` |
| `hsl(attr(c, %), 2%, 3%, 0.4)` | `hsl(attr(c, %), 2%, 3%, 0.4)` |
| `rgb(attr(c) 2 3)` | `rgb(attr(c), 2, 3)` |
| `lab(attr(c) 2 3)` | `lab(attr(c) 2 3)` |

The separator differences in the last two rows are the existing behavior for
`var()`, which accent-sass already reproduces: `rgb(var(--c) 2 3)` prints
`rgb(var(--c), 2, 3)` today. So the plumbing exists and only the two
predicates are missing entries.

The spec states the general rule in `spec/css/functions/special_variable.hrx`:
an implementation is expected to support every special variable string
everywhere `var()` is accepted, and the file tests `rgb()` as the
representative case.

### What this section does not unlock

**Not the 35 colour fixtures, on its own.** Every one of them passes a unit
to `attr()` -- the string `attr(c, %)` appears 70 times across the `rgb`,
`hsl` and `lab` files and no bare `attr(c)` appears at all -- and that fails
before any predicate is consulted:

```
a {b: rgb(attr(c, %), 2, 3)}
Error: expected ")".
```

The parse error is section 5: a bare `%` is not a value in this compiler.
Adding the two prefixes and rerunning `spec/core_functions/color` leaves all
57 failures in place. The 35 need section 5 first, and then this section, so
that the parsed `attr(c, %)` is accepted as a channel.

### What the change was

`attr(` and `if(` added to both predicates. `is_var` carries a minimum-length
guard derived from `"var(--_)"`, specific to custom properties, so the new
prefixes return before it rather than through it.

Two things to check rather than assume:

- **`if(` is also a Sass function.** Confirmed: `a {b: if(true, 1, 2)}` still
  prints `1`, and `rgb(if(true, 1, 2))` fails with `$channels: The rgb color
  space has 3 channels but 1 has 1` -- the same error dart-sass 1.103.1
  gives, because the Sass `if()` evaluates to `1` long before a value is
  inspected. Both are pinned in `crates/lib/tests/css-if.rs`.
- **Strictness.** `is_special_function` is consulted from 17 places, most of
  them the colour builtins, so widening it makes the compiler more lenient
  everywhere at once. Checked on the whole suite: "accepts invalid input"
  stayed at 29, so the 37 tests were not bought with leniency.

### A case difference this section did not close

dart-sass matches these names case-insensitively and this compiler does not.
`a {b: rgb(ATTR(c))}` prints `rgb(ATTR(c))` in dart-sass 1.103.1 and errors
here with `$channels: Expected red channel to be a number, was ATTR(c)`.

The gap is older than item 11: `rgb(VAR(--x), 1, 2)` diverges the same way and
did before any of this, while `CALC(` escapes it only because the calculation
parser handles that name itself. #13's CSS `if()` parser has the same shape --
`rgb(IF(css(): c))` fails with `expected ")"` where dart-sass prints the call
back. Closing it means matching every name in both predicates without regard
to case, and checking the strictness count again afterwards.

No test in the pinned spec revision covers any of it, which is why it is
recorded here rather than fixed alongside the rest of the section.

## 5. A bare `%` is not a value -- landed in #38

`spec/css/percent/{declaration,function}/{alone,before,after}`, and the gate
on section 4's 35 colour fixtures.

All 6 now pass; the suite went from 375 failures to 369 with no fixture
regressing. The colour fixtures moved from a parse error to the channel
check they were always meant to reach:

```
a {b: rgb(attr(c, %), 2, 3)}
Error: $red: attr(c, %) is not a number.
```

That is section 4's remaining work, and nothing else stands in front of it.

The rest of this section is kept as written, because the grammar it records
is not obvious from the diff.

### Current behavior (before the change)

A `%` on its own is a parse error wherever a value is expected:

| input | accent-sass | dart-sass 1.103.1 |
|---|---|---|
| `a {b: %}` | `Error: expected ")".` | `b: %;` |
| `a {b: % c}` | error | `b: % c;` |
| `a {b: c %}` | error | `b: c %;` |
| `a {b: c(%)}` | error | `b: c(%);` |
| `a {b: attr(c, %)}` | error | `b: attr(c, %);` |

All six `spec/css/percent` fixtures are these shapes, three in a declaration
and three inside a function call. All six are "Test case should succeed but
it did not".

### Reference behavior

dart-sass parses a lone `%` as an unquoted string and prints it back, in a
declaration value and as a function argument alike. It is not a number, and
nothing arithmetic happens to it.

### What the change was

`%` is a real value in dart-sass, not a raw-text fallback: `$x: %` binds it
to a variable and `rgb(%)` reaches rgb's channel check. Two rules separate it
from the modulo operator, and both were established against the binary rather
than assumed:

- **A `%` is the operator only when an operand follows it.** `5 % 2` and
  `5 %2` are modulo; `c %` and `1 %` are two-element lists, because nothing
  that could be an operand follows. The lookahead skips whitespace and
  comments, so `c % /* d */` is a list too, and it asks whether the next
  token starts an expression rather than matching a list of terminators --
  `1 %*2` reads the `%` as a value and then fails in evaluation with
  `Undefined operation "% * 2"`, which is what dart-sass does.
- **Plain CSS keeps the operator.** `a {b: c %}` in a `.css` file is
  `Operators aren't allowed in plain CSS.` in both engines, while
  `a {b: %}`, `% c` and `c(%)` compile, so the plain-CSS check belongs after
  the single-expression case, not before it.
- **A `%` value is rejected once the expression has consumed a comma.**
  dart-sass takes `%, 2` and `1 %, 2` but rejects `1, %, 2` and `[1, %]`. It
  accepts `(1, %)` and `c(1, %)` because parentheses and arguments parse each
  element as its own expression, which resets that state.

The comma rule reads like an implementation quirk rather than a design, but
it is consistent across every shape tested, and matching it costs one
condition. `looking_at_expression` also had to learn that `%` starts an
expression, or `(%)` and `c(%)` would close their parens early.

A `%` value also has to clear `allow_slash`, which `add_operator` does for a
real operator. Without it `1/2 %` printed `1/2 %` where dart-sass prints
`0.5 %` -- a slash list surviving where the division should have resolved.

One difference remains, unrelated to the six fixtures: `min(%)` errors with
`% is not a number` where dart-sass prints `min(%)`. `min()`, `max()` and
`clamp()` parse as calculations, and dart-sass falls back to a plain CSS
function when an argument is not a valid calculation value. That fallback is
missing here, and it is a calculation defect rather than a percent one --
no spec test in the pinned revision covers it.

## Testing

- Ground truth: `spec/css/percent.hrx`,
  `spec/css/functions/special/comment.hrx`,
  `spec/css/functions/special/prefixed/{lowercase,uppercase}.hrx`,
  `spec/css/functions/special/unprefixed.hrx`,
  `spec/css/functions/special_variable.hrx`, and the `special_functions`
  files under `spec/core_functions/color/{rgb,hsl,lab}` at the pinned
  revision.
- Scoped spec runs: `spec/css/functions`, `spec/css/percent`,
  `spec/css/unknown_directive` and `spec/core_functions/color` (see
  [README.md](README.md)).
- Add regression tests to `crates/lib/tests/` with the `test!` and `error!`
  macros. Cover at least: a silent comment in each position from section 1,
  a single-quoted string inside a special function, `TYPE(0)` against
  `-a-type('x')`, and `attr()` in both a single channel and the
  one-argument form.
- Verify every new expectation against dart-sass 1.103.1 before committing
  it. There is no local binary; run `npx -y sass@1.103.1`, never bare
  `npx sass`, which resolves to whatever is current.

## Acceptance criteria

- `spec/css/functions` passes under the roadmap's standard flags: 0
  failures, down from 22.
- ~~`spec/css/percent` passes: 0 failures, down from 6.~~ Done.
- ~~With sections 4 and 5 both landed, `spec/core_functions/color` drops from
  57 failures to 22, all 35 `attr` fixtures passing.~~ Done: 22, and the 20
  left in `spec/css/functions` are sections 1 to 3.
- `spec/css/unknown_directive` drops by at least 3, from sections 1 and 2
  reaching `almost_any_value`.
- ~~The whole-suite count of "Expected test to fail but it did not" has not
  risen above 29, so sections 4 and 5 did not buy their tests with
  leniency.~~ Done: still 29.
- The `frameworks` CI job still reports no colour-value differences.
