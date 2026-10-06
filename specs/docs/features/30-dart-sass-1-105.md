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

## What is left: module values

The 37 failures: `css/*` 7, `get_module/*` 12, `load/{live, with,
shares_state, no_error/*, error/with/private/*}` 8, the
`different_module/*/module_value` cases of the five `$module` functions 8,
and `module_{functions,mixins,variables}/module_value` 3.

- `Value::ModuleRef` holding the `Arc<RefCell<Module>>` and its canonical
  URL, which `inspect` needs for the namespace. `kind()` is `module`,
  equality is `Arc::ptr_eq`, arithmetic and unary operators error, the
  serializer prints `get-module("ns")` only when inspecting.
- A built-in module cache on `Visitor`. `load_module` constructs a fresh
  `declare_module_*()` on every `@use "sass:x"`, so two namespaces for one
  built-in are two values today; dart-sass has one instance per URL.
- One `get_module` helper -- a string looks the namespace up, a module value
  is itself, anything else is the new error -- replacing `module_namespace`
  and the four `match` blocks in `builtin/functions/meta.rs`.
- `meta.load`: the hard part. This compiler emits CSS into one tree in load
  order and `execute` records a module's statements for `replay_module_css`;
  dart-sass keeps a stylesheet per module and composes at the end. `load`
  must execute the module, keep its record and leave nothing in the tree,
  and a later `@use` or `@forward` of a module first loaded this way must
  emit the record at that point. `css` replays the record at the include in
  import context, upstream included, which is what `load_css_module` does;
  `load-css` becomes `css(load())`.
- Risks: `load/no_error/serialize` expects a map in a declaration not to
  error at `load`, so check whether a declaration is serialized as it is
  emitted. The six pre-existing `load_css/extend/*` and `load_css/twice/*`
  failures sit in the same replay path and may move with it.
- Optional, no tally impact: the multi-span rendering of item 2.

Verify every new expectation against `npx -y sass@1.105.1`; the subset
`core_functions/meta directives/extend values/modules` runs in two seconds.
