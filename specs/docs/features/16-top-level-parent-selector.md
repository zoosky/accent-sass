# A parent selector at the top level

14 sass-spec failures across eight areas, all of them the same error:

```
Error: Top-level selectors may not contain the parent selector "&".
```

dart-sass 1.103.1 does not raise it. The restriction is gone from the
reference implementation, and this compiler still enforces it.

Measured 2026-09-10 against master (`31361d7`), the pinned sass-spec
revision `4a9eea66`, and dart-sass 1.103.1 run as `npx -y sass@1.103.1`.

| Area | Failures |
|---|---:|
| `spec/libsass/base-level-parent` | 4 |
| `spec/libsass-closed-issues/issue_1527` | 3 |
| `spec/libsass-closed-issues/issue_1644` | 2 |
| `spec/libsass-closed-issues/issue_1569` | 1 |
| `spec/libsass-closed-issues/issue_2260` | 1 |
| `spec/libsass-closed-issues/issue_2304` | 1 |
| `spec/libsass-closed-issues/issue_2482` | 1 |
| `spec/libsass/parent-selector` | 1 |

Every one is "Test case should succeed but it did not", so this is a
compiler that rejects input the reference accepts -- the failure kind that
items 09 and 10 were drawn from, and the last sizeable pocket of it left.

## What dart-sass does

It keeps the `&` and prints it. CSS nesting made `&` meaningful to the
browser, so a top-level `&` is no longer nonsense to be rejected; it is
text to pass through, exactly as `preserve_parent_selectors` already does
for plain CSS in this compiler.

```scss
& { foo { bar: baz } }
foo & { a: b }
.foo, & { c: d }
```

```css
& foo {
  bar: baz;
}
foo & {
  a: b;
}
.foo, & {
  c: d;
}
```

Two independent confirmations that the restriction is gone rather than
merely untested:

- No fixture in the pinned sass-spec revision expects the message. `grep -r
  "Top-level selectors may not" sass-spec/spec` returns nothing.
- The string does not appear in the dart-sass 1.103.1 package at all.
  `npm pack sass@1.103.1` and grep `package/sass.dart.js`: zero matches.

## Where the code is

`crates/compiler/src/selector/list.rs:164`, `resolve_parent_selectors`. When
`parent` is `None` it returns `self` unchanged if the list has no parent
selector, and otherwise returns the error:

```rust
None => {
    if preserve_parent_selectors || !self.contains_parent_selector() {
        return Ok(self);
    }
    return Err((
        "Top-level selectors may not contain the parent selector \"&\".",
        self.span,
    )
        .into());
}
```

The doc comment above the function describes the error as part of the
contract, so it changes with the code.

## Implementation instructions

Return `Ok(self)` in both branches -- that is, treat a missing parent the
way `preserve_parent_selectors` already treats plain CSS -- and update the
doc comment. Then check the shapes the fixtures do not cover before you
conclude it is a two-line change:

- `&--suffix` at the top level, where the parent selector carries a suffix.
  The suffix path is where a resolved `&` normally has to produce a single
  compound; with no parent there is nothing to attach to.
- `@at-root { & { ... } }`, which two of the four `base-level-parent`
  fixtures use. `@at-root` clears the parent, so this reaches the same
  branch by a different route.
- `&` in the indented syntax (`issue_2482`) and inside a module loaded by
  `@use` (`issue_2304`), where the error currently points at the loaded
  file rather than the entry point.
- Selector functions. `selector.parse("&")` and friends take their own path
  through the parser with `allow_parent`, and must not start accepting
  something they reject today.

## Testing

- Ground truth: `spec/libsass/base-level-parent/{root,imported}/*.hrx` and
  `spec/libsass-closed-issues/issue_1527.hrx` at the pinned revision.
- Scoped spec runs: `spec/libsass`, `spec/libsass-closed-issues`.
- Add regression tests to `crates/lib/tests/` with `test!`: `&` alone,
  `& foo`, `foo &`, `.foo, &`, and `&` under `@at-root`. Add an `error!`
  test for whichever shape still errors, if any survives.
- Verify every new expectation against dart-sass 1.103.1 with
  `npx -y sass@1.103.1`, never bare `npx sass`.

## Acceptance criteria

- All 14 fixtures above pass, and the whole-suite "Test case should succeed
  but it did not" count drops from 48 to 34.
- The whole-suite "Expected test to fail but it did not" count has not
  risen above 28.
- The `frameworks` CI job still reports no colour-value differences.
