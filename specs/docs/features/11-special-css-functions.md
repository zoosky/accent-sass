# Special CSS functions

Unlocks the 22 sass-spec tests under `spec/css/functions`, the deepest area
no document claimed, the 6 under `spec/css/percent`, and 35 under
`spec/core_functions/color` that need two of the sections below together.
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
| 1 | A silent comment is copied into the output | 8 | -- |
| 2 | A quoted string is re-quoted | 8 | -- |
| 3 | `type()` is not a special function | 4 | -- |
| 4 | `attr()` and `if()` are not special variable strings | 2 | 35, with 5 |
| 5 | A bare `%` is not a value | 6 | 35, with 4 |

Sections 1 and 2 reach further than the table says. Both defects live in
functions that also parse unknown at-rule values, so they show up in
`spec/css/unknown_directive` too -- see the note at the end of section 2.

## 1. A silent comment inside a special function is copied through

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

## 2. A quoted string inside a special function is re-quoted

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

## 4. `attr()` and `if()` are not special variable strings

`spec/css/functions/special_variable/{attr,if}`, and 35 fixtures under
`spec/core_functions/color`

The 2 tests here are `rgb(attr(c))` and `rgb(if(css(): c))`. The 35 colour
fixtures need this section **and** section 5; see "What this section does not
unlock" below before you plan around them.

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

### Implementation instructions

Add `attr(` and `if(` to both predicates. `is_var` carries a minimum-length
guard derived from `"var(--_)"`; that guard is specific to custom properties
and must not be applied to the new prefixes.

Two things to check rather than assume:

- **`if(` is also a Sass function.** A Sass `if($cond, $a, $b)` is evaluated
  long before these predicates see a value, so the prefix should only ever
  match the CSS `if()` string that #13 introduced. Confirm with
  `rgb(if(true, 1, 2))`, which must keep evaluating the `if()` to `1` and
  then fail on the channel count. dart-sass 1.103.1 prints `$channels: The
  rgb color space has 3 channels but 1 has 1`, so this one belongs in an
  `error!`, not a `test!`.
- **Strictness.** `is_special_function` is consulted from 17 places, most of
  them the colour builtins, so widening it makes the compiler more lenient
  everywhere at once. The suite currently has 29 failures of the kind
  "accepts invalid input". Run the whole suite, not just the two scoped
  areas, and check that number has not risen.

## 5. A bare `%` is not a value

`spec/css/percent/{declaration,function}/{alone,before,after}`, and the gate
on section 4's 35 colour fixtures

### Current behavior

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

### Implementation instructions

Accept `%` as an identifier-like token in value position, in the declaration
parser and in the argument parser both -- the six fixtures split evenly
across the two, so fixing one leaves three failing.

This is what unblocks section 4's colour fixtures, so run
`spec/core_functions/color` after landing both. Landing this section alone
leaves those 35 failing at the channel check instead of the parser, which is
progress the tally will not show.

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
- `spec/css/percent` passes: 0 failures, down from 6.
- With sections 4 and 5 both landed, `spec/core_functions/color` drops from
  57 failures to 22, all 35 `attr` fixtures passing. Section 4 alone changes
  nothing there; do not read an unchanged 57 as the section having failed.
- `spec/css/unknown_directive` drops by at least 3, from sections 1 and 2
  reaching `almost_any_value`.
- The whole-suite count of "Expected test to fail but it did not" has not
  risen above 29, so sections 4 and 5 did not buy their tests with leniency.
- The `frameworks` CI job still reports no colour-value differences.
