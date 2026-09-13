# Bogus combinators

**Landed. The suite went from 173 failures to 111.** Sixty-two fixtures
closed, not the eighteen counted below. All eighteen of this item's pass,
and so do all 24 `spec/css/selector/combinator/` failures and all 20
`spec/core_functions/selector` failures with `combinator` in their path --
the residue item 04 counts that [Testing](#testing) asked about. Nothing
regressed, and "Expected test to fail but it did not" stayed at 16, so no
leniency paid for the omissions. Measured on master (`7763e204`) against
sass-spec `b39c32768` and dart-sass 1.104.0, native binary.

`css/selector` is down from 25 to 1 (`attribute/empty_namespace`) and
`core_functions/selector` from 25 to 5; none of the six left has a
combinator in its path. `.github/scripts/frameworks.sh`, run locally
against the same native binary, reports the same counts before and after
-- 5, 0, 0 and 903 differing lines for Bulma, Pico, Foundation and USWDS,
none colour-bearing -- so no rule in those four was dropped. [How it landed](#how-it-landed) records where the
work differed from the instructions below.

18 sass-spec failures, the deepest cause in the unclaimed residue, spread
over five areas that no document claims. Every one of them is the same
divergence: dart-sass drops a selector whose combinators cannot match
anything, and this compiler prints it.

Measured 2026-09-10 against master (`31361d7`), the pinned sass-spec
revision `4a9eea66`, and dart-sass 1.103.1 run as `npx -y sass@1.103.1`.
Re-measured 2026-09-11 on master (`f55ace41`) against sass-spec `b39c32768`:
all 18 still fail.

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

**Update, 2026-09-13:** the warning is emitted now. A style rule and an
`@extend` inside one report it with dart-sass 1.104.1's message and source
frame. Measured on the 33 fixture files that expect it, with warnings
compared: 100 failures before, 48 after. Of the 48, 42 are the warning the
selector functions give (`$extender: > is not valid CSS.`), and 6 are the
indented syntax's separate "This selector doesn't have any properties"
warning; neither is this rule.

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
- Also run `spec/css/selector` and `spec/core_functions/selector`, which
  item 04 counts as residue. On `f55ace41`, 24 of `css/selector`'s 25
  failures are under `css/selector/combinator/`, and 20 of
  `core_functions/selector`'s 25 have `combinator` in their path. This
  document names none of them, and nobody has checked whether this fix
  reaches them. Record what moves.
- Add regression tests to `crates/lib/tests/` with `test!`, covering each
  row of the table above, including the two rows that must keep printing.
- Verify every new expectation against the reference, dart-sass 1.104.0
  since [item 25](25-baseline-before-dart-sass-1-104.md), using the native
  release binary. `npx sass` runs the JavaScript build, which gives
  different answers in places.

## Acceptance criteria

- `spec/directives/extend` and `spec/non_conformant/extend-tests` are clear.
  They failed 6 and 10 when this was written; #70, item 24's cause 3,
  closed the one other failure in each (`pseudo/into_pseudo/extends_after`
  and `extend-loop`), so the 5 and 9 left there are all this item's.
- `spec/non_conformant/scss` drops by 2, `spec/non_conformant/sass` by 1,
  and `spec/libsass-closed-issues/issue_439` passes.
- The whole-suite "Expected test to fail but it did not" count has not
  risen: dropping rules must not be paid for with new leniency elsewhere.
- The `frameworks` CI job still compiles Bulma, Pico, Foundation and USWDS
  with no colour-value difference. A rule that drops selectors is the kind
  that goes too far quietly, and those four are the broadest check there
  is that it has not.

## How it landed

The rules were ported from the dart-sass 1.104.0 source rather than
inferred from its output: `Selector.isBogus`, `isBogusOtherThanLeadingCombinator`
and `isUseless` in `lib/src/ast/selector.dart`, and their call sites in
`lib/src/extend/` and `lib/src/visitor/`. Three things in this document
turned out to be imprecise.

**The omission is invisibility, not a check in `visit_style_rule`.**
dart-sass keeps the rule in the tree. Its serializer skips any complex
selector that is `isBogusOtherThanLeadingCombinator`, through the same
`isInvisible` that hides placeholders, and a rule with nothing visible left
disappears. `ComplexSelector::is_invisible` now includes that clause, so the
omission works per complex selector: `.a + ~ b, .c {p: 1}` prints `.c`.

**A trailing combinator is always invisible.** Step 4 reads as if `.b >`
were dropped or kept depending on its children. It is always dropped; in
`.b > {c {p: 2}}` the rule that prints is the nested `.b > c`, which has no
trailing combinator. The children only decide which warning dart-sass
emits.

**The extend work is three rules, not one.** `add_extension` skips a
useless extender, which is what the nine `extend-tests` fixtures need.
`selector.extend()` does not pass through it, so the per-compound check had
to be ported too: an extension that becomes useless once the compound's
trailing combinators are appended is dropped, which is why
`selector.extend(".c ~ ~ .d", ".c", ".e")` returns only `.c ~ ~ .d`. And
`unify_complex` now carries a leading or trailing combinator onto the
unified base as dart-sass's `unifyComplex` does, where it used to give up on
any selector ending in a combinator. That last one closed three
`selector.extend` fixtures, such as `trailing_combinator/extender/child`,
that failed before this item for that reason alone.

Other smaller ports came along: a bogus selector inside `:not()` makes the
rule invisible and is never a superselector, and `:has()` allows one
leading combinator where `:is()` does not.

**Hidden from CSS, kept in values.** dart-sass hides a bogus selector only
when it writes CSS. When it prints a selector as a SassScript value, it
filters nothing, so `selector.parse(":is(.a > + .b)")` keeps its argument.
This compiler prints values through `SelectorList`'s `Display`, which
filters invisible complex selectors, and the first version of this change
made that filter drop bogus arguments too, giving `:is()`. A code review
caught it. The invisibility checks now take an `include_bogus` flag, like
dart-sass's `isInvisibleOtherThanBogusCombinators`, and `Display` leaves
bogus selectors in. The serializer, which writes CSS, still drops them.

The review also found that `selector.replace` panicked in `trim` when no
replacement survived, as in `selector.replace("a.b", ".b", "c")`. That was
already true on master, but the new useless filter opened another path to
it. dart-sass fails with "components may not be empty" only when the whole
list is empty, so `trim` now accepts an empty list and `selector.replace`
raises that error.

Seventeen existing tests in `crates/lib/tests/` had frozen the old output:
ten in `extend.rs`, five in `selector-unify.rs` and two in `selectors.rs`.
They wove useless extenders in, unified selectors with two combinators in a
row, and printed `> > foo` and `+`. weaving useless extenders in and printing
`> > foo`. Each was checked against the dart-sass 1.104.0 binary and
corrected. The new regression tests are in
`crates/lib/tests/bogus-combinators.rs`.
