# Bogus combinators

18 sass-spec failures, the deepest cause in the unclaimed residue, spread
over five areas that no document claims. Every one of them is the same
divergence: dart-sass drops a selector whose combinators cannot match
anything, and this compiler prints it.

Measured 2026-09-10 against master (`31361d7`), the pinned sass-spec
revision `4a9eea66`, and dart-sass 1.103.1 run as `npx -y sass@1.103.1`.

| Area | Failures |
|---|---:|
| `spec/non_conformant/extend-tests` (129, 130, 133-139) | 9 |
| `spec/directives/extend/bogus` | 5 |
| `spec/non_conformant/scss` (`css_selector_hacks`, `weird-selectors`) | 2 |
| `spec/non_conformant/sass/selectors` | 1 |
| `spec/libsass-closed-issues/issue_439` | 1 |

## What dart-sass does

A *bogus* selector is one whose combinators are not valid CSS. dart-sass
1.103.1 warns about all of them under the `bogus-combinators` deprecation
and, for the subset that can never match, omits the rule. The exact rule,
established by running each shape through the binary:

| Selector | dart-sass 1.103.1 | This compiler |
|---|---|---|
| `.a > + x {p: 1}` | warns, omits the rule | prints it |
| `> > .d {p: 4}` | warns, omits the rule | prints it |
| `.g + ~ .h {p: 6}` | warns, omits the rule | prints it |
| `.b > {p: 2}` | warns, omits the rule | prints it |
| `> .c {p: 3}` | warns, **prints** `> .c` | prints it |
| `.e { > .f {p: 5} }` | prints `.e > .f`, no warning | agrees |

So two combinators in a row, or more than one leading combinator, make a
selector useless: it is always dropped. A single leading combinator is
legal CSS nesting and is kept -- the warning there says only "is invalid
CSS", without "It will be omitted from the generated CSS". A trailing
combinator is dropped when the rule has children that are not style rules;
dart's message names that case separately ("is only valid for nesting and
shouldn't have children other than style rules").

The second half is `@extend`. A bogus selector may not be an extender:

```scss
.a x {a: b}
.b > + y {@extend x}
```

dart-sass prints `.a x` and warns that `.b > + y` "can't be an extender".
This compiler prints `.a x, .a .b > + y, .b .a > + y`, having woven the
bogus extender into the result. Seven of the nine `extend-tests` failures
are that shape; the other two are the extendee being useless, which makes
the whole rule disappear.

## Why the deprecation warning does not gate this

The 18 failures are counted under the roadmap's standard flags, which pass
`--ignore-warning-diffs`. What the fixtures check is the *output*, so the
work is the omission and the extend refusal. Emitting the warning text
needs the warning facility that item 08's gap 1 describes and that the
compiler does not have; it is not required here, and adding the omission
without the warning closes every one of these tests.

## Where the code is

- `crates/compiler/src/selector/complex.rs` holds `ComplexSelector`. Its
  components are a flat `Vec<ComplexSelectorComponent>` in which a
  combinator is itself a component, so "two combinators in a row" reads as
  two adjacent `ComplexSelectorComponent::Combinator` values and "leading
  combinators" as the run of them at index 0. The comment at line 51
  already notes that adjacent combinators are possible; nothing classifies
  them, and that classification is the new part.
- `crates/compiler/src/evaluate/visitor.rs` emits the style rule
  (`visit_style_rule`). This is where a useless selector has to stop the
  rule from reaching the tree.
- `crates/compiler/src/selector/extend/` runs `@extend`. This is where a
  bogus extender has to be refused rather than woven.

## Implementation instructions

1. Add `is_useless` and `is_bogus` to `ComplexSelector`, matching the table
   above: useless is more than one leading combinator component or any two
   adjacent combinator components; bogus is useless, or a single leading
   combinator, or a trailing combinator.
2. In `visit_style_rule`, drop the rule when its resolved selector list has
   no non-useless complex selector. Keep the single-leading-combinator case:
   `> .c` must still print.
3. In the extend machinery, skip an extender whose selector is bogus. The
   test to aim at is `134_test_combinator_unification_for_hacky_combinators`,
   where the extendee is fine and only the extender is bogus.
4. Check the trailing-combinator case separately. `.b > {p: 2}` is dropped
   because its child is a declaration, while `.b > {c {p: 2}}` is kept.

## Testing

- Ground truth: `spec/directives/extend/bogus.hrx` and
  `spec/non_conformant/extend-tests/{129,130,133..139}_*.hrx` at the pinned
  revision.
- Scoped spec runs: `spec/directives/extend`,
  `spec/non_conformant/extend-tests`, `spec/non_conformant/scss`.
- Add regression tests to `crates/lib/tests/` with `test!`, covering each
  row of the table above, including the two rows that must keep printing.
- Verify every new expectation against dart-sass 1.103.1 with
  `npx -y sass@1.103.1`, never bare `npx sass`.

## Acceptance criteria

- `spec/directives/extend` drops from 6 failures to 1, the one left being
  `pseudo/into_pseudo/extends_after`, which item 24 covers.
- `spec/non_conformant/extend-tests` drops from 10 to 1, the one left being
  `extend-loop`, which item 24 covers.
- `spec/non_conformant/scss` drops by 2, `spec/non_conformant/sass` by 1,
  and `spec/libsass-closed-issues/issue_439` passes.
- The whole-suite "Expected test to fail but it did not" count has not
  risen: dropping rules must not be paid for with new leniency elsewhere.
- The `frameworks` CI job still compiles Bulma, Pico, Foundation and USWDS
  with no colour-value difference. A rule that drops selectors is the kind
  that goes too far quietly, and those four are the broadest check there
  is that it has not.
