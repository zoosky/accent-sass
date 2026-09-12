# The `@warn` banner, and repeated warnings

Two defects in how `@warn` reaches a user. Neither touches CSS output, and
neither is visible under the standard spec flags.

1. **A repeated `@warn` is swallowed.** The evaluator keys emitted warnings by
   span, so a mixin included twice warns once, and a `@warn` inside a loop
   warns on the first iteration only -- discarding messages whose text
   differs. No sass-spec fixture can catch this.
2. **The banner and location line differ.** This compiler writes
   `Warning: careful` above `./input.scss:1:7`; dart-sass writes
   `WARNING: careful` above an indented stack trace. Worth 18 fixtures.

Defect 1 is ranked first despite being worth no fixtures. The banner costs
fixtures; the dedupe loses information a user asked for.

[#93](https://github.com/zoosky/accent-sass/pull/93) fixed a third defect in
this area, the message text itself: `@warn "careful"` reported `"careful"`
with its quotes. That has landed; this item is what is left.

## Measurement

Measured 2026-09-12 against master (`85df0341`), sass-spec `b39c32768`, and
the native dart-sass 1.104.0 binary for macOS arm64, using the release build.

| Run | Failures |
|---|---:|
| Full suite, standard flags | 113 |
| Full suite, without `--ignore-warning-diffs` | 847 |
| The 23 files holding a plain `WARNING` expectation, standard flags | 1 |
| The same 23 files, without `--ignore-warning-diffs` | 43 |

So 734 failures across the suite are warning-only, and 42 of them are plain
`@warn` rather than a deprecation. The remaining 692 are deprecation
warnings, which is consistent with the 701 fixtures whose expected warning
begins `DEPRECATION WARNING`; they need the facility
[item 08](08-calculation-warnings-and-error-wording.md) describes and are not
this item's work.

The 43rd failure in that scoped run is
`spec/directives/at_root/sass/empty/no_query`, which fails earlier on an
indented-syntax parse gap belonging to
[item 19](19-indented-syntax-gaps.md). Three of the 45 fixtures that
statically expect a plain `WARNING` pass; why was not investigated.

### What the banner alone closes: 18, not 42

Each failing fixture's input was extracted and run through the release
binary, and the 42 split by whether this compiler emits any warning at all:

**18 where a warning is emitted and only its banner is wrong.** These are what
defect 2 closes: all 13 of `spec/directives/warn`, plus
`libsass-closed-issues/{issue_192,issue_2156/warn}`,
`libsass/debug-directive-nested/{function,mixin}` and `libsass/propsets`.

**27 where nothing is emitted at all.** The warning does not exist in this
compiler, so a banner fix leaves them failing. Six are
`css/selector/combinator/newline`, which is
[item 15](15-bogus-combinators.md); the rest are deprecation and
indented-syntax warnings spread across `core_functions/math/div`,
`directives/{extend,if,mixin}/whitespace`, `css/unknown_directive`,
`parser/selector`, `operators/newlines` and five `libsass-closed-issues`
files. Counting them here would credit this item with work that belongs to
others.

## Defect 1: a repeated `@warn` is swallowed

`Visitor::visit_warn_rule` guards on a `HashSet<Span>`
(`crates/compiler/src/evaluate/visitor.rs:2460`, the field at `:182`,
initialized at `:285`):

```rust
if self.warnings_emitted.insert(warn_rule.span) {
```

The span is the `@warn` rule's own position, so every execution of one rule
shares a key. Measured against this input:

```scss
@mixin m { @warn "from mixin"; }
.a { @include m; }
.b { @include m; }
@each $i in 1, 2, 3 {
  @warn "loop #{$i}";
}
```

dart-sass emits five warnings; this compiler emits two. The second
`@include` is dropped, and so are `loop 2` and `loop 3` -- messages whose
text differs from the one that was printed. The comment on the field reads
"avoid emitting duplicate warnings for the same span", which is dart-sass's
rule for *deprecation* warnings, not for `@warn`.

**No fixture can catch this.** The runner's `extractWarningMessages`
(`sass-spec/lib/test-case/compare.ts:31`) keeps only the *first* line
matching `/^\s*(DEPRECATION )?WARNING/` -- it calls `.find()`, not `.filter()`
-- so a dropped second warning is invisible to every fixture in the suite.
That is why a defect that discards user output survived a 14,266-test suite,
and why this needs a regression test rather than a fixture.

`warnings_emitted` has exactly one reader, so removing the dedupe is one
`if`. Check first whether anything wants it: the guide notes USWDS emits
about 12 KB of `@warn` per compile, and repeats may make that noticeably
worse. That is a reason to measure the volume, not a reason to drop
messages.

## Defect 2: the banner and location line

`StdLogger::warn` (`crates/compiler/src/logger.rs:31-39`):

```rust
eprintln!(
    "Warning: {}\n    ./{}:{}:{}",
    message, location.file.name(), location.begin.line + 1, location.begin.column + 1
);
```

dart-sass writes, for `@warn "careful"` at the top level:

```
WARNING: careful
    input.scss 1:1  root stylesheet
```

and for a warning raised inside a function called from a mixin, one frame per
level (`spec/directives/warn.hrx`, `functions_in_stack`):

```
WARNING: From function: testing
    input.scss 4:3   issues-warning()
    input.scss 9:11  calls-function-that-warns()
    input.scss 13:3  root stylesheet
```

Two things follow. **The fixtures need only the first line.** The runner
compares one line, so `WARNING: <message>` is enough to close all 18; the
trace is never compared. **Parity needs the trace anyway**, because a user
reading a warning from inside a framework needs to know where it came from.
Whether the evaluator already tracks a call stack that could produce those
frames was not investigated.

`StdLogger::debug` needs no change. It writes `input.scss:2 DEBUG: a`, which
is byte-identical to dart-sass and is what the two `spec/directives/debug`
fixtures already expect -- they pass today.

## Implementation instructions

- Change the banner to `WARNING:` in `StdLogger::warn`, and leave
  `StdLogger::debug` alone.
- Build the location line as dart-sass does: `file line:column  context`,
  indented four spaces, innermost frame first, ending at `root stylesheet`.
  If no call stack is available, emitting the single `root stylesheet` frame
  still closes the fixtures; say so in the commit rather than implying full
  parity.
- Drop the span dedupe in `visit_warn_rule`, and remove the field if nothing
  else claims it.
- Decide what to do with `--verbose` (`crates/lib/src/main.rs:177`). It is
  declared, its help text promises to "print all deprecation warnings even
  when they're repetitive", and nothing reads it. Either wire it to a
  deprecation-repetition rule when that facility exists, or remove it;
  leaving a flag that does nothing is worse than either.
- `--no-unicode` currently reaches `unicode_error_messages` only. dart-sass
  applies it to warnings too; check before assuming this matters.

## Testing

- Ground truth for the banner: `spec/directives/warn.hrx` at the pinned
  revision, run **without** `--ignore-warning-diffs`, which is not the
  default and must be passed explicitly.
- The dedupe needs Rust tests, since no fixture can see it. `TestLogger` in
  `crates/lib/tests/macros.rs` already collects every message, so a test that
  includes a mixin twice and asserts two entries is a few lines in
  `crates/lib/tests/warn.rs`.
- Verify every expectation against the native dart-sass 1.104.0 binary.
  `npx sass` is the JavaScript build and differs in places.
- The `frameworks` and `bootstrap` jobs compare CSS only, so neither sees
  this. Removing the dedupe changes how much a framework prints to stderr;
  measure USWDS before and after.
- `.github/scripts/wasm-api-smoke.mjs` asserts logger *message values* and
  runs in two workflows outside `tests.yml`. The banner is CLI-only and
  should not reach it, but check rather than assume: a change to what the
  logger receives has broken that script before.

## Acceptance criteria

- `spec/directives/warn` passes without `--ignore-warning-diffs`: 0 failures,
  down from 13.
- The 18 fixtures listed above pass; the 27 that emit no warning are expected
  to still fail and belong to other items.
- A mixin included twice warns twice, and a `@warn` in a three-iteration loop
  warns three times, each with its own text.
- `@debug` output is unchanged.
- The compatibility page
  (`docs/content/03.reference/04.compatibility/default.md`) currently tells
  readers not to match on the lines around a `@warn` message. Update it when
  this lands.
