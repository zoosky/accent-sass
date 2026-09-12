# USWDS parity

**USWDS 3.13.0 compiles to the same CSS as dart-sass 1.104.0.** Strip the
comments from both outputs and they are byte-identical: 27,762 lines each,
zero differences. Every selector, declaration, at-rule and value matches.
A project can use this compiler for USWDS and get the stylesheet dart-sass
would have produced.

Measured 2026-09-12 on master (`f4254a3d`) against the native dart-sass
1.104.0 binary, through `.github/scripts/frameworks.sh`'s entry point
(`@use "uswds"` with `packages` on the load path).

## What still differs, and why it does not matter

| | Expanded (the default) | Compressed |
|---|---|---|
| Rules, declarations, values | identical | equivalent, not identical |
| Comments | all 326 present, 20 blocks in a different order | dropped by both |
| Whole file | 33,684 lines on both sides, 144 differing | not byte-identical |

**Expanded output** differs only in where 20 documentation-comment blocks
sit. USWDS writes a doc comment at the top of each rule and function file,
above that file's own `@use` rules, and those files emit no other top-level
CSS -- so the comment *is* the module's CSS, and where it lands is where the
module's CSS lands. dart-sass places module CSS while combining modules at
the end of the compilation; this compiler emits it as each module executes.
The two orders agree almost everywhere, which is why the difference shows up
in USWDS and in no other framework in the corpus.

The trigger has not been reduced to a minimal case. Five shapes were tried
against the reference and all agreed with it: two comment-headed modules
forwarded from an index; the same where the comment is a repeat; an index
whose order differs from dependency order; and two diamond graphs with a
CSS-less module in the middle. Closing this would start with bisecting
`uswds-utilities` until one hunk reproduces. Before spending that, note the
payoff is comment placement in a file whose CSS already matches.

**Compressed output** (`--style compressed`) is equivalent CSS rather than
identical bytes, and is under-minified rather than wrong. The classes, with
counts from this corpus:

| Class | Count | This compiler | dart-sass |
|---|---:|---|---|
| Numbers inside `calc()` | 623 | `calc(1rem - .25rem)` | `calc(1rem - 0.25rem)` |
| Commas in font stacks | 378 | `Helvetica,Roboto` | `Helvetica, Roboto` |
| Other, mostly `@supports` spacing and `transparent` written as `rgba(0,0,0,0)` | 110 | `@supports (mask: ...)` | `@supports(mask: ...)` |
| A rule left empty once its comment is dropped | 8 | `.usa-prose>table{}` kept | omitted |
| Semicolon before the closing brace | 8 | `...:100%;}` | `...:100%}` |

Every one is valid CSS that means the same thing, so the rendered result is
unchanged; the file is merely a little larger than dart-sass's. The
serializer carries a `todo: compressed` marker in
`crates/compiler/src/selector/complex.rs`, so this is a known unfinished
mode rather than a new finding. The `frameworks` job compiles expanded and
does not see any of it.

## History

The corpus reported **903** differing lines for USWDS from the job's first
run until 2026-09-12, and the roadmap quotes that figure in items 15 and 25,
which is what was true when they were written. Those 903 were *missing*
comments, not misplaced ones: the output was 32,781 lines against
dart-sass's 33,684.

[Item 26](26-pre-module-comment-repeats.md) closed them. It was written for
Bulma's five lines, and the same rule -- a comment above `@use` or
`@forward` is recorded against the module that rule loads, and written again
before every module that loads it -- turned out to account for 759 of
USWDS's 903. What is left is the 144 above.

| Revision | USWDS differing lines | Output lines |
|---|---:|---:|
| `ca6e74b4`'s parent (PR #87) | 903 | 32,781 |
| master after PR #88 | 144 | 33,684 |
| dart-sass 1.104.0 | -- | 33,684 |
