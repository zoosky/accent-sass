# Special CSS functions

Unlocks the 22 sass-spec tests under `spec/css/functions`, the deepest area
no document claimed, and 35 more under `spec/core_functions/color` that share
one of the four causes below. Measured 2026-09-06 against master (`1c3608e`),
the pinned sass-spec revision `4a9eea66`, and dart-sass 1.103.1 run as
`npx -y sass@1.103.1`:

```
120 runs, 98 passing, 22 failures, 0 todo, 0 ignored, 0 errors
```

A *special function* is one whose arguments are not Sass expressions:
`element()`, `expression()`, `progid:...()`, `url()`, `type()`, and any
vendor-prefixed `calc()`. Their contents are read as text and printed back
nearly verbatim. Unprefixed `calc()` is not among them -- it parses as a real
calculation, which is why `CALC(0)` prints `0` while `-a-calc(0)` prints
itself. Every defect below is therefore text that should have been dropped,
text that should have been preserved, or a function that never took the text
path at all.

The four are independent and can land separately, but section 2 has to
precede section 3 for two of section 3's tests to pass. Section 4 is the one
worth doing first: it is 37 tests, not 2, because the same predicate governs
the colour functions.

| Section | Defect | Tests here | Elsewhere |
|---|---|---:|---:|
| 1 | A silent comment is copied into the output | 8 | -- |
| 2 | A quoted string is re-quoted | 8 | -- |
| 3 | `type()` is not a special function | 4 | -- |
| 4 | `attr()` and `if()` are not special variable strings | 2 | 35 |

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
2939). Change only the one inside `parse_interpolated_declaration_value`
unless a spec run says otherwise; `crates/compiler/src/parse/css.rs:158`
already passes `true` and is a different path.

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

### Current behavior

Two predicates decide whether an unevaluated string may stand where a number
is expected:

- `is_special_function` in `crates/compiler/src/utils/mod.rs:36` accepts
  `calc(`, `var(`, `env(`, `min(`, `max(` and `clamp(`.
- `Value::is_var` in `crates/compiler/src/value/mod.rs:336` accepts `var(`
  alone, and governs the forms where one argument stands for several
  channels.

Neither knows `attr(` or `if(`, so every colour function rejects them:

```
a {b: rgb(attr(c))}
Error: $channels: Expected red channel to be a number, was attr(c).
```

All 35 colour fixtures fail this way -- every one is "Test case should
succeed but it did not", not a wrong-output difference. They divide as 14
under `rgb`, 14 under `hsl` and 7 under `lab`.

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

### Implementation instructions

Add `attr(` and `if(` to both predicates. `is_var` carries a minimum-length
guard derived from `"var(--_)"`; that guard is specific to custom properties
and must not be applied to the new prefixes.

Two things to check rather than assume:

- **`if(` is also a Sass function.** A Sass `if($cond, $a, $b)` is evaluated
  long before these predicates see a value, so the prefix should only ever
  match the CSS `if()` string that #13 introduced. Confirm with a test that
  `rgb(if(true, 1, 2))` still evaluates rather than passing through.
- **Strictness.** `is_special_function` is consulted from 17 places, most of
  them the colour builtins, so widening it makes the compiler more lenient
  everywhere at once. The suite currently has 29 failures of the kind
  "accepts invalid input". Run the whole suite, not just the two scoped
  areas, and check that number has not risen.

## Testing

- Ground truth: `spec/css/functions/special/comment.hrx`,
  `spec/css/functions/special/prefixed/{lowercase,uppercase}.hrx`,
  `spec/css/functions/special/unprefixed.hrx`,
  `spec/css/functions/special_variable.hrx`, and the `special_functions`
  files under `spec/core_functions/color/{rgb,hsl,lab}` at the pinned
  revision.
- Scoped spec runs: `spec/css/functions` and `spec/core_functions/color`
  (see [README.md](README.md)).
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
- `spec/core_functions/color` drops from 57 failures to 22, all 35 `attr`
  fixtures passing.
- The whole-suite count of "Expected test to fail but it did not" has not
  risen above 29, so section 4 did not buy its tests with leniency.
- The `frameworks` CI job still reports no colour-value differences.
