# Repeated comments before `@use` and `@forward`

**Landed. Bulma compiles byte-identically to dart-sass 1.104.0.** The
`frameworks` corpus reported five differing lines for Bulma for as long as the
job has existed, and all five were one comment. This item ports the rule that
produces them, at the cost of two sass-spec fixtures: the suite goes from 111
failures to 113.

That trade needs stating plainly, because the rule being ported is a bug that
dart-sass has already fixed. The owner chose on 2026-09-12 to match the
reference release exactly rather than to anticipate the next one.

## What Bulma shows

`sass/form/_index.scss` opens with `/* Bulma Form */`, once, above a
`@charset` and six `@forward` rules. dart-sass 1.104.0 prints that comment
six times: before `shared`, and again before each of the five form modules
that follow, every one of which also carries `@use "shared"`. This compiler
printed it once.

Measured 2026-09-11 against Bulma 1.0.4 and the native dart-sass 1.104.0
binary, using `.github/scripts/frameworks.sh`:

| Input | Before | After |
|---|---:|---:|
| `bulma/bulma.scss` | 5 | 0 |
| The modular example in Bulma's [customize docs](https://bulma.io/documentation/customize/with-modular-sass/) | 5 | 0 |
| `@use "bulma/sass/form"` alone | 5 | 0 |

The modular example is the second route Bulma documents: `@use ... with (...)`
over individual module paths rather than one entry file. Its `with (...)`
overrides, its `@forward` list and its trailing `@import url(...)` all matched
already; the same five comments were its whole difference.

## The rule

dart-sass records the loud comments written above a `@use` or `@forward` and
keys them by the module that load resolves to (`_preModuleComments` in
`lib/src/visitor/evaluate.dart`). When it assembles the output it writes a
module's recorded comments before each module that module depends on.

Two details make the comment repeat, and both are ported here:

1. **The map is inherited.** dart-sass saves and restores it around each
   module it executes but never resets it on entry, so a module executed
   while its loader holds a map shares that same map object. Bulma's five
   form modules are executed from `_index.scss` after it registered the
   comment for `shared`, so each of them carries the same registration.
2. **Each load emits.** The comments are written again before every module
   whose dependency list names the registered module, even one already
   written out, and once per rule that loads it -- a file that both `@use`s
   and `@forward`s the same module gets two copies.

This compiler emits CSS as modules execute rather than combining it
afterwards, so the first copy is simply the comment where it was written, and
`Visitor::emit_pre_module_comments` writes the repeats at the load site.
`Visitor::pending_top_comments` stands in for the module root dart-sass reads,
and `Visitor::modules_with_css` for its `transitivelyContainsCss`.

A repeat is written only at the top level of the document. dart-sass writes
these comments while combining modules, which is always top level, whereas a
load site here can sit anywhere: a `@use` inside a file `@import`ed within a
style rule loads at that rule's position. A code review caught the first
version writing the comment into that rule, which dart-sass never does. A
top-level `@import` of such a file still repeats, as the reference does.

## What it costs

`spec/directives/use/comment/loud/repeated_use` and
`spec/directives/use/comment/loud/skip` now fail. Both are headed "Regression
test for sass/dart-sass#2851", and both encode the *fixed* behaviour: one
copy.

The dart-sass side, verified rather than inferred:

- [sass/dart-sass#2851](https://github.com/sass/dart-sass/issues/2851),
  "Leading loud comments are duplicated when an entrypoint `@use`s a module
  its other dependencies also `@use`", closed 2026-09-08. Filed as a bug.
- [sass/dart-sass#2854](https://github.com/sass/dart-sass/pull/2854),
  "Avoid duplicating loud comments that appear before `@use`", merged the same
  day. It resets `_preModuleComments` on module entry and emits each upstream's
  comments at most once per module -- exactly the two details above.
- dart-sass `main` carries it under an unreleased **1.104.1**: "Fix a bug where
  loud comments before `@use` rules could be emitted multiple times under
  certain circumstances." The newest release is still 1.104.0.
- The pinned sass-spec revision, `b39c32768`, **is** the commit "Add regression
  tests for sass/dart-sass#2851 (#2166)". The pin already expects the fix.

The native 1.104.0 binary fails both fixtures, which is how the divergence was
found: this compiler passed them before this item by printing one copy, the
same as fixed dart-sass.

One case still differs from 1.104.0 even with the port. In `repeated_use` the
reference writes the repeat before the comment that follows the second `@use`,
because it places every repeat while combining modules, after all of them have
run; this compiler writes it at the load site, so a comment written between the
two rules comes first. That fixture fails either way, and closing the gap would
mean deferring module CSS until the end, which is a different architecture.

## When 1.104.1 lands

**Revert this item, do not extend it.** Moving the reference to 1.104.1 makes
the repeats wrong, and reverting restores what master did before: one copy,
which is what the two fixtures expect. The check afterwards is the same as the
one that justified the port -- Bulma, the modular example and the form module
compiled against the new binary, expecting zero differing lines, and the suite
back to 111 rather than 113.

## Out of scope, found while measuring

Two module-loading differences predate this item and are unchanged by it,
recorded here because the probes that found them are easy to lose:

- `@import`ing a file that `@use`s a module emits that module's CSS twice.
- `meta.load-css` of a module whose upstream is already loaded emits the
  upstream's CSS twice and places the module's own leading comment before it
  rather than after.

Neither has a sass-spec fixture failing against it today.
