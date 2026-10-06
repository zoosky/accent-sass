# Moving the reference to dart-sass 1.105.1

This item moves the reference implementation from dart-sass 1.104.1 to
1.105.1 and the sass-spec pin from `b39c32768` to `85d5d7125`. Like
[item 25](25-baseline-before-dart-sass-1-104.md) it is listed under
"Reference moves" in the [README](README.md). Unlike item 25, the move lands
before the release is followed: 1.105.0's feature is a new value type, and
this document records the baseline, what the move changed, and the plan for
the rest.

## Baseline before the move

Measured 2026-10-06 on master `2b432f1e` (#113 merged), macOS, release
build, with CI's flags (`--trim-errors --ignore-warning-diffs
--ignore-error-diffs`):

| sass-spec revision | Runs | Failures |
|---|---|---|
| `b39c32768`, the old pin | 14,266 | 111 |
| `85d5d7125`, upstream main | 14,355 | 148 |

The move adds 89 runs and 37 failures, all under `core_functions/meta`, and
fixes none. Every fixture failing at the old pin fails at the new one, and
nothing outside `core_functions/meta` is new. CI reads two higher than macOS,
as item 25 describes.

## What 1.105.0 and 1.105.1 change

Nine commits between the tags
(`gh api repos/sass/dart-sass/compare/1.104.1...1.105.1`), five in the
library.

1. **First-class modules** (`850d57ef9`, sass/dart-sass#2861; the language
   change is sass/sass#4247, flushed to `spec/built-in-modules/meta.md` and
   `spec/types/module.md`). `meta.load($url, $with: null)` loads a module as
   `@use` would and returns it as a value without emitting its CSS.
   `meta.get-module($module)` returns the module behind a namespace. The
   mixin `meta.css($module)` emits a module's CSS where it is included, and
   `load-css($url, $with)` is redefined as `css(load($url, $with))`. Every
   `$module` parameter -- `function-exists`, `mixin-exists`,
   `global-variable-exists`, `get-function`, `get-mixin`,
   `module-functions`, `module-mixins`, `module-variables` -- accepts a
   module value as well as a string. The value: `type-of` is `module`,
   `==` is identity, `not` is allowed, every other operator is
   `Undefined operation "get-module("x") + get-module("x")".`, `inspect`
   prints `get-module("ns")` with the namespace the module's URL would get
   under `@use` (empty parentheses when that is not an identifier), and any
   other serialization is `get-module("x") isn't a valid CSS value.`.
2. **`@extend` across media queries** (`61ab7025b`, sass/dart-sass#2869).
   The `From line 1, column 1 of input.scss:` prefix is gone from both
   errors, and each carries secondary spans: `extension @media`,
   `extended selector @media`, `extended selector`, and for the merged case
   `first extension`, `second extension` and their `@media`s. This compiler
   never printed the prefix, so the messages already match; the spans are
   outside the parity rule.
3. **Span boundaries** (`2ac451571`, `706cfbf3c`). Selector and media-query
   spans lose trailing whitespace, `CssMediaQuery` gains a span, and the
   interpolation map stops at the first non-whitespace character rather than
   a `}`. Output-neutral: the bogus-combinator warning for `.a >   \n{b: c}`
   prints identically on 1.104.1, 1.105.1 and this compiler.
4. **Wording.** `There is no module with the namespace "x".` loses "the",
   and a `$module` that is not a string is now
   `$module: 1 is neither a string nor a module reference.`.
5. **JS API and embedded protocol** gain `SassModule` and
   `Value.assertModule()`. Not applicable: the browser binding exposes no
   value objects.

Probed against `npx -y sass@1.105.1` where the fixtures leave a question
open:

- Built-in modules are singletons. `meta.get-module(math)` equals a second
  alias of `sass:math`, equals `meta.load("sass:math")`, and prints
  `get-module("math")`.
- A module loaded by `meta.load` and later `@use`d prints its CSS once, at
  the `@use`, and the two values are equal.
- `meta.css` prints the module's upstream too: with `shared` already
  `@use`d, `meta.css(meta.load("withdep"))` prints `shared` a second time.
- Extension errors inside a loaded module surface at `meta.css`, not at
  `meta.load` (`load/no_error/extend`).
- A module is truthy; in a list or map `inspect` prints it inline.

## What this item changed

- The `bootstrap` and `frameworks` jobs install 1.105.1; the `sass-spec`
  submodule is pinned to `85d5d7125`; `README.md`, `CLAUDE.md`, the
  documentation site and this directory's README name 1.105.1.
- The two messages in item 4 above, in `builtin/modules/mod.rs`,
  `builtin/modules/meta.rs` and `builtin/functions/meta.rs`, with the ten
  test expectations that quoted them. The three `module-*` functions share
  one `module_namespace` helper that produces the new error.
- **A crash found by the new pin.** `directives/extend/error/cross_media/merged`
  -- the same selector extended from two different media queries, both
  `!optional` -- panicked on an `unwrap` in `add_extension` where the merge
  refused the pair, instead of reporting
  `You may not @extend the same selector from within different media
  queries.`. The fixture passed under `--ignore-error-diffs`, which counts a
  crash as an error. The error is now returned. Regression test in
  `extend.rs`.

The move changes no tally: the wording is hidden by `--ignore-error-diffs`
and the crash already counted as an error.

## Module values

The second pull request implements the feature. Measured the same way as the
baseline, on macOS against the `chore/dart-sass-1.105.1` revision this branch
started from:

| Revision | Runs | Failures |
|---|---|---|
| before, `9bf29cbd` | 14,355 | 148 |
| after | 14,355 | 100 |

48 fixtures pass that did not, and none fails that passed. The 37 listed
above; the six `*/error/dash_sensitive` and `*/error/module/dash_sensitive`
cases of `module-functions`, `module-mixins`, `module-variables`,
`function-exists`, `global-variable-exists` and `mixin-exists`; four of the
six `load_css` failures the plan flagged -- `extend/in_other/{before,
after}`, `extend/shared_cssless_midstream` and
`twice/load_css/different_extend` -- and `load_css/plain_css/
through_other_mixin`. `twice/use/different_extend` stays open; the first
deviation below is why. The 48 new tests in
`crates/lib/tests/module-values.rs` were each checked against the 1.105.1
release binary, and the one that asserts that deviation says so.

What landed, against the plan:

- `Value::ModuleRef(SassModule)`, an `Arc<RefCell<Module>>` compared by
  pointer. `Module` carries its URL now -- the cache key for a stylesheet,
  the `sass:` URL for a built-in -- and derives the namespace `inspect`
  prints from it the way dart-sass's `defaultNamespace` does. The unary and
  binary operators report `Undefined operation`, with the one exception
  dart-sass has: `"string" + module` serializes the module and fails as `isn't
  a valid CSS value`.
- The built-in modules are cached on the `Visitor`, one instance per URL.
- One `module_from_value` helper serves every `$module` parameter. A string
  is looked up as the `@use` rule wrote it, which is what made the
  `dash_sensitive` fixtures pass: `Modules` keeps the written namespaces
  beside its `Identifier` map. `a_b.$x` still reaches `@use ... as a-b`,
  because the parser hands the evaluator an `Identifier`; that half is a
  parser change and stays open.
- `meta.load` executes the module with the same `execute` as `@use`, in a
  mode of its own. The module and whatever it loads share a fresh extension
  store -- as an `@import` context does, so a module's `@extend` applies to
  the modules it loaded at once -- and the module does not join the loader's
  upstream, so its extensions reach nothing outside the load. When execution
  ends, the top-level statements it emitted are taken out of the tree
  (`CssTree::remove_subtree`); the record `execute` already keeps for
  `@import` is what remains. The module goes on a pending list, and so does
  every module first executed during the load.
- `meta.css` replays the record at the include, as `@import` of a cached
  module does. A copy of a module `meta.load` executed registers its
  selectors afresh with the store in force at the include, so an `@extend`
  there applies to the copy and the module's own `@extend` rules do not leak;
  a copy of a module `@use` executed keeps sharing its selector handles, so
  the extensions `apply_module_extensions` resolves at the end reach it. The
  pending-list check for a missing `@extend` target runs when the CSS is
  emitted, not when the module is loaded, which is what `load/no_error/extend`
  and `load_css/error/extend` require between them.
- A `@use` or `@forward` of a pending module emits its record at that point
  and takes it off the list: the module's own statements and those of loaded
  modules still pending, each once, which is why recorded statements now
  carry the module that emitted them. Probed: `@use "mid"` (which loads
  `other`) then `@use "other"` prints `mid`'s CSS, then `other`'s, then the
  root's, as 1.105.1 does.
- `meta.load-css` is `css(load())`. Its URL resolves against the file the
  call is written in, which dart-sass passes as `baseUrl`; that is what
  `through_other_mixin` tests.

Deviations that remain, none of which a fixture counts:

- A copy of a module that `@use` executed takes the extensions of the whole
  module graph, since its selector handles are shared with the original.
  dart-sass clones the module's own graph and applies that graph's
  extensions to the clone, plus the include site's. The two agree unless a
  module outside the copied module's graph extends into it, which is
  `load_css/twice/use/different_extend`: the `@use` copy and the `load-css`
  copy both print `b, a`. Registering the copy's selectors afresh instead
  would pass that fixture and lose the module's own extensions, which
  `apply_module_extensions` resolves only at the end; a faithful clone needs
  that walk run over the copied module's graph alone, and is left open.
  Registering afresh everywhere also broke
  `directives/use/extend/scope/use_and_import_into_diamond_extend`, which
  needs the `@import` copy to share.
- A module `meta.load` executed and `@use`d later does not report a missing
  `@extend` target in a module it loaded along the way, because the shared
  store belongs to the outermost load. dart-sass reports it once the module
  is in the root's graph.
- `Module loop: this module is already being loaded.` does not name the file.
  The `@use` message has the same gap.
- `(a b) * 1` prints `Undefined operation "a b * 1"`; dart-sass
  parenthesizes the list. This predates the module value and applies to any
  operand.

The multi-span rendering of item 2 above was not attempted.
