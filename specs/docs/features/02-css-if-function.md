# The CSS if() function

Unlocks 164 sass-spec tests, all under `spec/expressions/if`
(`sass` 48, `raw` 48, `syntax` 34, `css` 21, `short_circuit` 11,
`else` 2).

## Current behavior

grass knows only the Sass ternary `if($condition, $if-true, $if-false)`,
special-cased in `crates/compiler/src/parse/value.rs:1159` and
`crates/compiler/src/parse/value.rs:1597` and implemented as the builtin
`if_` registered in `crates/compiler/src/builtin/functions/meta.rs:344`.
The new CSS conditional syntax fails to parse:

```
Error: expected ")".
  ,
1 | a {b: if(sass(true): c; else: d)}
  |                    ^
```

## Reference behavior

dart-sass 1.103.1 additionally supports the CSS `if()` function:

```scss
a {b: if(sass($x > 3): big; else: small)}
b {c: if(media(min-width: 600px): 1em; style(--d: e): 2em; else: 3em)}
```

- The argument is a `;`-separated list of `condition: value` branches;
  the final branch may use the bare keyword `else`.
- A `sass(...)` condition wraps an ordinary Sass expression. dart-sass
  evaluates it at compile time and collapses the `if()` to the matching
  branch's value (the `sass/` and `short_circuit/` test groups).
- Conditions dart-sass cannot decide at compile time — `media()`,
  `supports()`, `style()` and raw condition text — are emitted as plain
  CSS for the browser to resolve (the `raw/` and `css/` groups), with
  Sass expressions inside branch values still evaluated.
- Branches after a statically-true condition are not evaluated
  (`short_circuit/`), and an `if()` whose every branch is statically
  false with no `else` produces `null`.

Both forms coexist: comma-separated arguments select the legacy ternary,
colon syntax selects the CSS form. Derive the exact disambiguation and
error behavior from the `syntax/` test group rather than from this
summary.

## Implementation instructions

1. In the `if` special case in `crates/compiler/src/parse/value.rs`,
   look ahead after `if(` to decide which form to parse. The CSS form
   needs its own AST node (add a variant to `AstExpr` in
   `crates/compiler/src/ast/expr.rs`) holding a list of
   condition/value branch pairs, where a condition is either a parsed
   Sass expression (from `sass(...)`), the `else` keyword, or
   uninterpreted interpolated text.
2. Parse branch values as space/comma value expressions terminated by
   `;` or the closing `)`. The `syntax/` group pins down whitespace,
   trailing `;`, nested parentheses and error recovery.
3. Evaluate in `crates/compiler/src/evaluate/visitor.rs`: walk the
   branches in order; a statically decidable condition either selects
   the branch (evaluate its value, return it, stop) or is dropped; the
   first undecidable condition switches the whole expression to CSS
   output mode, serializing remaining branches with their conditions
   verbatim and values evaluated.
4. Return `null` when no branch matches and there is no `else`, so
   `if(sass(false): c) == null` holds and the declaration is elided.
5. Keep the legacy ternary untouched, including its lazy-argument
   special-casing.

## Testing

- Ground truth: `spec/expressions/if/`. Scope the spec run with that
  path (see [README.md](README.md)).
- Add `test!` cases to `crates/lib/tests/` for: a true and false
  `sass()` branch, `else`, a passthrough `media()` branch, mixed
  decided/undecided branches, `null` elision, short-circuiting, and the
  legacy ternary still working. Add `error!` cases from the `error/`
  subtrees. Verify every expectation against the dart-sass 1.103.1
  binary first.

## Acceptance criteria

- `spec/expressions/if` passes apart from error-span differences.
- No regression in `spec/directives/if` (the `@if` rule) or in the
  legacy `if()` function tests.
