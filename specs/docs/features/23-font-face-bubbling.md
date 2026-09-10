# @font-face bubbling

5 sass-spec failures in `spec/css/font-face`, all one cause, and the fix is
one condition.

Measured 2026-09-10 against master (`31361d7`), the pinned sass-spec
revision `4a9eea66`, and dart-sass 1.103.1 run as `npx -y sass@1.103.1`.

When an at-rule is nested in a style rule, Sass copies the style rule into
it so that declarations written directly inside the at-rule have somewhere
to go: `a {@foo {b: c}}` becomes `@foo {a {b: c}}`. `@font-face` is the
exception. Its descriptors belong to the at-rule itself, so dart-sass does
not copy the selector in:

```scss
a { b: c; @font-face { d: e } }
```

```css
a {
  b: c;
}
@font-face {
  d: e;
}
```

This compiler produces `@font-face { a { d: e } }`.

## The condition

`visit_unknown_at_rule` in `crates/compiler/src/evaluate/visitor.rs:2200`:

```rust
if !visitor.style_rule_exists() || visitor.flags.in_keyframes() {
```

dart-sass 1.103.1 has a third term. From the package
(`npm pack sass@1.103.1`, `package/sass.dart.js`, line 81868):

```js
if (styleRule == null || t1._inKeyframes || _this.name.value === "font-face")
```

Add the same test. Two details it fixes only if you copy them exactly:

- The comparison is against the **plain, unvendored** name. `unvendor` is
  used two lines above for `keyframes`, but dart compares `name.value`
  directly, so `@-moz-font-face` still bubbles.
- It is case-sensitive, and it reads the name after interpolation is
  resolved, so `#{"font-face"}` is caught too. No fixture covers that, or
  the `@FONT-FACE` spelling; pin both with tests.

## The empty case

`font-face/bubble/empty` needs item 18 as well. dart-sass prints
`@font-face { /**/ }` on one line, because the comment is the only child
and it started on the same line as the brace; this compiler will print the
comment on its own line even after the condition is fixed. The fixture is
counted here rather than under item 18 because the bubbling is the larger
half of its diff.

## Testing

- Ground truth: `spec/css/font-face.hrx` at the pinned revision, which
  covers a nested rule, a mixin, three levels of nesting, and a file
  brought in by `@import` and by `meta.load-css`. The last of those,
  `bubble/loaded/meta-load-css`, passes today, so it is the control.
- Scoped spec runs: `spec/css/font-face`, `spec/css/keyframes`, and
  `spec/directives/at_root` -- the condition sits on the path every unknown
  at-rule takes.
- Add regression tests to `crates/lib/tests/` with `test!`: `@font-face`
  nested in a style rule, `@-moz-font-face` nested the same way (which must
  still bubble), and an interpolated `#{"font-face"}`.
- Verify every new expectation against dart-sass 1.103.1 with
  `npx -y sass@1.103.1`, never bare `npx sass`.

## Acceptance criteria

- `spec/css/font-face` drops from 5 failures to 1, and to 0 once item 18's
  section 1 lands.
- No other area moves. The condition is one name, but it is on a hot path,
  so the whole-suite count is the measurement that counts.
