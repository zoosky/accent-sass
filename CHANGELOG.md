# Changelog

All notable changes to this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
While the major version is `0`, a breaking change bumps the minor version.

`accent-sass` is a fork of [`connorskees/grass`](https://github.com/connorskees/grass).
Published to crates.io as `accent-sass`, `accent_sass_compiler` and
`accent-sass-macro`, which share a version and are released together. Entries
at `0.13.4` and below are upstream's and are kept for lineage.

## [Unreleased]

### Added

- a C ABI for WebAssembly hosts, behind the `wasi-exports` feature.
  `accent_sass_alloc`, `accent_sass_dealloc`, `accent_sass_compile_string`,
  `accent_sass_compile_path` and `accent_sass_result_free` turn the
  `wasm32-wasip1` cdylib into a reactor a plugin host instantiates once and
  calls many times, instead of paying process startup per stylesheet. Strings
  cross as UTF-8 in linear memory and a result is three 32-bit words: status,
  pointer, length. Built with the workspace's `small` profile, the module is
  1.31 MiB against 2.42 MiB under `release`
- options for that C ABI. `accent_sass_options_new` returns an opaque handle a
  host fills in and passes to `accent_sass_compile_string_with_options` or
  `accent_sass_compile_path_with_options`, which covers output style, entry
  point syntax, load paths, `charset`, `alertAscii` and `quiet`. The setters
  are named after dart-sass's JavaScript API so this ABI and the browser
  binding do not drift apart; a value the ABI does not define is refused rather
  than rounded to a default. `accent_sass_compile_string` and
  `accent_sass_compile_path` are unchanged and still compile with defaults
- a JavaScript API for the browser build. `compileString(source, options)` and
  `compile(path, options)` join `from_string`, which keeps working. The options
  are `style`, `syntax`, `loadPaths`, `files`, `url`, `charset`, `alertAscii`,
  `quiet` and `logger`, named after dart-sass's JavaScript API wherever it has
  a name for the same knob; a failed compile throws an `Error` carrying
  `message`, `formatted`, `file`, `line` and `column`, and `index.d.ts`
  declares the surface with real types. Before this the package exported one
  function with `StdFs` behind it, so every `@use` failed at run time with
  `Can't find stylesheet to import.` and a page could compile a single
  stylesheet and nothing else
- `MemoryFs` and `from_string_with_file_name` are public. The browser binding
  resolves imports through `MemoryFs`, and making it a public type rather than
  something private to the binding is what lets `cargo test` reach it: the
  bindings themselves exist only on `wasm32-unknown-unknown`. It also stands on
  its own for a host that already holds its stylesheets, such as a CMS
  compiling a theme out of a database
- `--check` on the command line, which compiles and writes nothing. Given an
  output file it compares the CSS it produced against what that file already
  holds, so a job can ask whether committed CSS is still what the Sass produces
  without a build step, a temporary file or a `diff`. It exits `3` on a stale
  or missing output file and `1` on a stylesheet that does not compile, which
  keeps "your CSS is out of date" and "your Sass is broken" apart in a log;
  `2` is still clap's usage error. Writing nothing is the point of the mode:
  it reports whether a build is current without being able to change the answer
- `--indented` works, and is no longer hidden. It reads the entry point as the
  indented syntax whatever the file is called. That matters most for `--stdin`,
  which is the one input route with no extension to infer a syntax from, and
  the route the flag exists to serve

### Changed

- the reference implementation is dart-sass 1.104.0, and both of its changes
  are followed. Exact negative zero prints as `-0` rather than `0`, so it
  keeps its sign when used in a CSS calculation; `math.round`, `math.ceil`,
  `math.floor` and the calculation `round()` still return `0`, because
  dart-sass rounds to an integer. A colour channel that is `NaN` or negative
  zero becomes `0`, and so does a hue that is infinite:
  `hsl(math.div(1, 0), 50%, 50%)` is now `hsl(0, 50%, 50%)` rather than
  `hsl(calc(NaN), 50%, 50%)`. As in dart-sass, `color.change()` on an rgb
  colour keeps such a channel until the colour is converted to another space,
  and `red()`, `green()` and `blue()` read a `-0` channel back as `0`
- **Breaking: `@extend` across a media query is an error.** An `@extend`
  written inside `@media` may only extend a selector in the identical query;
  extending across queries would need the extended rule to exist in a context
  it was never written for. It extended silently before, which is worse than
  refusing: the check was commented out and the store never recorded a
  selector's media context at all, so every selector looked top-level.
  `!optional` does not silence it, since it says the extension need not match
  anything rather than that a wrong match is allowed
- **Breaking: builtin calls are checked against dart-sass's parameter lists.**
  Builtins read their arguments by position with no declared parameter list, so
  an unknown name could not be rejected and a real one could not be found:
  `map.remove($key: 1)` and `list.join(c, d, $invalid: true)` compiled. All 196
  registrations now carry the parameter list dart-sass 1.103.1 declares,
  overloads included, and a call picks its overload, is verified against it,
  and has its named arguments bound to their declared positions. The
  unknown-name error says "parameter", as dart-sass does, for user-defined
  functions too
- **Breaking: seven inputs dart-sass refuses are rejected.** A style rule
  inside a keyframe block, `@mixin --a` and `@include --a` (plain CSS is
  getting mixins of its own and `--` names are reserved for them), a custom
  property nested beneath another declaration, and an unbalanced bracket in a
  selector. The bracket tracking also closes ten fixtures in the indented
  syntax, where a newline inside a bracket no longer ends the selector
- **Breaking: a selector whose combinators cannot match is omitted.** Two
  combinators in a row, more than one leading combinator, or a trailing one
  make a selector bogus, so a rule left with nothing visible is not printed and
  a useless selector can no longer act as an extender. The rules are ported
  from the dart-sass 1.104.0 source, which hides such selectors when writing
  CSS rather than in the evaluator, so `selector.parse(":is(.a > + .b)")` still
  keeps its argument. `selector.replace()` raises dart-sass's "components may
  not be empty" where it used to panic
- a loud comment written above `@use` or `@forward` is repeated before every
  module that loads it, as dart-sass 1.104.0 does. This is what Bulma's five
  differing lines in the framework corpus were; Bulma now compiles
  byte-identically to the reference. The repeats are written only at the top
  level, because dart-sass writes them while combining modules. dart-sass
  itself calls this a bug and fixed it for the unreleased 1.104.1, so it costs
  two sass-spec fixtures and is to be reverted when the reference moves

### Fixed

- ten command-line flags consumed the argument after them. `--indented`,
  `--update`, `--no-error-css`, `--no-source-map`, `--embed-sources`,
  `--embed-source-map`, `--watch`, `--poll`, `--no-stop-on-error` and
  `-i`/`--interactive` were each declared with no action, and clap 4 defaults
  to an action that takes a value, so `accent-sass --watch style.scss` read
  `style.scss` as the value of `--watch` and then failed with `the following
  required arguments were not provided: <INPUT>`. A script written against
  dart-sass's command line failed on the argument parse rather than on the
  missing feature, and the error named the input file rather than the flag
  that ate it. All ten are switches now. Six of them -- `--update`, `--watch`,
  `--poll`, `-i`/`--interactive`, `--embed-sources` and `--embed-source-map`
  -- say they are ignored rather than passing in silence, because each would
  change what the program does if it worked; `--no-error-css`,
  `--no-source-map` and `--no-stop-on-error` already describe what this binary
  does and stay quiet
- `--stdin` with a named file writes to it. The lone positional was read as the
  input, so `accent-sass --stdin out.css` tried to compile `out.css` and failed
  with `No such file or directory` rather than writing the CSS it read from
  standard input, which is what dart-sass does. A second positional alongside
  `--stdin` is refused now rather than silently ignored. **This overwrites the
  named file**, which is what dart-sass does and what the old behaviour did not:
  `cat x.scss | accent-sass --stdin style.scss` used to compile `style.scss` and
  ignore standard input, and now truncates it and writes the compiled CSS there.
  Redirect to the destination instead if the positional was standing in for the
  input
- a failed compile truncated the output file. `accent-sass style.scss app.css`
  opened `app.css` with `truncate(true)` before running the compile, so a
  stylesheet that stopped compiling left an empty `app.css` behind. It
  destroyed the last good CSS at the moment you most want to keep it, and a
  missing input file did the same. The output file is opened only after the
  compile succeeds
- `@warn` and `@debug` report a string message as its text rather than with
  its quotes, as dart-sass does: `@warn "careful"` says `careful`, not
  `"careful"`. Only the outermost value is unwrapped, so a string inside a
  list still prints quoted, and every other value is reported as before. The
  command line and the `Logger` trait are both affected, because the quoting
  happened in the evaluator
- `%` and the calculation `mod()` with an infinite divisor no longer count
  positive zero as negative. `0 % infinity` is `0` rather than `NaN`, and
  `0 % -infinity` is `NaN` rather than `0`, as dart-sass gives
- unit names are case-sensitive, as they are in dart-sass. Only the canonical
  spelling names a known unit, so `1Q` now prints as `1Q` rather than `1q`,
  `math.div(1kHz, 1hz)` no longer simplifies, and `1PX + 1px` is an error
  about incompatible units. The one place case is ignored is the check that
  decides whether two units could ever be compatible, which is what makes
  `calc(1Q + 1deg)` an error and `calc(1Q + 1mm)` legal
- calculation errors say what dart-sass says. An *operation* a calculation
  cannot perform is now distinguished from an *expression* it cannot hold and
  reported at the operator, a rest argument is named as one, a bare list is
  parenthesised in `Value (1 2 3) can't be used in a calculation.`, and the
  parser reports `expected ")".`, `Expected expression.` or
  `Expected identifier.` by position instead of the stale
  `Expected number, variable, function, or calculation.`, which dart-sass no
  longer emits anywhere. `calc("a")`, `calc(())`, `calc(#fff)`, `calc(/ 1px)`
  and `calc(1px % 2px)` are rejected as the values and operators they are
  rather than as parse failures, and a keyword argument is named as one:
  `sqrt($x: 1)` reports `Keyword arguments can't be used with calculations.`
- a stylesheet may use `&` at the top level. CSS nesting made `&` meaningful to
  the browser, so dart-sass passes a top-level one through as text rather than
  raising "Top-level selectors may not contain the parent selector", which it
  no longer emits anywhere. A parent selector carrying a suffix is still an
  error, with dart-sass's message, and the check looks inside a
  pseudo-selector's argument, because `:is(&--x)` is an error too
- five defects in `sass:math`. A fuzzy `is_zero()` stood in for an exact zero
  in `asin`, `atan` and `log`, so `math.log(0.000000000001)` returned
  `calc(-infinity)` where its value is `-27.63`, and `acos` had the same defect
  through `is_one()`. `clamp()` had no case for `$min >= $max` and returned
  `$number` on a tie, so `clamp(1, 2, 0)` gave `0` rather than `1`; it also
  compared `$min` against `$number` and `$min` against `$max`, which let
  `clamp(0, 1, 2px)` through. `$min-number` was the smallest normal double
  where Dart's `double.minPositive` is the smallest subnormal, 16 orders of
  magnitude apart. `math.unit()` bracketed a denominator only when the
  numerator was empty, so `math.unit(1px * 1em / 1rad / 1s)` printed
  `px*em/rad*s`
- three defects in `string.split()`. Rust's `str::split("")` matches at every
  character boundary, so an empty separator returned an empty piece at each
  end; it now splits into code points, and `$limit` is ignored on that path as
  in dart-sass. An empty string yields an empty list rather than one empty
  piece, and each piece keeps the input's quotedness rather than being quoted
  regardless
- a map, and an arglist, is the list of elements each is. `Value::separator`
  reported a comma for every map, `list.join` hard-coded one, and
  `list.append` turned a map or an arglist into a one-element list holding a
  container -- which then refused to serialize with "isn't a valid CSS value".
  An empty map, an empty arglist and an empty list are now equal in every
  direction, `!=` included, while an empty arglist is still not equal to `()`,
  the one pairing where the separator counts
- an arglist takes the separator of the list splatted into it, a comma only
  when nothing decided one, so `@include foo(1, 2, $list...)` with a
  space-separated `$list` prints `2 3 4 5` rather than `2, 3, 4, 5`. The
  separator was lost in four places, and equality compared an arglist to any
  comma-separated list whatever its own separator. An arglist nested inside
  another list keeps the parentheses that keep the two separators apart
- a childless at-rule keeps its place in the rule. It went into the tree
  without the check that splits a style rule when a nested rule comes between
  two of its children, so `a { b {c: d} @e f; }` emitted `a {@e f}` first;
  source order decides the cascade
- `if()` expands a rest argument before verifying the call, so
  `if(true, b, c...)` no longer fails with `Missing argument $if-false.` The
  arguments written in the call stay unevaluated, so the branch not taken
  still never runs
- a hex colour written with an alpha channel drops its source spelling:
  `#0123` prints as `rgba(0, 17, 34, 0.2)`, and an opaque `#abcf` as
  `#aabbcc`. Four- and eight-digit hex is not yet well supported in browsers,
  so dart-sass lets the serializer infer the form. The trigger is that an alpha
  channel was written, not that it is transparent
- a private-use character is written back as an escape in expanded mode, as
  dart-sass does, so `content: "\e600"` no longer emits the character itself
  and no longer adds a `@charset "UTF-8"` the reference does not emit.
  Compressed mode writes the character. Escaping is a serialization concern, so
  such a character is still one character to `string.split()`
- the comments above a plain CSS `@import` stay with it. Imports were collected
  and emitted ahead of the whole document, leaving the comments written between
  them behind. An `@import` written after a rule still moves up, but to the end
  of the import block rather than to the top of the document
- `@extend` produces dart-sass's selector set. `ComplexSelector::is_super_selector`
  is now a port of dart-sass 1.103.1's `complexIsSuperselector`, whose old
  check rejected `.d > .e` as a superselector of `.b .d > .e`;
  `extend_complex` returns a single-selector path's selector itself, so it
  keeps its identity; and `add_extension` reads the target's
  extensions-by-extender list after adding the new extenders, so an extension
  loop closes
- nested `@media` queries merge and are kept the way dart-sass does.
  `MediaQuery::merge` compared the modifiers where dart-sass compares the
  types in the branch where exactly one query is negated, so `not screen`
  inside `screen` came out as `screen`; and the following-sibling test counted
  invisible siblings, so an empty bubbled `@media` split its parent
- a plain CSS function keeps its spelling. Interning rewrites `_` to `-`, which
  is right for looking a function up and loses the spelling of a call that
  finds nothing and is written back as plain CSS, so `file_join(...)` printed
  as `file-join(...)`. The spelling survives `meta.call` with a string,
  `get-function($css: true)` and inspecting the reference
- whitespace inside an unknown at-rule's value is collapsed, so
  `@apply  (  --bar  );` prints `@apply ( --bar );`. The value was read with
  dart-sass's reader for selectors, which does not collapse, rather than its
  reader for declaration values, which does
- a style rule's selector is read before `@extend` rewrites it, for `&` in
  SassScript and for resolving nested rules, so `--&` no longer prints the
  extender that `@extend` added. dart-sass keeps each rule's original selector
  beside the live one for exactly this; `@extend` still reads the live one
- `@at-root` inside an unknown at-rule no longer leaves an empty copy of it
  beside the one holding the body. dart-sass's `_trimIncluded` removes the
  trailing run of included parents that already enclose the rule; this port
  returned the innermost node but never removed the run
- a selector list keeps its line break wherever the newline falls, not only
  when the whitespace after a comma holds one, so `a\n, b` nested under `z &`
  prints on two lines as dart-sass does. The check searches tokens rather than
  building a string at every comma, which had made a long single-line list
  parse in quadratic time: 40,000 selectors took 8 s and now take 0.04 s
- units convert part by part when either side is complex, so
  `(23in/2fu) > (23cm/2fu)` is `true` rather than
  `Incompatible units cm/fu and in/fu.` Each unit in the target's numerator
  pairs with the first convertible unit left in the source's numerator, and
  likewise for denominators; comparison, equality, `+`, `-`, `%`, `clamp` and
  `math.compatible` all go through it
- an inlined nested `calc()` keeps parentheses around a leading `var()` call,
  so `calc(1 + calc(var(--c)))` prints `calc(1 + (var(--c)))`. A variable may
  expand to anything, which is why dart-sass keeps them
- a nested `@font-face` does not get the enclosing selector. A nested at-rule
  normally gets a copy of the style rule so that declarations written inside it
  have somewhere to go; `@font-face` is the exception, because its descriptors
  belong to the at-rule itself. The comparison is on the plain name, so
  `@-moz-font-face` and `@FONT-FACE` still bubble

## [0.15.0] - 2026-09-08

Twenty-two merged pull requests since `0.14.0`: plain CSS parity (nesting, the
`@function` rule, `if()`), the special CSS functions, the indented syntax's
newline rules, dart-sass's per-module `@extend` model, and the calculation
work.

Against the pinned sass-spec revision (`4a9eea66`) this takes the suite to
13,926 of 14,218 passing, measured on macOS 2026-09-08; the Linux CI runner
reports two fewer, an offset that is stable across commits.

### Added

- the CSS `if()` function in plain CSS files. `if()` with CSS-style conditions
  landed for Sass earlier; the plain CSS parser did not reach that code path
  and stopped at the first `:`. A `sass()` condition, which is settled at
  compile time, is rejected there
- the plain CSS `@function` rule. `@function` whose name begins with `--`
  declares a CSS custom function rather than a Sass one, so Sass passes the
  rule through untouched -- parameters, `returns` clause, nested rules and all.
  Its `result` descriptor is parsed like a custom property, taking its value
  verbatim instead of as SassScript, and `--a()` at a call site always names a
  CSS custom function even where identifier normalisation would otherwise reach
  `@function __a()`
- CSS nesting in plain CSS files. A `.css` file may nest style rules; Sass no
  longer rejects them and no longer resolves them, since CSS nesting is the
  browser's job. The rule keeps its own selector and stays nested, `&` is
  written out unresolved, and at-rules stop bubbling out of a rule once nesting
  has been passed through
- a bare `%` is a value. `a {b: %}` and `$x: %` were parse errors; dart-sass
  takes a lone `%` as an unquoted string, and reads `%` as the modulo operator
  only when an operand follows it. A `%` value is rejected once the expression
  has consumed a comma, which is what dart-sass does
- `attr()` and the CSS `if()` count as special variable strings, so the colour
  functions leave them unevaluated instead of rejecting them: `rgb(attr(c))`
  passes through the way `rgb(var(--c))` already did. Only the browser can
  resolve either. Sass's own `if($cond, $a, $b)` is untouched, since it is
  evaluated long before a value is inspected
- `type()` takes the special-function text path, so its argument is text rather
  than SassScript and reaches the output as written, and a bare `type(`
  lowercases. `-a-type(` is not special, matching dart-sass: its argument stays
  a Sass expression and its quotes are normalized

### Changed

- **Breaking: which names a `@function` may have**, following the
  [function-name proposal](https://github.com/sass/sass/tree/main/accepted/function-name.md)
  and dart-sass 1.103.1. `calc` and `clamp` are ordinary names now, and a
  function of either name wins over the calculation. `expression`, `url`,
  `and`, `or` and `not` are reserved only as spelled, so `-a-and()` is a legal
  name; `element` stays reserved through any vendor prefix. `type` is newly
  reserved for the plain-CSS function. The name is checked as written rather
  than with `_` normalised to `-`, so `-moz_element` is legal and
  `-moz-element` is not
- **Breaking: the minimum supported Rust version rises from `1.85.0` to
  `1.96.1`**, normalising the floor across the Accent crates. Per this
  project's policy, that makes the next release a minor version bump. The
  gating CI jobs move with it.

  Clippy reads `rust-version`, so the bump turned lints on with no code
  change: `collapsible_if` began suggesting let chains, which need 1.88, at 23
  sites. All 23 are collapsed. No comment was displaced -- each sat above the
  outer `if`.
- **Breaking: `@extend` is scoped to the extending module's upstream closure**,
  porting dart-sass's per-module extension model. A module loaded by `@use` or
  `@forward` gets its own extension store, and one loaded in an import context
  shares the enclosing store, because `@import` means "as if written here". An
  `@extend` no longer reaches CSS in a sibling module that never loaded it, and
  a mandatory `@extend` whose target is out of scope now errors instead of
  silently succeeding. Closes
  [connorskees/grass#104](https://github.com/connorskees/grass/issues/104) for
  this fork
- **Breaking: a space-separated calculation is stricter.** Values written next
  to each other are only meaningful when a neighbour is text the compiler
  cannot resolve, and the check for that looked inside an operation rather than
  at the operation itself. `calc(1 px + 2px)`, `calc(1 px * 2)`,
  `calc(1 (px + 2px))`, `calc(1 var(--c) + 2px)`, `calc(var(--c) + 1px 2px)`
  and `calc(1 min(var(--c), 2px))` compiled here and are "Missing math
  operator." in dart-sass; they now error too. An operation is not text however
  much text it holds, which is what separates the rejected `calc(1 px + 2px)`
  from the legal `calc(#{$a} px + 2px)`
- **Breaking: `CalculationArg` gains a `Paren` variant**, public through
  `sass_value`, so an exhaustive match on it needs a new arm. Parentheses
  around a space-separated group are now recorded rather than inferred from
  position: `calc(1 (2 var(--c)) 3)` keeps them instead of flattening to
  `calc(1 2 var(--c) 3)`, which this compiler itself rejects on the way back
  in. Every pair the source wrote is kept, as dart-sass keeps them
- a declaration written after a nested rule splits the parent rule instead of
  being hoisted back up beside the earlier declarations, which is what
  dart-sass does. Where both set the same property, that changes the cascade
- a loud comment does the same: one written after a nested rule holds its place
  in source order rather than hoisting up beside an earlier comment
- an interpolation inside a calculation is one operand of the expression rather
  than opaque text for the whole argument, unless it is the whole argument. The
  source's whitespace is re-serialized (`calc(#{$a}  +  2px)` is
  `calc(1px + 2px)`), and an inlined nested `calc()` keeps only the parentheses
  its precedence needs. An interpolation written against an identifier belongs
  to that identifier -- `calc(x#{$a})` is `calc(x1)` -- and such an identifier
  is opaque text, so it names neither a calc constant nor a function
- adjacency in a calculation binds looser than any operator, as it does in
  dart-sass, so `calc(#{$a} px + 2px)` is `1` beside `px + 2px` rather than
  `(1 px)` plus `2px`

### Fixed

- the three crate manifests declare `rust-version = "1.96.1"`, the MSRV this
  project documents and gates on. They said `1.96`, so cargo accepted a build
  on 1.96.0 -- a release this project has never tested against and does not
  support. The supported floor does not move; the manifests now say what the
  README, this file and the three gating CI jobs already said
- an at-rule header in the indented syntax may be split across lines wherever
  dart-sass allows it. A newline ends a statement in `.sass`, but not at a
  position where a statement cannot end -- between `@function` and its name,
  between `@for` and its variable, after `from`, after a binary operator,
  inside an argument list or a bracketed list. This fork modelled that with a
  parenthesis depth counter, which only covered the parentheses; it now carries
  dart-sass's `consumeNewlines` value at each call site, which closed 114
  sass-spec failures across 26 areas. See
  [`specs/docs/features/10-indented-newlines.md`](specs/docs/features/10-indented-newlines.md)
- `//` inside a plain CSS value is two slashes, not the start of a comment
  plain CSS forbids. `a {b: 1///bar}` in a `.css` file was rejected
- whitespace may fall anywhere inside `@import ... supports(...)`, including a
  newline in the indented syntax: the whole query sits inside parentheses, so
  it is whitespace there the way it is in an argument list. The same applies to
  a non-`supports` modifier's arguments
- the indented syntax tolerates a trailing `;` at the end of a statement, and
  says "multiple statements on one line are not supported in the indented
  syntax." when a second statement follows it. A `;` was rejected outright
- `@import` ends with a statement separator, so `@import "a.css" b` and an
  indented block beneath an `@import` are errors, and a trailing `;` after it
  in the indented syntax is not
- a custom property may have an empty value (`--a:;`), per the CSS spec and
  dart-sass 1.103.1. It was an error, and `--a:{b: c}` was accepted where
  dart-sass expects a `;`
- a declaration whose value has no CSS representation -- an empty list, say --
  now raises the error dart-sass raises instead of being dropped from the
  output silently
- the indented syntax names what a stray indented block sits beneath, as
  dart-sass does: `Nothing may be indented beneath a @import rule.` rather than
  `Nothing may be indented here`
- `selector.replace()` rejects a parent selector in any of its three arguments,
  matching `selector.extend()` and dart-sass. It previously accepted `&` and
  panicked while serializing the result
- `@extend` is deterministic. `ExtendedSelector` hashed its pointer but
  compared its value, so two rules with equal selectors could hash differently,
  collide in a hash set, and leave one of them without its extension -- wrong
  output in 2 of 400 runs of the same input
- a silent comment inside a special function is dropped rather than copied into
  the output, and a quoted string inside one keeps the source's quote character
  (`-a-calc('x')` stays `'x'`, while `unknown('x')` is still normalized). A
  custom property keeps its `//`, since its value is raw CSS where `//` is two
  slashes, and `url()` and `url-prefix()` hold a raw URL for the same reason

## [0.14.0] - 2026-09-04

The first release under the `accent-sass` name, and the first whose version
is the fork's own rather than inherited from upstream. It collects twenty
merged pull requests: the complete CSS Color 4 model, the CSS math functions,
the CSS `if()` function, first-class mixins, and the module-system work.

Against the pinned sass-spec revision (`4a9eea66`) this takes the suite to
13,560 of 14,218 passing, measured 2026-09-04.

### Added

- CSS math functions inside calculations -- `round()`, `mod()`, `rem()`, `abs()`, `sign()`, `log()`, `exp()`, `pow()`, `sqrt()`, `hypot()`, `clamp()`, `min()`, `max()`, and the trigonometric functions `sin()`, `cos()`, `tan()`, `asin()`, `acos()`, `atan()`, `atan2()` -- and the calculation constants `pi`, `e`, `infinity`, `-infinity` and `NaN`
- the CSS `if()` function
- first-class mixins, and `meta.load-css()` with configuration
- support every CSS Color 4 color space, matching Dart Sass 1.79+: colors can be written with the plain-CSS `lab()`, `lch()`, `oklab()`, `oklch()`, and `color()` functions (`color(display-p3 1 0 0)`, `color(xyz 0.3 0.2 0.1)`, and the `srgb`, `srgb-linear`, `display-p3-linear`, `a98-rgb`, `prophoto-rgb`, `rec2020`, `xyz-d50`, and `xyz-d65` spaces), any channel or the alpha can be `none`, and a color remembers its space: `color.to-space()`, `color.channel()`, `color.adjust()`, `color.change()`, `color.scale()`, `color.mix()` (with a `$method`), `color.invert()` and `color.complement()` (with a `$space`), `color.grayscale()`, `color.is-in-gamut()`, `color.to-gamut()` (`clip` and `local-minde`), `color.same()`, `color.is-legacy()`, and `color.ie-hex-str()` accept and produce colors in every space, with every conversion computed in the same operation order as Dart Sass
- add `color.is-missing()` and `color.is-powerless()`, and `color.opacity()` to the `sass:color` module
- track the legacy color space (`rgb`, `hsl`, or `hwb`) a color was written in or last converted to, matching Dart Sass 1.79+: `color.space()` reports it, an hsl-space color always serializes as `hsl(..)` (so `color.adjust(hsl(120 100% 50%), $lightness: -50%)` prints `hsl(120, 100%, 0%)` instead of `black`), an hwb-space color serializes as a name or hex code when its rgb channels are whole numbers and as `hsl(..)` otherwise, `color.mix()` without a `$method` and `rgba($color, $alpha)` produce rgb-space colors, and compressed output takes the shorter of the rgb and hsl forms
- add `color.to-space()`, `color.is-legacy()`, `color.is-in-gamut()`, `color.to-gamut()` (both the `clip` and `local-minde` methods), and `color.same()` for the legacy spaces
- accept the `$space` argument to `color.invert()` and `color.complement()`, and the `$method` argument to `color.mix()` (`rgb`, `hsl`, or `hwb`, optionally with a `shorter`, `longer`, `increasing`, or `decreasing` hue interpolation method); `color.mix()` with a `$method` interpolates with premultiplied alpha per CSS Color 4
- support missing hues: converting an achromatic color into hsl or hwb (with an explicit `$space`, or implicitly in `color.invert()`) yields a color whose hue is `none`, which serializes as `hsl(none 0% 50%)` and which `color.adjust()`, `color.scale()`, `color.invert()`, and `color.complement()` refuse to modify with Dart Sass's error
- add the plain-CSS `hwb()` function (space-separated syntax) as a color; `hwb()` no longer range-checks whiteness or blackness, scaling a sum above 100% like Dart Sass
- add `color.channel($color, $channel, $space: null)` for the legacy rgb/hsl/hwb spaces
- accept the `$space` argument to `color.adjust()`, `color.change()`, and `color.scale()` (rgb/hsl/hwb only)
- store legacy color channels as floats instead of rounding to integers, matching Dart Sass 1.79+; non-integral rgb colors serialize as `rgb(R%, G%, B%)`, colors written as or derived from `hsl()`/`hwb()` serialize in `hsl(..)` form (without a `deg` suffix), and a NaN hue serializes as `calc(NaN)`
- support unquoted imports in the indented/SASS syntax

### Changed

- **Breaking: the crates move to the Rust 2024 edition**, which raises the minimum supported Rust version from `1.70.0` to `1.85.0`. The workspace sets `resolver = "3"` to match, and formatting follows the 2024 style edition -- reordered imports and collapsed `if`/`else` across the tree
- **Breaking: the proc-macro crate is renamed** from `include_sass` to `accent-sass-macro`. `include_sass` on crates.io belongs to upstream `grass`, so the fork cannot publish under it. Users of `accent_sass::include!` are unaffected; the dependency is internal
- **Breaking: the crates are renamed.** `grass` is now `accent-sass`, `grass_compiler` is now `accent_sass_compiler`, and the binary is `accent-sass`. Rust paths move from `grass::` to `accent_sass::`. The upstream this forks is still `connorskees/grass`
- `@forward` visibility (`show`/`hide`) and module member conflicts are enforced
- `meta.load-css()` and `@use` share one module instance, and the module cache takes part in `@import` of configured modules
- the sibling-combinator compounds move instead of cloning during selector unification
- serialize non-legacy colors as Dart Sass does: `lab(50% 10 20)`, `oklch(50% 0.1 20deg)`, `color(srgb 1 0 0 / 0.5)`, `none` for missing channels (also `rgb(none 1 2)` and `hsl(120deg none 50%)` for legacy colors with a missing channel), and a lab-family color whose lightness is out of range as `color-mix(in lab, color(xyz ...) 100%, black)` or with a relative `from black` prefix
- the legacy functions (`red()`, `hue()`, `lighten()`, `opacify()`, `alpha()`, `rgba($color, $alpha)`, `mix()` without a `$method`, `invert()` without a `$space`, ...) reject non-legacy colors with Dart Sass's errors; `color.mix()` and `color.invert()` name the color that needs a `$method` or `$space`
- rewrite the `$channels` parsing of `rgb()`, `hsl()`, and `hwb()` as a port of Dart Sass's `_parseChannels`: the error messages match (`$channels: Expected a space- or slash-separated list, was (1, 2, 3)`, `The rgb color space has 3 channels but (1 2 3 4) has 4.`, `$red: Expected 10px to have unit "%" or no units.`), `var()` and `calc()` fall back to the plain-CSS function call (`color.hwb(var(--foo))` is `hwb(var(--foo))`), and a `%` alpha or channel scales like Dart Sass
- match Dart Sass's number output exactly: `inspect()` prints full precision (`0.6666666666666666`), a whole number prints its exact integer digits, a number is rounded to ten decimal places by decimal digit (so `0.99999999999999` prints `1` instead of `0` in compressed mode), and compressed output keeps the zero of a short negative number (`-0.5`)
- `rgb($color, $alpha)` with a NaN alpha yields `rgba(255, 0, 0, 0)` like Dart Sass, and the plain-CSS `invert()`, `grayscale()`, and `opacity()` write a NaN argument as `calc(NaN)`
- `color.channel()` reads the channels of the color's own space (or of `$space`), errors on a channel the space does not have, and is case-sensitive; `color.adjust()`, `color.change()`, and `color.scale()` pick the space from the channel keywords, error on a keyword the space does not have (`$red: Color space hsl doesn't have a channel with this name.`), clamp adjusted rgb channels instead of erroring on the range, leave changed channels unclamped (`color.change(#cc0f35, $red: 300)` is out of gamut), and scale a channel that is already out of range no further
- match Dart Sass's hsl/hwb conversion bit for bit (plain multiply-add operation order instead of FMA, hue scaled as `(hue / 360) % 1`), round the legacy `red()`/`green()`/`blue()` results like Dart Sass does, and keep out-of-gamut legacy hsl colors unclamped (`hsl(-1 -1 -1)` round-trips as `hsl(359, 0%, -1%)`; saturation is lower-clamped at 0 like the CSS channel)
- serialize an interpolated `calc()` without the source's leading/trailing whitespace inside the parentheses, matching Dart Sass
- keep a trailing loud comment on the same output line as the declaration it follows, matching Dart Sass
- apply `@extend` across `@use`/`@forward` boundaries: extending a placeholder defined in another module now emits the extended rule instead of nothing (the shared-store approximation of connorskees/grass#104)
- indent continuation lines of a multi-line selector list to the current level, matching Dart Sass
- `color.adjust()` no longer clamps lightness (and no longer upper-clamps saturation), matching Dart Sass 1.79+; a legacy color pushed out of the rgb gamut serializes in `hsl(..)` form
- serialize numbers that have no plain-CSS representation as `calc()`, matching Dart Sass: a non-finite value becomes `calc(infinity)`, `calc(-infinity)`, or `calc(NaN)` (with units as factors, e.g. `calc(infinity * 1px)`), and complex units become e.g. `calc(1px / 1em)` instead of erroring with "isn't a valid CSS value"

### Fixed

- selector unification ordering, and namespace superselectors
- comments and newlines are accepted everywhere the grammar already permits them
- the MSRV build, and user-defined functions whose names collide with the new math functions
- fix `%` with an infinite operand: an infinite dividend is NaN, and an infinite divisor keeps the dividend when the operands share a sign and is NaN otherwise
- error when `@extend` is used across `@media` boundaries
- more robust support for NaN in builtin functions

---

## Upstream history (`grass`)

Everything below is upstream's changelog. Only the heading levels changed, so
the versions nest under this document's title; the text is untouched.

## 0.13.4

- support `...$keys` argument to `map-has-key(..)`/`map.has-key(..)`
- parse [aliased colors](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color#description) (e.g. `cyan` for `aqua`) as colors rather than identifiers

## 0.13.3

- implement builtin string-module function `string.split(..)` (#96) by @xpe
- implement functionality for intercepting logs (#93) by cryocz

## 0.13.2

- update rustix dependency to silence security warning
- fix @forward statement altering the scope of the forwarded module (#85) by @kketch
- bump MSRV to 1.70.0

## 0.13.1

- update `clap` dependency to 4.x.x to silence `atty` security warning
- bump MSRV to 1.64.0 for new `clap` version
- fix bug in which `--no-charset` flag wasn't respected

## 0.13.0

- fix various module system bugs when combined with `@import`. this is potentially breaking in rare cases where users were relying on the incorrect behavior
- expose more AST internals in `grass_compiler`
- allow building docs with stable/beta rust compiler
- support `...$keys` argument to `map-get(..)`/`map.get(..)` (#83)

## 0.12.4

- implement builtin map-module functions `map.deep-merge(..)` and `map.deep-remove(..)`

## 0.12.3

No visible changes for users of the `grass` crate

Exposes more internals of the `grass_compiler` crate, allowing for custom functions implemented in rust to be accessed from Sass.

## 0.12.2

- implement an import cache, significantly improving the performance of certain pathological cases
- slash lists can be compared using `==`
- resolve rounding errors for extremely large numbers
- potentially breaking bug fixes in certain color functions
  - `color.hwb(..)` no longer allows whiteness or blackness values outside the bounds 0% to 100%
  - `scale-color(..)` no longer allows the `$hue` argument. previously it was ignored
  - `scale-color(..)`, `change-color(..)`, and `adjust-color(..)` no longer allow invalid combinations of arguments or unknown named arguments
  - many functions that accept hues now convert other angle units (`rad`, `grad`, `turn`) to `deg`. previously the unit was ignored
- improve compressed output of selectors containing newlines and `rgba(..)` colors
- improve resolution of imports containing explicit file extensions, e.g. `@import "foo.scss"`
- fix bug in which whitespace was not emitted between `+` or `-` inside calc for compressed output ([#71](https://github.com/connorskees/grass/pull/71) by @ModProg)

## 0.12.1

- add `grass::include!` macro to make it easier to include CSS at compile time
- various optimizations improving the bootstrap benchmark by ~30% and the bulma benchmark by ~15%
- improve error message for complex units in calculations
- more accurate formatting of named arguments in arglists when passed to `inspect(..)`
- more accurate formatting of nested lists with different separators when passed to `inspect(..)`
- support `$whiteness` and `$blackness` as arguments to `scale-color(..)`
- more accurate list separator from `join(..)`
- resolve unicode edge cases in `str-index(..)`
- more robust support for `@forward` prefixes
- allow strings as the first argument to `call(..)`
- bug fix: add back support for the `$css` argument to `get-function(..)`. regressed in 0.12.0

## 0.12.0

- complete rewrite of parsing, evaluation, and serialization steps
- **implement the indented syntax**
- **implement plain CSS imports**
- support for custom properties
- represent all numbers as f64, rather than using arbitrary precision
- implement media query merging
- implement builtin function `keywords`
- implement Infinity and -Infinity
- implement the `@forward` rule
- feature complete parsing of `@supports` conditions
- support media queries level 4
- implement calculation simplification and the calculation value type
- implement builtin fns `calc-args`, `calc-name`
- add builtin math module variables `$epsilon`, `$max-safe-integer`, `$min-safe-integer`, `$max-number`, `$min-number`
- allow angle units `turn` and `grad` in builtin trigonometry functions
- implement `@at-root` conditions
- implement `@import` conditions
- remove dependency on `num-rational` and `beef`
- support control flow inside declaration blocks
  For example:

```scss
a {
  -webkit-: {
    @if 1 == 1 {
      scrollbar: red;
    }
  }
}
```

will now emit

```css
a {
  -webkit-scrollbar: red;
}
```

- always emit `rgb`/`rgba`/`hsl`/`hsla` for colors declared as such in expanded mode
- more efficiently compress colors in compressed mode
- treat `:where` the same as `:is` in extension
- support "import-only" files
- treat `@elseif` the same as `@else if`
- implement division of non-comparable units and feature complete support for complex units
- support 1 arg color.hwb()

## 0.11.2

- make `grass::Error` a `Send` type
- expose more internals of `grass::Error`, allowing for custom formatting
- fix WASM builds

## 0.11.1

- fix load path bug in which paths were searched for relative to the SCSS file, not the executable (#57)

## 0.11.0

- `fs` option added to allow interception and reimplementation of all file system operations (such as imports)
- `wasm` feature renamed to/replaced with `wasm-exports`, which no longer materially alters the API: `from_path` is reinstated, and `from_string` once again returns the full error type; but the WASM export `from_string` (which returns a string error) is now a new function `from_string_js`. (It was renamed from `wasm` to `wasm-exports` because the name was misleading; Rust code that uses grass doesn’t need this feature, it’s solely to get this `from_string` WASM export.)

## 0.10.8

- bugfix: properly emit the number `0` in compressed mode (#53)

## 0.10.7

- special case plain CSS fn `clamp`
- support more uses of plain CSS fns inside `rgb`/`rgba`/`hsl`/`hsla`
- better support for `@at-root` at the toplevel and inside media queries
- bugfixes for the module system
- more robust handling of load paths that are directories

## 0.10.6

- **feature complete, byte-for-byte support for bootstrap**
  - add bootstrap v5.0.2 to ci
  - run script to verify output against the last 2,500 commits to bootstrap
- feature complete `min`/`max` support -- special functions and `min`/`max` are now allowed as arguments
- removed dependency on `peekmore`, which sped up parsing and simplified lookahead
- emit comments inside the `@if` rule body
- fix bug in `hue(...)` function in which the value would be incorrect when the `red` channel was the highest and the green channel was lower than the blue channel
- no longer round output from `saturation(...)` function
- improve handling of newlines for `@media`, `@supports`, `@at-root`, placeholder selectors, unrelated style rules, and unknown @-rules
- arglists can be equal to comma separated lists
- throw error for invalid uses of `@charset`
- more robustly parse `@else if`, allowing escaped and uppercase characters
- resolve two `@extend` bugs -- one in which we would incorrectly emit `a b, a > b` as a selector, even though `a b` is a superselector of `a > b`, and a feature called "three-level extend loop", in which a stylesheet where `a` extends `b`, `b` extends `c`, and `c` extends `a` would fail to include all 3 selectors in certain places
- support compressed values for comma separated lists and numbers
- more robustly parse unknown @-rules

## 0.10.5

- support compressed output
- support new builtin functions `math.div`, `map.set`
- support the HWB colorspace and builtin functions `color.hwb`, `color.blackness`, `color.whiteness`
- `:is` pseudo selector is now considered an alias of `:matches` in `@extend`
- support `$keys...` argument in `map.merge`
- `%` now implements the modulo operation, rather than finding the remainder. this largely affects negative numbers
- fix parsing bug in which `/***/` in a selector would miss the closing `/`

## 0.10.4

- plain css `invert(..)` accepts numbers with any unit
- plain css imports (e.g. `@import url(foo)` or `@import "foo.css"`) are now emitted at the top of documents

## 0.10.3

- hyphen followed by interpolation is not treated as subtraction, e.g. `10-#{10}` => `10 -10` rather than `0`
- function arguments do not affect variables in outer scopes (fixes [#37](https://github.com/connorskees/grass/issues/37))
- improve error messages for NaN with units passed to builtin functions

## 0.10.2

- use `std::fs::OpenOptions` to open files ([#35](https://github.com/connorskees/grass/pull/35) by [@MidasLamb](https://github.com/MidasLamb))

## 0.10.1

- **implement `@use` and the module system**
- support the filter syntax for function arguments, e.g. `alpha(opacity=1)`
- disallow certain at-rules in functions, resolving several panics
- allow vendor-prefixed special CSS functions, e.g. `-webkit-calc(...)`
- allow decimal percent selectors inside `@keyframes`
- allow vendor-prefixed `@keyframes`
- resolve parsing bug for maps involving silent comments
- allow escaped `!` in selectors
- allow multiline comments in functions
- resolve several panics on malformed input when parsing bracketed lists
- support NaN in all contexts
- add support for unicode ranges
- recognize plain CSS imports beginning with `//`, e.g. `@import "//fonts.googleapis.com/css?family=Droid+Sans";`
- resolve integer overflows in `@for` when bounds were equal to `i32::MIN` and `i32::MAX`
- allow quoted strings in default function arguments

## 0.10.0

- bugfixes for `@media` query regressions
- bugfixes for maps, arglists, and `@each`
- implement string interning for identifiers and style properties
- implement spec-compliant variable scoping
- emit `@import` when importing `url(...)` or `*.css`
- resolve all panics for malformed `@import`
- various optimizations that now allow us to compile bootstrap 10% faster than `libsass`
- errors inside builtin functions use `inspect` to print values
- bugfixes for color and map equality (e.g. `red` == `#ff0000`)
- hide unimplemented command line flags
- implement CLI options for `--quiet`, `--load-path` ([#22](https://github.com/connorskees/grass/pull/22) by @JosephLing), `--no-charset`, `--stdin`, and `--no-unicode`
- use unicode characters in error messages by default
- allow comma separated `@import` statements ([#23](https://github.com/connorskees/grass/pull/23) by @JosephLing)
- implement and correctly parse `!optional` in `@extend`
- lazily evaluate `!default` variable values
- disallow interpolation in mixin and function names
- improve parsing for `@supports` and unknown at-rules

### Breaking

- functions now take an `Options` struct

## 0.9.5

A small release fixing potential build issues and improving documentation.

This release is not published to NPM due to [a bug](https://github.com/rustwasm/wasm-pack/issues/837)
in `wasm-pack`.

## 0.9.4

- implement `@keyframes`
- don't strip newlines following comments in selectors

## 0.9.3

- fix parsing bugs for empty bracketed lists
- partially implement inverse units
- remove all remaining `todo!()`s from binary and unary ops
- parse keywords case sensitively
- various optimizations that make bulma about _6x faster_ to compile

## 0.9.2

- implement builtin functions `min` and `max`
- bugfixes for `@extend` and `selector-unify`
- allow `@content` to take arguments
- bugfixes for `@content`, for example it will no longer infinitely recurse for chained mixins
- better support queries in `@media`
- bugfixes for `@media`
- add support for splats, e.g. `rgba([1, 2, 3, 4]...)`
- resolve a number of parsing bugs for `@for`, variable declarations, selectors, and maps
- completely rewrite how styles are evaluated, allowing short circuiting of values like `false and unit(foo)` and `if(true, foo, unit(foo)`

## 0.9.1

This release is largely focused on `@extend`, but it also resolves some regressions resulting from the new parser.

- **implement `@extend`**
- properly document new API
- MVP implementation of `@supports`
- fix regression in which `@at-root` would panic when placed after a ruleset
- fix regression related to `@mixin` and `@function` scoping when combined with outer, local variables
- remove most remaining `unwrap`s that could result in a panic

## 0.9.0

This release is focused on setting up the groundwork for implementing `@extend` as well
as being able to compile Bootstrap.

- implement all builtin selector functions
  - `selector-append`
  - `selector-extend`
  - `selector-nest`
  - `selector-parse`
  - `selector-replace`
  - `selector-unify`
  - `simple-selectors`
  - `is-superselector`
- implement builtin function `content-exists`
- allow `@import`, `@warn`, and `@debug` in all contexts, such as inside `@mixin`
- refactor control flow evaluation, resolving some issues blocking Bootstrap

#### Breaking Changes

- remove the `StyleSheet` struct in favor of freestanding functions, `from_string` and `from_path`

## 0.8.3

This release is largely focused on performance and robustness

- implement smallint optimization for numbers, making some benchmarks 50% faster
- remove `bimap` as a dependency for storing named colors in favor of an ad hoc, more specialized data structure
- remove _dozens_ of panics on malformed input
- use `beef::Cow` instead of `std::borrow::Cow`
- increase code coverage to 80%

## 0.8.2

This release contains significant (>10x) improvements for WASM speed.
Performance is now comparable to libsass bindings with `node-sass` as
well as `dart-sass` with dart2js. It is, however, roughly 4x slower than
native `grass`.
