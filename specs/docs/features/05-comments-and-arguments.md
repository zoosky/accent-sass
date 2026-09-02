# Comment positions and argument-list syntax

Unlocks about 100 sass-spec tests from two cross-cutting parser gaps:
56 failures whose test paths contain `comment`, spread across
`spec/css`, `spec/directives` and `spec/expressions`, and 46 failures
under `spec/callable` for trailing commas and indented-syntax argument
lists. Both are lexer/argument-parser work, so they share one document.

## Gap 1: comments in more positions

### Current behavior

grass rejects or mis-emits comments in positions dart-sass accepts.
Three representative failures:

```
Error: expected ";".
  ,
1 | @use "other" with ($a: b) /**/
  |                           ^
```

```
--- expected
+++ actual
 a {
-  b: -a-calc( c);
+  b: -a-calc(// c);
 }
```

```
--- expected
+++ actual
-@supports (--a:  b) {
+@supports (--a: // b) {
```

The first is a token position where the parser calls
`whitespace_without_comments` (`crates/compiler/src/parse/base.rs:12`)
or checks for `;` directly instead of skipping comments via
`whitespace` (`crates/compiler/src/parse/base.rs:24`). The second and
third come from the special-function and custom-property text parsers
(`crates/compiler/src/parse/value.rs`, around the interpolated-text
handling at lines 1431-1530), which copy a silent comment into the
output instead of consuming it; dart-sass strips it and leaves the
surrounding whitespace.

### Implementation instructions

1. Enumerate the 56 failing tests (`grep comment` over the failure
   list) and group them by parse site. The clusters observed:
   after `@use`/`@forward` config lists, inside special functions
   (`calc`-like, `element()`, `expression()`), inside `@supports`
   conditions and custom properties, around `@for` bounds
   (`spec/directives/for`, 5 tests), and in `spec/css/comment` and
   `spec/expressions/comments` themselves.
2. For token-position failures, replace the offending
   `whitespace_without_comments` call or direct peek with `whitespace`,
   which already skips both comment kinds.
3. For text-mode failures, teach the special-function/custom-property
   scanners to consume `//` to end of line (emitting nothing, keeping
   the newline handling dart-sass has) and `/* */` per the spec tests —
   some positions preserve loud comments, some drop them; take each
   case's expectation from its `.hrx` file.
4. Silent comments do not exist in plain `.css` input; make sure the
   plain-CSS parser (`crates/compiler/src/parse/css.rs:29`,
   `skip_silent_comment`) keeps rejecting them where it already does.

## Gap 2: argument-list syntax

### Current behavior

Two syntax forms are missing:

- Trailing commas after rest and keyword-rest arguments:
  `@include utils.a(1..., (c: 2)..., );` fails with `expected ")"`.
  Parameter lists (`@function a($b..., )`) have the same gap. The
  invocation parser is `parse_argument_invocation` in
  `crates/compiler/src/parse/stylesheet.rs:2247`.
- Newlines inside argument lists in the indented syntax: a `.sass`
  file may break `@function a(` across lines. The indented parser in
  `crates/compiler/src/parse/sass.rs` treats the newline as end of
  statement (its continuation handling at
  `crates/compiler/src/parse/sass.rs:111` only covers lines ending in
  a comma). 29 failures under `spec/callable/whitespace/newlines`.

### Implementation instructions

1. In `parse_argument_invocation` and the parameter-list parser,
   accept an optional trailing comma after `...` arguments, mirroring
   the acceptance already in place for plain arguments.
2. In the indented-syntax parser, allow newlines while inside an
   unclosed paren in a declaration/argument context. dart-sass's rule:
   within parentheses or brackets, the indented syntax behaves like
   SCSS with respect to newlines. Use
   `spec/callable/whitespace/newlines` as the exact contract, including
   its error cases.

## Testing

- Ground truth: the failing tests under `spec/callable`,
  `spec/css/comment`, `spec/css/functions/special`,
  `spec/css/supports/comment`, `spec/expressions/comments` and the
  comment tests inside `spec/directives`.
- Add `test!`/`error!` cases to `crates/lib/tests/` per fixed parse
  site, verified against the dart-sass 1.103.1 binary.

## Acceptance criteria

- The comment-path failure count drops to near zero without new
  failures in `spec/css/plain` (plain CSS must stay strict).
- `spec/callable` passes apart from error-span differences.
