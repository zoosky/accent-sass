# Indented syntax gaps

9 sass-spec failures in four unrelated `.sass` parsing gaps, plus 1 in
`spec/css/comment` that item 05 counts as residue. Item 10 cleared the
`consumeNewlines` family; what is left is what that parameter did not
reach.

Section 2 has since been closed from another branch -- see the note under
that heading before you start on it. Six of the nine remain.

Measured 2026-09-10 against master (`31361d7`), the pinned sass-spec
revision `4a9eea66`, and dart-sass 1.103.1 run as `npx -y sass@1.103.1`.

| Section | Cause | Failures |
|---|---|---:|
| 1 | A loud comment in a value may not span lines | 4, plus 1 claimed |
| 2 | ~~A selector may not span lines inside brackets~~ | 3, since closed |
| 3 | `@import` takes only one unquoted URL | 1 |
| 4 | `@at-root` with no children is an error | 1 |

Every one of the nine is "Test case should succeed but it did not": the
compiler rejects input dart-sass accepts.

## 1. A loud comment in a value may not span lines

`spec/expressions/comments/as_whitespace/sass/{before-comment,before-comment-no-indent,after-comment,after-comment-no-indent}`,
and `spec/css/comment/loud/multi_line/sass`, which item 05 counts as
residue

In the indented syntax a loud comment inside a declaration value is
whitespace, and it may run past the end of the line:

```sass
a
  b: c /* 
    d */ e
```

```css
a {
  b: c e;
}
```

This compiler stops the value at the newline and then complains about the
comment it has half-read:

```
Error: expected */.
  ,
2 |   b: c /* 
  |           ^^^^^^^^^^
```

The one-line form (`b: c /* d */ e`) already passes, so the comment is
recognized; what is missing is that a comment, once opened, suspends the
rule that a newline ends the statement. That is the same shape as the
`consume_newlines` parameter item 10 threaded through the parser, and the
fix belongs next to it: while scanning a loud comment, read to `*/`
regardless of indentation.

Indentation is not part of the rule -- the `no-indent` variants put the
closing `*/` at column 1 and are equally valid.

## 2. A selector may not span lines inside brackets -- closed

**Closed by #68, which is open at the time of writing.** That branch is
item 24's cause 1, and it needed dart-sass's bracket stack in
`almost_any_value` to reject an unbalanced bracket in a selector. Once the
stack exists, dart-sass's newline case reads
`if (indented && brackets.isEmpty) break loop`, and the `brackets.isEmpty`
half is exactly this section: the two are one mechanism, so the fix landed
there rather than here. The diagnosis below was right, and is kept for the
record.

The change reached seven further fixtures that no document claims, item 24
included -- `spec/css/selector/attribute/sass/whitespace/{after_lbracket,
after_lbracket_indented,after_operator,after_val,before_operator}` and
`spec/css/selector/pseudoselector/whitespace/sass/{after_param,
before_param}`. They are this section's cause seen in a pseudo-selector's
parentheses and around an attribute selector's operator. Recorded here
because this is the section that describes the rule, not because anything
is left to do about them.

`spec/parser/indentation/multiline_indent_level/{none,same,more}`

```sass
a[
b]
  c: d;
```

```css
a[b] {
  c: d;
}
```

An attribute selector may be broken across lines, and the continuation
line's indentation is not significant -- the three fixtures put it at
column 1, at the block's own indentation, and deeper, and all three mean
the same thing. This compiler fails two of them at the bracket with
`Expected identifier.` and the third with `Inconsistent indentation,
expected 4 spaces.`, which shows the indentation stack is being consulted
inside the brackets.

The rule is the same as section 1's: inside a bracket, a newline is not a
statement terminator and the indentation stack does not apply until the
bracket closes.

## 3. `@import` takes only one unquoted URL

`spec/non_conformant/sass/import/unquoted`

```sass
@import unquoted, sub/unquoted
```

dart-sass loads both files. This compiler stops at the comma with
`Expected "url".`.

The difference really is the syntax rather than the import machinery, which
is worth knowing before you go looking in the wrong place: written as
`@import unquoted, sub/unquoted;` in a `.scss` file, *both* engines reject
it with the same `Expected "url".`. Only the indented syntax accepts a
comma-separated list of unquoted URLs, and only dart-sass implements that.

## 4. `@at-root` with no children is an error

`spec/directives/at_root/sass/empty/no_query`

```sass
@at-root
```

dart-sass produces empty output. This compiler fails with `expected
selector.`. The neighbouring `empty/query` (`@at-root (with: rule)`) and
`empty/selector` (`@at-root a`) fixtures pass, so only the bare form is
missing: a childless `@at-root` with no query and no selector is a no-op,
not a parse error.

## Testing

- Ground truth: `spec/expressions/comments.hrx`,
  `spec/parser/indentation.hrx`,
  `spec/non_conformant/sass/import/unquoted.hrx` and
  `spec/directives/at_root/sass.hrx` at the pinned revision.
- Scoped spec runs: `spec/parser`, `spec/expressions`,
  `spec/directives/at_root`, `spec/non_conformant/sass`, plus the whole
  suite for sections 1 and 2, which change how newlines are read.
- Add regression tests to `crates/lib/tests/` with `test!`, in the
  indented syntax, covering both the indented and the no-indent variants
  of section 1. Section 2's three indentation levels are already covered,
  in `crates/lib/tests/selectors.rs`.
- Verify every new expectation against dart-sass 1.103.1 with
  `npx -y sass@1.103.1`, never bare `npx sass`.

## Acceptance criteria

- `spec/expressions/comments`, `spec/directives/at_root` and
  `spec/non_conformant/sass/import` are clear of the six that remain.
  `spec/parser/indentation` was section 2's, and is clear once #68 lands.
- `spec/css/comment` drops by 1 through section 1.
- The whole-suite "Test case should succeed but it did not" count drops by
  at least 6, and the "Expected test to fail but it did not" count has not
  risen: section 1 loosens the parser, which is exactly how leniency creeps
  in.
