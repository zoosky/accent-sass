# The unclaimed tail

39 sass-spec failures that no other document claims and none of which is
deep enough to earn one. They are 12 causes; the largest is 4 failures.
This document exists so the tail is *read* rather than counted, and so
whoever picks one up starts from a cause instead of a diff.

Measured 2026-09-10 against master (`31361d7`), the pinned sass-spec
revision `4a9eea66`, and dart-sass 1.103.1 run as `npx -y sass@1.103.1`.

| # | Cause | Failures |
|---|---|---:|
| 1 | Missing strictness checks | 8 |
| 2 | An arglist does not keep its separator | 4, one of them since closed |
| 3 | `@extend` produces the wrong selector set | 4 |
| 4 | `@extend` across media queries is not an error | 3 |
| 5 | Nested `@media` is flattened | 3 |
| 6 | A childless at-rule jumps its place in the rule | 3 |
| 7 | `@import` is hoisted past the comments above it | 2 |
| 8 | `if()` does not take a rest argument | 2 |
| 9 | A hex colour with alpha keeps its source text | 2 |
| 10 | A named argument does not reach a variadic parameter | 1 |
| 11 | A private-use character is not escaped | 1 |
| 12 | Six singletons | 6 |

## 1. Missing strictness checks

`core_functions/list/join/error/named`,
`css/keyframes/error/in_keyframe_block/style_rule`,
`css/mixin/error/css/mixin`,
`css/propset/error/custom_property/{simple,nested/complex}`,
`directives/mixin/custom_ident_include`,
`parser/interpolation/error/partial_bracket/{scss,sass}`

Eight inputs dart-sass rejects and this compiler accepts. They share a
kind, not a mechanism, so they are eight small changes:

| input | error dart-sass raises |
|---|---|
| `list.join(c, d, $invalid: true)` | `No parameter named $invalid.` |
| `@keyframes a { to {to {c: d}} }` | `Style rules may not be used within keyframe blocks.` |
| `@mixin --a {}` | `Sass @mixin names beginning with -- are forbidden...` |
| `a { b: { --d: e } }` | `Declarations whose names begin with "--" may not be nested.` |
| `@include --a` | as above, for `@include` |
| a partial `#{` bracket | a parse error |

The whole-suite "Expected test to fail but it did not" count is 28, so
these eight are a quarter of that column. Item 17's section 3 is another
one, counted there.

`list/join/error/named` is closed by `fix/builtin-parameter-lists`, together
with section 10. See there.

## 2. An arglist does not keep its separator

`non_conformant/sass/var-args/success`,
`non_conformant/scss-tests/{071,090}_*`,
~~`libsass-closed-issues/issue_1269`~~ -- closed by item 21

```scss
@mixin foo($a, $b...) { b: $b }
$list: 3 4 5;
.foo {@include foo(1, 2, $list...)}
```

dart-sass prints `b: 2 3 4 5`; this compiler prints `b: 2, 3, 4, 5`. When a
list is splatted into a rest parameter, the resulting arglist takes that
list's separator, and this compiler always uses a comma. `issue_1269` is
the same defect reached through `list.join($list, $items, $separator: auto)`,
where the arglist's separator decides the result.

This is the arglist half of item 21: there, `Value::Map` and `Value::List`
do not interchange; here, `Value::ArgList` carries the wrong separator.

Item 21 has since taken `issue_1269` with it, by matching `Value::ArgList`
wherever it matched `Value::Map` -- `list.join` was reading a hard-coded
comma rather than the value's separator. The three left are a different
defect and are not fixed by that: they need the arglist to be *built* with
the separator of the list that was splatted into it, which happens in
argument evaluation rather than in a list builtin.

## 3. `@extend` produces the wrong selector set

`libsass-closed-issues/issue_{1091,2055}`,
`non_conformant/extend-tests/extend-loop`,
`directives/extend/pseudo/into_pseudo/extends_after`

Four fixtures where the extend machinery runs to completion and produces a
different set of selectors. Two directions:

- **Too many.** `issue_1091` gets `.a, .d > .e, .b .c, .b .d > .e` where
  dart-sass stops at `.b .c`; `issue_2055` produces a shorter `:not()`
  chain than dart-sass does. Both point at the trimming pass that drops a
  selector already covered by another.
- **Too few.** `extend-loop` is missing `.z2.x2.y2.b2` from two of its
  media blocks, and `into_pseudo/extends_after` is missing an
  `:is(midstream)` alternative.

These are the last of item 04's territory that item 04 does not cover, and
they are genuinely fiddly; take them one at a time, and read
`crates/compiler/src/selector/extend/` before assuming which pass is wrong.

**Closed by `fix/extend-selector-set`.** The trimming pass was not wrong. The
four came from three places where the port had drifted from dart-sass
1.103.1:

- `issue_1091`: `ComplexSelector::is_super_selector` was the pre-rewrite
  algorithm. It rejected `.d > .e` as a superselector of `.b .d > .e`. It is
  now a port of `complexIsSuperselector`, which also closes nine
  `is_superselector/complex` fixtures.
