# A parent selector at the top level

**Landed. The suite went from 284 failures to 266.** Eighteen fixtures
closed, not the fourteen counted below: four of them are in areas a document
already claims -- `spec/css/selector/parent/{alone/first,in_at_rule}`,
`spec/core_functions/selector/nest/parent/alone/first` and
`spec/directives/import/top_level_parent` -- so the cause reached past the
areas this item was scoped to. Nothing regressed, and the "Test case should
succeed but it did not" column fell from 48 to 30.

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

## What the change was

`Ok(self)` in both branches -- a missing parent is now treated the way
`preserve_parent_selectors` already treats plain CSS -- with one exception
found by checking the shapes no failing fixture covers.

**A suffixed parent is still an error, and dart-sass words it differently.**
`&--x` at the top level gives `A top-level selector may not contain a parent
selector with a suffix.` in dart-sass 1.103.1, and so do `:is(&--x)` and
`foo &--x`: the rule is anywhere in the list, including inside a
pseudo-selector's argument. Six fixtures in the pinned revision expect that
message, and all six passed before the change only because the standard flags
ignore error text -- this compiler was raising the other message. Deleting the
check outright would have turned six passing tests into failures without the
whole-suite count showing a net loss.

`ComplexSelector::contains_suffixed_parent_selector` is the new predicate,
mirroring `contains_parent_selector` and recursing into pseudo-selector
arguments the same way.

Three shapes were checked and needed nothing: `@at-root { & { ... } }`, which
two of the four `base-level-parent` fixtures use; `&` in the indented syntax
(`issue_2482`) and inside a module loaded by `@use` (`issue_2304`); and the
selector functions, where `selector.nest("&c")` reaches the same branch and so
gets the suffix error for free.

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

- ~~All 14 fixtures above pass, and the whole-suite "Test case should
  succeed but it did not" count drops from 48 to 34.~~ Done, and further:
  18 fixtures, 48 to 30.
- ~~The whole-suite "Expected test to fail but it did not" count has not
  risen above 28.~~ Done: still 28.
- The `frameworks` CI job still reports no colour-value differences.
