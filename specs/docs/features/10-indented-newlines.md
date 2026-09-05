# Newlines in the indented syntax

A newline ends a statement in `.sass`, except where a statement cannot end
-- inside an argument list, after a binary operator, between `@for` and its
variable. dart-sass decides that per call site with a `consumeNewlines`
parameter on `whitespace()`. This fork modelled it with a parenthesis depth
counter, which covered the parentheses and nothing else.

**This item is closed.** Measured 2026-09-05 against the pinned sass-spec
revision `4a9eea66` and the dart-sass 1.103.1 binary, under the roadmap's
standard flags: the suite went from 511 failures to **397**, 114 closed and
none opened.

## The defect

`enter_parens` raised a counter for the duration of an argument list, a
`@use ... with (...)` configuration, an `@import ... supports(...)` query and a
CSS `if()`. While the counter was up, a newline was whitespace; otherwise it
ended the statement. That is a region, and dart-sass's parameter is not one:

```dart
  ForRule _forRule(LineScannerState start, Statement Function() child) {
    whitespace(consumeNewlines: true);
    var variable = variableName();
    whitespace(consumeNewlines: true);
    expectIdentifier("from");
    whitespace(consumeNewlines: true);
    var from = _expression(consumeNewlines: true, until: ...);
    if (exclusive == null) scanner.error('Expected "to" or "through".');
    whitespace(consumeNewlines: true);
    var to = _expression();
```

Every call in the header passes `true`, and the last one -- the `to`
expression -- passes `false`, because the statement *does* end after it and
the body follows on the next line. A region flag cannot express that: raising
it for the whole header would swallow the body into the expression.

So the parameter had to be the parameter.

## What changed

`BaseParser::whitespace`, `whitespace_without_comments` and
`expect_whitespace` each take a `consume_newlines: bool`, which only
`SassParser` reads; every other syntax treats a newline as whitespace always
and ignores it. The flag threads on into
`StylesheetParser::parse_expression`, `parse_interpolated_declaration_value`
and `parse_supports_condition`, matching dart-sass call site for call site.
`enter_parens`, `restore_parens`, `newlines_are_whitespace` and
`SassParser::parens_depth` are gone: the ~200 explicit values subsume them.

Two structural fixes came with it.

**The at-rule name.** `parse_at_rule` consumed the whitespace after `@name`
once, for every rule. dart-sass does not: `atRule` dispatches immediately and
each handler consumes its own leading whitespace with its own value -- `true`
for `@each`, `@for`, `@function`, `@include`, `@mixin`, `@use`, `@warn` and
the rest, `false` for `@at-root`, `@import`, `@media`, `@supports` and an
unknown at-rule. That single shared call was why `@function` followed by a
newline could not find its name. The same whitespace came out of
`plain_at_rule_name` and `parse_plain_at_rule_name`, which feed the
declaration-level and function-level dispatches.

**Brackets in a declaration value.** `parse_interpolated_declaration_value`
broke out of its loop on a newline whenever the parser was indented and the
depth counter was down. dart-sass also requires the bracket stack to be
empty, so a value split inside `[...]` or `(...)` keeps going. The condition
now reads `is_indented() && !consume_newlines && brackets.is_empty()`.

## What it closed

114 failures, every one of them an indented-syntax test. 68 of the paths carry
`/sass` as a segment; the other 46 are `.sass` inputs whose directory does not
say so (`spec/operators/newlines`, `spec/parser/interpolation/whitespace`,
`spec/directives/forward/member/newlines`).

| Area | Closed |
|---|---|
| `spec/css/supports` | 15 |
| `spec/values/lists` | 13 |
| `spec/directives/for` | 12 |
| `spec/css/media` | 10 |
| `spec/values/maps` | 9 |
| `spec/directives/each` | 8 |
| `spec/directives/forward` | 7 |
| `spec/directives/if` | 5 |
| `spec/operators/newlines`, `directives/mixin`, `directives/at_root` | 4 each |
| `spec/parser/interpolation`, `directives/use`, `directives/function` | 3 each |
| eleven more areas | 1-2 each |

`spec/directives/for`, `spec/directives/function`, `spec/values/lists`,
`spec/css/media` and `spec/css/style_rule` are now clear. `spec/css/supports`
has 6 left, none of them about newlines.

## Regression tests

`crates/lib/tests/indented-newlines.rs` pins both sides: seven headers that
may be split across lines, and two that may not (`@media` and `@supports`,
where dart-sass errors and so does this fork). Every expectation was checked
against the dart-sass 1.103.1 binary before it was written down.