- `into_pseudo/extends_after`: `extend_complex` rebuilt a single-selector
  path instead of returning it. That dropped the selector's identity, and with
  it its place in `originals`.
- `extend-loop` and `issue_2055`: `add_extension` copied the target's
  extensions-by-extender list up front. dart-sass holds a live reference, so
  an extender added in the same call is extended too.

## 4. `@extend` across media queries is not an error

`libsass-closed-issues/issue_{673,712,1923}`

```scss
.foo { content: 'foo' }
@media print { .bar { @extend .foo } }
```

dart-sass raises `You may not @extend selectors across media queries.` and
names both spans. This compiler extends silently. This is the same family
as `zoosky/accent-sass` #34, which made an out-of-scope mandatory `@extend`
an error and cut the "accepts invalid input" column from 43 to 29; the
media-query rule is the next one in that family.

## 5. Nested `@media` is flattened

`non_conformant/scss/media/nesting/{removed,retained}`,
`libsass-closed-issues/issue_2154`

A `@media` inside a `@media` either merges with its parent or stays
nested, depending on whether the queries can be combined. This compiler
always hoists the inner rule to the top level and, when the queries do
merge, emits the merged block in the wrong order relative to the outer
one. `retained` is the clearer of the two: dart-sass keeps
`@media not screen and (color) { a {..} @media screen { x {..} } }` nested,
and this compiler splits it into two top-level blocks.

**Closed by `fix/nested-media-merge`.** Hoisting was not the cause; the
bubbling logic already matched dart-sass. There were two defects:

- `removed` and `retained`: `MediaQuery::merge` compared the modifiers where
  dart-sass compares the types, in the branch where exactly one query is
  negated. The modifiers always differ there, so `not screen` merged with
  `screen` came out as `screen` instead of empty or unrepresentable.
- `issue_2154`: the tree's `has_following_sibling` counted invisible
  siblings. An empty bubbled `@media` split its parent in two. It now skips
  them, as dart-sass's `hasFollowingSibling` does. Declarations, comments,
  childless at-rules and nested imports keep the any-sibling test, because
  dart-sass's `_copyParentAfterSibling` does not skip invisible nodes. They
  now go through their own `add_child_after_sibling`.

## 6. A childless at-rule jumps its place in the rule

`css/unknown_directive/semicolon/nested/interleaved/{final,before_rule,before_declaration}`

```scss
a {
  b {c: d}
  @e f;
}
```

dart-sass prints `a b {c: d}` and then `a {@e f}`, in that order. This
compiler prints `a {@e f}` first.

The cause is visible in `visit_unknown_at_rule` in
`crates/compiler/src/evaluate/visitor.rs`: the no-body branch calls
`self.css_tree.add_stmt(stmt, self.parent)` directly, while declarations go
through `add_child`, which splits a style rule when a nested rule comes
between two of its children. The comment on that call in `visit_style` (line
4215) explains the split and why it exists; the at-rule branch predates it.
Route the childless at-rule through `add_child` as well.

## 7. `@import` is hoisted past the comments above it

`libsass-closed-issues/issue_{469,1080}`

```scss
/** comment 1 */
@import url("import-1");
```

dart-sass keeps that order. This compiler prints both `@import`s first and
then both comments. Plain CSS imports are collected and emitted ahead of
the rest of the document, and the comments written between them are left
behind.

## 8. `if()` does not take a rest argument

`libsass-closed-issues/issue_2321`,
`values/numbers/divide/slash_free/argument/macro/rest`

`if(true, b, c...)` fails with `Missing argument $if-false.` `if()` is
special-cased so that only one branch is evaluated: `visit_ternary` at
`crates/compiler/src/evaluate/visitor.rs:3809` verifies the call with
`if_arguments().verify(if_expr.0.positional.len(), ...)`, counting the
positional arguments as written. A rest argument is one entry in that
count no matter how many values it holds, so a call that supplies
`$if-false` through `c...` is rejected before anything is expanded.

## 9. A hex colour with alpha keeps its source text

`values/colors/alpha_hex/{initial_digit,initial_letter}`

`#0123` prints as `#0123` here and as `rgba(0, 17, 34, 0.2)` in dart-sass.
A colour parsed from a hex literal keeps its original spelling and prints
that back, which is right for `#abc` and wrong once there is an alpha
channel: dart-sass drops the original text for four- and eight-digit hex
and falls back to the `rgba()` form. The channel values themselves are
right -- the same fixtures read them back correctly.

## 10. A named argument does not reach a variadic parameter

`core_functions/map/remove/named`

`map.remove($map: (c: d), $key: c)` fails with `No argument named $key.`
dart-sass declares the function as `$map, $key, $keys...` -- the string is
in the 1.103.1 package verbatim -- so `$key` is a real parameter name. This
compiler treats everything after `$map` as the variadic tail and has no
name for it. Section 1's `list.join(c, d, $invalid: true)` is the
mirror-image case, where an unknown name is accepted instead of rejected;
both come from builtin signatures being positional lookups rather than
declared parameter lists.

