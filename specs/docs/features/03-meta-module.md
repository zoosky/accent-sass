# sass:meta mixin reflection and load-css strictness

Unlocks 152 sass-spec tests under `spec/core_functions/meta`:
`load_css` 53, `get_mixin` 28, `module_mixins` 13, `apply` 9,
`accepts_content` 6, plus about 40 across the `*-exists`,
`module-functions`/`module-variables`, `type-of` and `inspect` families.

This item has two independent halves; they can ship as separate pull
requests.

## Half 1: first-class mixin values

### Current behavior

`crates/compiler/src/builtin/modules/meta.rs` registers the module's
functions at lines 168-184. `get-mixin`, `module-mixins`, `apply` and
`accepts-content` are absent, and accent-sass has no mixin value type — the
`Value` enum in `crates/compiler/src/value/mod.rs` has `SassFunction`
support but nothing equivalent for mixins.

### Reference behavior

dart-sass provides:

- `meta.get-mixin($name, $module: null)` returns a first-class mixin.
- `meta.apply($mixin, $args...)` includes it, forwarding a content
  block when the caller has one.
- `meta.module-mixins($module)` returns a map from mixin names to
  first-class mixins.
- `meta.accepts-content($mixin)` reports whether a mixin can take a
  `@content` block.
- `meta.type-of()` returns `mixin` for such values, and `meta.inspect()`
  serializes them as `get-mixin("name")`.

### Implementation instructions

1. Add a `Value` variant for mixins mirroring `SassFunction`, wrapping
   the existing mixin representation used by `@include` resolution in
   `crates/compiler/src/evaluate/visitor.rs`. Update the exhaustive
   matches the compiler forces you to visit: `type-of`, `inspect`,
   serialization (a mixin value in plain CSS output is an error, like a
   function value), and equality.
2. Implement the four functions in
   `crates/compiler/src/builtin/modules/meta.rs`, following the
   existing `get-function`/`module-functions`/`call` implementations as
   the pattern. `apply` is the involved one: it must run through the
   same code path as `@include`, including `@content` forwarding —
   dart-sass only allows `meta.apply` in a statement position where a
   content block is syntactically possible.
3. `accepts-content` reads whether the mixin body references
   `@content`; the parser already tracks this for `content-exists`.

## Half 2: load-css strictness

### Current behavior

Almost all 53 `load_css` failures are "expected test to fail but it did
not": accent-sass accepts configurations and load patterns dart-sass rejects.

### Reference behavior

`meta.load-css($url, $with: null)` validates like `@use ... with`:

- Every key in `$with` must be a variable the loaded stylesheet declares
  with `!default` (`error/with/undefined`, `error/with/not_default`).
- A module already loaded without configuration cannot be re-loaded
  with one, and conflicting configurations are errors.
- Nested loads and loads of stylesheets that use `@extend` across the
  boundary have their own restrictions (`error/load/nested/*`,
  `error/from_other/extend`).

### Implementation instructions

Locate the `load-css` implementation in
`crates/compiler/src/builtin/modules/meta.rs` (registered at line 184)
and add the validations above, reusing the checks the `@use ... with`
path performs so the two stay consistent. Take each expected error, and
its message, from the `.hrx` files under
`spec/core_functions/meta/load_css/error/`.

## Testing

- Ground truth: `spec/core_functions/meta/`. Scope the spec run with
  that path (see [README.md](README.md)).
- Add `test!`/`error!` cases to `crates/lib/tests/` for each new
  function including `apply` with and without content, and for each
  `load_css` error class. Verify expectations against the dart-sass
  1.103.1 binary first.

## Acceptance criteria

- `spec/core_functions/meta` passes apart from error-span differences.
- `cargo test --features=macro` shows no regressions in the existing
  meta and module tests.
