---
title: Compatibility
template: docs
lead: >-
  What matches dart-sass, what does not, and how it is measured.
menu:
  visible: true
  order: 4
description: >-
  How accent-sass compares to dart-sass 1.104.0: sass-spec results, the
  framework corpus, and the known divergences.
---

dart-sass is the reference implementation, currently **1.104.0**. A deviation
from it is a bug rather than a dialect, with two declared exceptions: error
message wording and error spans.

## The framework corpus

CI compiles four real frameworks with both engines on every commit. **A single
differing colour value fails the job.**

| Framework | Version | Result |
|---|---|---|
| Bulma | 1.0.4 | No colour differences |
| Pico | 2.1.1 | No colour differences |
| Foundation | 6.9.0 | No colour differences |
| USWDS | 3.13.0 | No colour differences |

Foundation gets a second check: a probe stylesheet calls every Sass function
Foundation documents, one per line, and **any** differing line fails the build.
Whole-framework output tolerates rule grouping and ordering; a function result
does not.

Bootstrap 5.0.2 is compiled too, but advisory: it reports the delta rather than
gating.

### USWDS is byte-identical

Strip the comments from both outputs and USWDS 3.13.0 is identical to
dart-sass: **33,684 lines each, zero differences.** Every selector,
declaration, at-rule and value matches.

With comments, 144 of 33,684 lines differ, and all of them are 20
documentation-comment blocks sitting in a different place. USWDS puts a doc
comment at the top of each rule file above that file's own `@use` rules, and
those files emit no other top-level CSS -- so the comment *is* the module's
CSS, and where it lands is where the module's CSS lands. dart-sass places
module CSS while combining modules at the end of compilation; this compiler
emits it as each module executes. The two orders agree everywhere else.

## The spec suite

The official `sass-spec` suite, run against the release build:

```
14266 runs, 14085 passing, 173 failures, 8 todo, 0 ignored, 0 errors
```

Measured 2026-09-11 against sass-spec `b39c32768`, the first revision carrying
dart-sass 1.104.0's expectations. The job is advisory in CI and publishes the
tallies rather than gating.

The 173 remaining failures are tracked one work item at a time in
[`specs/docs/features/`](https://github.com/zoosky/accent-sass/tree/master/specs/docs/features),
ranked by how many fixtures each would unlock.

## Known divergences

### Compressed output {#compressed-output}

`--style compressed` produces equivalent CSS that is **under-minified** rather
than wrong. Measured across USWDS:

| Class | Count | This compiler | dart-sass |
|---|---:|---|---|
| Numbers inside `calc()` | 623 | `calc(1rem - .25rem)` | `calc(1rem - 0.25rem)` |
| Commas in font stacks | 378 | `Helvetica,Roboto` | `Helvetica, Roboto` |
| `@supports` spacing, `transparent` | 110 | `@supports (mask: ...)` | `@supports(mask: ...)` |
| A rule left empty once its comment is dropped | 8 | kept | omitted |
| Semicolon before the closing brace | 8 | `...:100%;}` | `...:100%}` |

Every one is valid CSS meaning the same thing; the file is simply larger than
dart-sass's. If byte-for-byte minification matters, run a dedicated minifier
over the expanded output.

### Warning and debug formatting

The message text of `@warn` and `@debug` matches dart-sass, including the rule
that a string message is reported as its text: `@warn "careful"` says
`careful`. `@debug` matches in full, down to the line it writes:
`file.scss:2 DEBUG: careful`.

What surrounds a `@warn` message does not. This compiler writes
`Warning: ...` above an `./file:line:column` location, where dart-sass writes
`WARNING: ...` above an indented stack trace. Do not match on those two lines.

### Error messages and spans

Wording may change between bugfix versions, and which characters an error
points at may change. Do not match on either.

### Not implemented

- Source maps
- `--watch`

## How to check a claim yourself

```sh
# The framework corpus, against a real dart-sass binary
.github/scripts/frameworks.sh
```

The script installs the frameworks from npm, compiles each with both engines,
and reports differing lines, colour-bearing differences, and the difference
after canonicalising both sides through `lightningcss`.

> [!NOTE]
> Use the **native** dart-sass release binary, which is what CI uses. The npm
> `sass` package is the JavaScript build and does not always agree with it.