**Closed by `fix/builtin-parameter-lists`**, together with section 1's
`list/join/error/named`. Every builtin now carries the parameter lists
dart-sass 1.103.1 declares, overloads included, in
`crates/compiler/src/builtin/signatures.rs`. The table was generated from
dart-sass's own sources at that tag. A call picks its overload the way
`BuiltInCallable.callbackFor` does and is checked with that overload's
`verify`. Named arguments then move into their declared positions, up to
the first parameter that was not passed; dart-sass also evaluates defaults
for the rest, which this does not. `map.set` and `map.merge` read which
overload matched, because counting arguments cannot tell their two forms
apart. The unknown-name error now says "parameter", as dart-sass does, for
user-defined functions too.

## 11. A private-use character is not escaped

`libsass-closed-issues/issue_1231`

```scss
div::before { content: "\e600" }
```

dart-sass prints `content: "\e600"` and no `@charset`. This compiler prints
the character itself, which makes the output non-ASCII and so adds
`@charset "UTF-8"`. dart-sass writes private-use characters back as escapes
in expanded mode. `spec/core_functions/string/split/private_use_character`
needs this too; item 20 records that.

## 12. Six singletons

| Fixture | Cause |
|---|---|
| `libsass-closed-issues/issue_143` | An unknown function's name is normalized: `file_join(...)` prints as `file-join(...)`. dart-sass keeps the underscore for a function it does not know. |
| `libsass-closed-issues/issue_1263` | `@apply ( --bar )` keeps the source's inner spacing; this compiler doubles it to `@apply (  --bar  )`. |
| `libsass-closed-issues/issue_2000` | `--&` in a declaration value prints the selector *after* extension. dart-sass resolves `&` when the declaration is evaluated, before `@extend` runs. |
| `libsass/units/simple` | `(23in/2fu) > (23cm/2fu)` errors with `Incompatible units cm/fu and in/fu.` dart-sass converts the compatible part of a complex unit. |
| `libsass/at-root/140_test_at_root_in_unknown_directive` | `@at-root` inside an unknown at-rule leaves an empty `@fblthp {}` behind. The emptied parent should be dropped. |
| `non_conformant/scss-tests/186_test_newlines_removed_from_selectors_when_compressed` | A newline between two selectors in a list is not preserved in expanded output. `ComplexSelector` already carries `line_break` for this. |

**All six are closed**, one pull request each, since they share no cause.
Several turned out to be something other than the table above guessed:

- `issue_143`, #73: `Identifier` rewrites `_` to `-` when a name is
  interned. That is right for lookup, but it lost the spelling of a call that
  finds no function and is written back as plain CSS. `FunctionCallExpr` now
  keeps the name as written, and `SassFunction::Plain` carries it.
- `issue_1263`, #74: the whitespace code was already right. An unknown
  at-rule read its value with `almost_any_value`, dart-sass's reader for
  selectors. dart-sass uses `_interpolatedDeclarationValue`, which collapses
  whitespace, and the at-rule now uses that reader.
- `issue_2000`, #75: dart-sass keeps each style rule's `originalSelector`,
  its selector before `@extend`, and reads it for `&` and for nesting. This
  compiler read the extended selector.
- `units/simple`, #78: a complex unit now converts part by part, as
  dart-sass's `_coerceOrConvertValue` does, pairing `in` with `cm` and `fu`
  with `fu`.
- `140_test_at_root_in_unknown_directive`, #76: the port of `_trimIncluded`
  returned the right parent but never removed the trimmed run. So the parent
  was copied anyway, and the original was left empty.
- `186_test_newlines_removed_from_selectors_when_compressed`, #77: the
  selector-list parser marked a line break only for a newline after a comma.
  dart-sass marks one for any change of line, including a newline before the
  comma.

## One more, outside the 39

`values/calculation/calc/operator/var/calculation` is in an area items 07
and 08 claim, and belongs to neither: `calc(1 + calc(var(--c)))` prints as
`calc(1 + var(--c))` here, dropping the parentheses dart-sass keeps when it
inlines the inner `calc()`: `calc(1 + (var(--c)))`. It is recorded here because "claimed" is measured
by area on the roadmap's front page, and an area being claimed does not
mean every fixture in it is. It is not counted in the 39 above, which would
make the front page's arithmetic disagree with itself.

**Closed by #79.** dart-sass's `_needsParentheses` keeps parentheses around
an inlined `calc()` whose text starts with a `var(` call, since the variable
may expand to anything. `needs_parens` lacked that rule.

## Testing

- Ground truth: each fixture named above at the pinned revision.
- Add regression tests to `crates/lib/tests/` with `test!` and `error!` for
  whichever cause you take, and verify every expectation against dart-sass
  1.103.1 with `npx -y sass@1.103.1`, never bare `npx sass`.
- Take one cause per branch and pull request. These are unrelated to each
  other, and a branch that fixes three of them cannot be reviewed.

## Acceptance criteria

Per cause, since this is not one work item:

- The fixtures listed for that cause pass.
- The whole-suite "Expected test to fail but it did not" count falls by the
  number of strictness fixtures the cause closes, and rises nowhere.
- No other area regresses.
