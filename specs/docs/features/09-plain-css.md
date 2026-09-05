# Plain CSS

`spec/css/plain` was the deepest unclaimed area on the roadmap: 51
failures, 45 of them "rejects valid input". Measured 2026-09-04 against
the pinned sass-spec revision `4a9eea66` and the dart-sass 1.103.1
binary, under the roadmap's standard flags.

**This item is closed.** It took three pull requests, one per defect
group: CSS nesting passthrough (`zoosky/accent-sass` #27) closed 27 of
the 51, the CSS `@function` rule (#29) closed 8, and the remaining three
defects (#30) closed the last 16.

```
236 runs, 236 passing, 0 failures, 0 todo, 0 ignored, 0 errors
```

What each defect was, and what it turned out to be, is recorded below.

## Landed: CSS nesting passthrough

A `.css` file may nest style rules. Sass does not resolve that nesting --
it is CSS nesting, which the browser resolves -- so the rule keeps its own
selector and stays nested in the output:

```css
/* plain.css */
a {b {c: d}}
```

```css
/* output */
a {
  b {
    c: d;
  }
}
```

The rules that fall out of that, all of which dart-sass 1.103.1 applies
and this fork now applies too:

- `&` is the CSS nesting selector, not Sass's parent selector. It is
  written out unresolved, may appear anywhere in a compound selector
  (`.b&.c`), and may not carry a suffix (`&b` is an error).
- An at-rule directly inside the outermost rule still bubbles out of it,
  the way it does in Sass. Once nesting has been passed through, at-rules
  stay where they were written and media queries are not merged: the
  stylesheet already needs a browser that supports nesting.
- A leading combinator is fine in a nested rule and an error at the top
  level, where there is no enclosing rule to be relative to. A trailing
  combinator is an error anywhere.
- A plain CSS rule that uses `&` and is loaded into a Sass rule through
  `@import` or `meta.load-css` is nested under that rule rather than
  merged into it, so the `&` still reaches the browser.

## Landed: the CSS `@function` rule

`spec/css/plain/function/**` -- 8 failures, closed in #29 together with
`spec/css/function` (16) and one of `spec/directives/function`'s. A
`@function` whose name begins with `--` declares a CSS custom function,
which Sass passes through untouched; its `result` descriptor is parsed
like a custom property, taking the value verbatim rather than as
SassScript.

What #29 deliberately left is the rest of `spec/directives/function`:
the [function-name proposal]'s name rules (12 failures) and three
indented-syntax whitespace tests. Neither is about the CSS rule.

[function-name proposal]: https://github.com/sass/sass/tree/main/accepted/function-name.md

## Landed: whitespace in `@import ... supports(...)`

`spec/css/plain/import/whitespace/supports/**` -- 14 failures, closed in
#30.

```scss
@import "a.css" supports(
    a: b)
```

gave "Expected expression.". Two of the 14 were plain whitespace after
the open paren in SCSS, which the query parser simply did not skip. The
other twelve were the indented syntax, and they split again:

- Eleven were newlines. The query sits inside `supports(`...`)`, so a
  newline in it is whitespace the way it is in an argument list. The
  fork already models that with `enter_parens`; the import-modifier path
  did not use it, and neither did the newline case in
  `parse_interpolated_declaration_value`. (Item
  [10](10-indented-newlines.md) later replaced `enter_parens` with
  dart-sass's `consumeNewlines` parameter, which is where that behaviour
  lives now.)
- One, `supports/calc/sass`, was not about `supports()` at all: it was
  the trailing `;` in `@import "a.css" supports(calc(1));`. The indented
  syntax tolerates one, and `@import` was not calling
  `expect_statement_separator` in the first place. Adding both closed 22
  further failures elsewhere -- `spec/css/style_rule/sass`,
  `spec/core_functions/newlines`, and the `sass/semicolon` test under
  half a dozen directives.

## Landed: the CSS `if()` function in plain CSS

`spec/css/plain/if` -- 1 failure, closed in #30.

```css
a {b: if(css(1): c; css(2): d; else: e)}
```

gave `expected ")"`. `if()` with CSS-style conditions landed for Sass in
[02-css-if-function.md](02-css-if-function.md) (#13); the plain CSS
parser had its own `identifier_like` and did not reach that code path.
Routing it there exposed five error tests that had been passing for the
wrong reason: a `sass()` condition is settled at compile time, so plain
CSS rejects it.

## Landed: `//` inside a plain CSS value

`spec/css/plain/slash/without_intermediate/no_whitespace` -- 1 failure,
closed in #30.

```css
a {b: 1///bar}
```

gave "Silent comments aren't allowed in plain CSS.". There are no silent
comments in plain CSS, so `//` in a value position is just two slashes.
dart-sass's plain CSS parser answers "not a comment" rather than raising
when it is inside an expression, which needed the fork's
`skip_silent_comment` to gain a return value and the value parser to
record that it is inside one.

## Ground rules

The roadmap's [ground rules](README.md#ground-rules-for-every-item) apply:
verify every expectation against the dart-sass 1.103.1 binary, add
regression tests to `crates/lib/tests/`, and run both clippy toolchains
before committing.
