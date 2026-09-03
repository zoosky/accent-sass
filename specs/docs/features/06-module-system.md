# @use, @forward and @import edge cases

Unlocks 114 sass-spec tests: `spec/directives/use` 46,
`spec/directives/forward` 41, `spec/directives/import` 27. This is
also the area the project README calls out as the fork's rough edge,
so its real-world weight is higher than the raw count suggests.
Related: the `meta.load-css` strictness work in
[03-meta-module.md](03-meta-module.md) shares validation logic with
`@use ... with`.

## Current behavior

The failures fall into three clusters.

### Comments after configuration lists

Shared with [05-comments-and-arguments.md](05-comments-and-arguments.md):

```
Error: expected ";".
  ,
1 | @use "other" with ($a: b) /**/
  |                           ^
```

`parse_use_rule` (`crates/compiler/src/parse/stylesheet.rs:1641`) and
`parse_forward_rule` (`crates/compiler/src/parse/stylesheet.rs:1418`)
do not skip comments between the closing `)` and the `;`. About six
tests across `use/comment` and `forward/comment`.

### Missing error strictness

`use/error` (17) and `forward/error` (20) are mostly "expected test to
fail but it did not": accent-sass accepts member collisions, invalid
configurations, bad `show`/`hide` lists and namespace clashes that
dart-sass rejects.

### Semantics gaps

The remainder are behavioral: `@forward` prefixes interacting with
`@import` and `show`/`hide`, configured modules re-forwarded with a
second configuration, `@import` of files containing `@forward`, and
member visibility through chained forwards. The project README notes
this explicitly: importing modules that contain `@forward` with
prefixes may not behave as expected.

## Implementation instructions

1. Fix the comment-skipping first — it is a two-line change per rule
   (use the comment-aware `whitespace` from
   `crates/compiler/src/parse/base.rs:24` before expecting `;`) and
   clears the `comment/` groups.
2. For the error clusters, enumerate the `.hrx` files under
   `spec/directives/use/error/` and `spec/directives/forward/error/`,
   and add each missing check where the module system resolves members
   and applies configurations (module resolution lives in
   `crates/compiler/src/evaluate/visitor.rs` and the module types it
   uses). Take messages from the spec files; with `--trim-errors` only
   the first line must match.
3. For the semantics cluster, work test by test. Reduce each failing
   `.hrx` to a minimal reproduction, run it through dart-sass 1.103.1,
   and fix the resolution logic. Expect most fixes to land in how
   forwarded members are recorded (prefix application, show/hide
   filtering, configuration inheritance) rather than in parsing.
4. Keep `meta.load-css` in sync: any configuration validation you add
   for `@use ... with` should be shared with the `load-css`
   implementation, not duplicated.

## Testing

- Ground truth: `spec/directives/use`, `spec/directives/forward`,
  `spec/directives/import`. Scope the spec run with those paths (see
  [README.md](README.md)).
- Multi-file tests: the `.hrx` format bundles several files; the
  existing test suite has multi-file fixtures under
  `crates/lib/tests/` to copy the pattern from.
- Add `test!`/`error!` cases per fixed behavior, verified against the
  dart-sass 1.103.1 binary.
- Regression risk is module resolution at large: run the full
  `cargo test --features=macro` suite and the `frameworks` comparison,
  since Bulma, Foundation and USWDS exercise `@use`/`@forward`
  heavily.

## Acceptance criteria

- The three directive groups pass apart from error-span differences.
- No regressions in the local suite or the framework comparisons.
