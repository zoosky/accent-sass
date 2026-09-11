# Loud comment fidelity

9 sass-spec failures across seven areas, plus one in `spec/css/comment`
that item 05 counts as residue. None of them is about what a comment
contains; all of them are about where it is printed.

Measured 2026-09-10 against master (`31361d7`), the pinned sass-spec
revision `4a9eea66`, and dart-sass 1.103.1 run as `npx -y sass@1.103.1`.
Re-measured 2026-09-11 on master (`f55ace41`) against sass-spec `b39c32768`:
all 9 still fail, and so does item 05's `css/comment/weird_indentation`.

| Section | Cause | Failures |
|---|---|---:|
| 1 | A comment on the opening-brace line moves to its own line | 6 |
| 2 | A continuation line loses the output indentation | 2, plus 1 claimed |
| 3 | An at-rule prelude keeps a trailing comment | 1 |

## 1. A comment on the opening-brace line moves to its own line

`libsass-closed-issues/issue_{894,941,1007,1567}`,
`non_conformant/basic/06_nesting_and_comments`,
`css/keyframes/bubble/empty`

dart-sass keeps a loud comment on the line it was written on, whether that
line ends a declaration, opens a block or closes one:

| input | dart-sass 1.103.1 | accent-sass |
|---|---|---|
| `a { /**/ }` | `a { /**/ }` | comment on its own line, `}` on a third |
| `.one, .two { /* 3 */ color: red }` | `{ /* 3 */` | comment on its own line |
| `@media screen { /* x */ body {...} }` | `{ /* x */` | comment on its own line |
| `foo baz { margin: 0 } /* end */` | `} /* end */` | comment on its own line |

The compiler already has the machinery: `is_trailing_comment` and
`stmt_end_line` in `crates/compiler/src/serializer.rs:1629-1651`, and
`write_children` rewinds the newline and sets `inline_comment` when they
say so. Two holes leave the six fixtures failing.

**The first child has no previous statement.** `write_children` starts
`prev_end_line` at `None`, so a comment that trails the `{` is never
trailing -- there is nothing to compare against. dart-sass compares the
first child against the *parent*, taking the line of the last `{` inside
the parent's span, and when the comment is the only child it also keeps the
closing brace on that line, which is what makes `a { /**/ }` a one-liner.

**Top-level statements do not go through `write_children`.** They go
through `visit_group` (line 1171), which writes a newline between
statements and knows nothing about comments. That is `issue_1007`, whose
`/* end */` follows a top-level rule.

`stmt_end_line` returns `None` for everything except a style declaration
and a comment, so a comment after a nested rule's closing brace cannot be
trailing either. Give it a line for `CssStmt::RuleSet` and the at-rules,
using the span the statement already carries.

## 2. A continuation line loses the output indentation

`libsass-todo-issues/issue_1026`,
`non_conformant/scss/css_property_comments`, and
`spec/css/comment/weird_indentation`, which item 05 counts as residue

A multi-line comment keeps the relative indentation of its lines, measured
against the column the comment starts at, and is then re-emitted at the
output indentation. `write_comment` at `serializer.rs:1588` does the first
half and drops the second:

```rust
let diff = (line.len() - line.trim_start().len()).saturating_sub(col);
format!("{}{}", " ".repeat(diff), line.trim_start())
```

`diff` is the source indentation minus the comment's start column, which is
the relative part, but nothing adds `self.indentation` back, so every
continuation line starts at column `diff`:

```scss
div a {
  /**
   * a multiline comment
   */
  top: 10px;
}
```

dart-sass prints ` * a multiline comment` indented to match the `/**`;
this compiler prints it at column 1. Add the current indentation to each
continuation line. The first line is already written after
`write_indentation`, so only the lines after it are wrong.

## 3. An at-rule prelude keeps a trailing comment

`css/moz_document/comment/after_arg/loud`

```scss
@-moz-document url-prefix(a) /**/ {}
```

dart-sass prints `@-moz-document url-prefix(a) {}` and this compiler prints
`@-moz-document url-prefix(a) /**/ {}`. The comment is part of the at-rule
value, which `almost_any_value` in
`crates/compiler/src/parse/stylesheet.rs` copies verbatim -- item 11's
sections 1 and 2 worked in the same function, dropping silent comments
there and fixing quote characters. The silent variant of this fixture
(`after_arg/silent`) already passes because of that work; the loud one
needs the value trimmed of a trailing loud comment.

Check what a loud comment *between* two parts of a value does before you
trim unconditionally: `@a b /**/ c;` keeps the comment in dart-sass, so
this is specifically about a comment at the end of the prelude.

## Testing

- Ground truth: `spec/css/comment/weird_indentation.hrx`,
  `spec/css/moz_document/comment.hrx`, and the five libsass issue files
  named above, at the pinned revision.
- Scoped spec runs: `spec/css/comment`, `spec/libsass-closed-issues`,
  `spec/non_conformant`, and the whole suite -- section 1 changes the
  serializer for every block in every stylesheet, so a scoped run proves
  nothing about the blast radius.
- Add regression tests to `crates/lib/tests/` with `test!`, one per row of
  the table in section 1 and one multi-line comment for section 2.
- Verify every new expectation against the reference, dart-sass 1.104.0
  since [item 25](25-baseline-before-dart-sass-1-104.md), using the native
  release binary. `npx sass` runs the JavaScript build, which gives
  different answers in places.

## Acceptance criteria

- The six fixtures in section 1, the two in section 2 and the one in
  section 3 pass.
- `spec/css/comment` drops from 10 failures to 9. On `f55ace41`, item 19's
  section 1 names one of the nine left, `loud/multi_line/sass`. The other
  eight are item 05's residue, and item 05 lists them without a cause
  written down for any: five loud comments in the indented syntax, under
  `block/loud/sass` and `error/loud/sass`, and three under `sourcemap`.
- `spec/css/font-face/bubble/empty` needs section 1 as well as item 23; it
  is counted under item 23, not here.
- The whole-suite count does not rise anywhere else. The serializer change
  in section 1 touches every block, so the whole-suite number is the
  measurement that counts.
