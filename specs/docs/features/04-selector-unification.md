# Selector unification ordering

Unlocks 93 sass-spec tests under `spec/core_functions/selector`:
`unify` 42, `is_superselector` 24, `extend` 20, plus a handful under
`replace` and `simple_selectors`. The same code paths back `@extend`,
so fixes here also affect real stylesheets, not only the introspection
functions.

## Current behavior

The failures are output mismatches, not crashes. The dominant symptom
is compound-selector ordering during unification:

```
--- expected
+++ actual
 a {
-  b: .c.e > .d.f;
+  b: .e.c > .d.f;
 }
```

`selector.unify(".c > .d", ".e > .f")` must produce `.c.e > .d.f` —
simple selectors from the first operand come before those merged in
from the second — but accent-sass emits the merged selector's simples first.
The `is_superselector` and `extend` groups fail on related judgment
calls in the same machinery, including newer combinator forms such as
`:has(+ ~ a)` that accent-sass mishandles.

## Where the code lives

- `crates/compiler/src/selector/compound.rs:214` — `Compound::unify`,
  the likely home of the ordering bug.
- `crates/compiler/src/selector/simple.rs:174-330` — `Simple::unify`
  and its `unify_default`, `unify_universal`, `unify_type` and
  `unify_pseudo` helpers, which decide where a merged simple selector
  is inserted into the target compound.
- `crates/compiler/src/selector/extend/functions.rs:13` —
  `unify_complex`, plus the combinator handling in the same file.
- `crates/compiler/src/selector/list.rs:120` and
  `crates/compiler/src/selector/mod.rs:66` — the list- and
  complex-level entry points.
- `is-superselector` logic lives alongside these in the `selector`
  module.

## Implementation instructions

1. Start from the `unify/complex/combinators` failures: reduce one
   (`@debug selector.unify(".c > .d", ".e > .f")`) against both
   binaries and trace where the operand order flips. The dart-sass
   rule of thumb: unification inserts the second operand's simple
   selectors into the first operand's compound, preserving the first
   operand's order, with pseudo-elements and their subordinate
   pseudo-classes kept at the end. Confirm the exact rule from the
   dart-sass source and the spec expectations rather than from this
   summary.
2. Fix the insertion order in `Simple::unify` /
   `Compound::unify` first, re-run the `unify` group, and only then
   move to `is_superselector` and `extend` — many of their failures
   should collapse once the shared helpers are right.
3. For the combinator cases (`:has(+ ~ a)` and the
   `css/selector/combinator` tests, 24 more under `spec/css`), check
   how the selector parser in `crates/compiler/src/selector/parse.rs`
   validates leading/multiple combinators inside `:has()`; dart-sass
   accepts them there while rejecting them in ordinary selectors.
4. Treat this as porting dart-sass's algorithm, not patching symptoms:
   each divergence you find in `unify`/`is_superselector` semantics is
   worth a comparison against the dart-sass implementation before you
   change accent-sass, because `@extend` correctness depends on the same
   invariants.

## Testing

- Ground truth: `spec/core_functions/selector/` and
  `spec/css/selector/`. Scope the spec run with those paths (see
  [README.md](README.md)).
- Add `test!` cases to `crates/lib/tests/` for each ordering rule you
  fix, verified against the dart-sass 1.103.1 binary.
- `@extend` is the regression risk: after the change, re-run the full
  local suite and the `bootstrap` and `frameworks` comparisons, and
  check `spec/non_conformant/extend-tests` does not get worse.

## Acceptance criteria

- The `unify`, `is_superselector` and `extend` groups pass apart from
  error-span differences.
- No new differing lines in the `bootstrap` and `frameworks` CI
  comparisons.
