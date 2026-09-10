# A custom property's raw text

6 sass-spec failures in `spec/css/custom_properties`, all one cause: a
custom property's value is raw CSS and keeps its newlines and indentation,
and this compiler folds it onto one line.

Measured 2026-09-10 against master (`31361d7`), the pinned sass-spec
revision `4a9eea66`, and dart-sass 1.103.1 run as `npx -y sass@1.103.1`.

Item 11 recorded this divergence when it landed and left it open; the
`silent_comment_with_following_line` test in
`crates/lib/tests/custom-property.rs` pins the current behavior so a change
to it is visible.

```scss
a {
  --b: c
    d;
  --e: {
    f: g;
  };
}
```

dart-sass prints the value back as written. This compiler prints
`--b: c d;` and `--e: { f: g; };`.

## Where it goes wrong

Not in the parser. Instrumenting
`parse_interpolated_declaration_value` shows the newline surviving into the
AST: for the input above the interpolation is `String(" c\n    d")`. The
loss is at the other end, in `write_style` at
`crates/compiler/src/serializer.rs:1556`, which routes every declaration
through `visit_value`:

```rust
// todo: _writeFoldedValue and _writeReindentedValue
if style.parsed_as_sass_script && !self.options.is_compressed() {
    self.buffer.push(b' ');
}

self.visit_value(&style.value.node, style.value.span)?;
```

`visit_value` reaches `visit_unquoted_string` (line 1395), which folds a
newline and the whitespace after it into a single space. That is correct
for an ordinary unquoted string and wrong for a custom property, and the
`todo` names the two functions dart-sass uses instead.

## What dart-sass does

`visitCssDeclaration` branches on `parsedAsSassScript`, the same flag this
compiler's `Style` already carries:

```dart
if (!node.parsedAsSassScript) {
  if (compressed) _writeFoldedValue(node) else _writeReindentedValue(node);
} else {
  if (!compressed) _buffer.writeCharCode($space);
  node.value.value.accept(this);
}
```

- `_writeFoldedValue` walks the text and replaces each newline, plus every
  whitespace character after it, with one space. That is what compressed
  output wants, and it is roughly what `visit_unquoted_string` does today.
- `_writeReindentedValue` keeps the line structure. It finds the value's
  minimum indentation, then rewrites each line at the output indentation
  plus the difference. Two special cases: a value with no newline is
  written unchanged, and a value whose last line is blank
  (`minimumIndentation == -1`) is right-trimmed and given a trailing space.
  The indentation it re-indents to is the smaller of the value's minimum
  indentation and the column the property name starts at, so a value
  indented less than its own declaration does not gain space.

Both were read out of the dart-sass 1.103.1 package
(`npm pack sass@1.103.1`, `package/sass.dart.js`), not inferred.

## Implementation instructions

Branch in `write_style` on `parsed_as_sass_script` exactly as dart does,
and write the two helpers against the `Value::String` text. The output
indentation is `self.indentation`, and the property's start column comes
from the span the `Style` already carries -- `write_comment` two functions
below already does the same `look_up_pos(span.low()).position.column`
lookup for comments.

Leave `visit_unquoted_string` alone. It is right for values that *are*
SassScript, and item 11's tests depend on it.

## Testing

- Ground truth: `spec/css/custom_properties/{simple,indentation,without_semicolon}.hrx`
  and `spec/css/custom_properties/syntax/sass/multiline_list.hrx` at the
  pinned revision. `indentation.hrx` is the thorough one: it covers a value
  indented below the declaration, hard tabs, and a blank line inside the
  value.
- Scoped spec runs: `spec/css/custom_properties`, `spec/css`, and the
  compressed-output tests -- the folded path is the compressed one, and no
  fixture in the standard run exercises it.
- Add regression tests to `crates/lib/tests/custom-property.rs`, and
  update `silent_comment_with_following_line`, which currently pins the
  folding. Verify its new expectation against the binary first: it is
  exactly the kind of test that gets re-baselined to whatever the new code
  prints.
- Verify every new expectation against dart-sass 1.103.1 with
  `npx -y sass@1.103.1`, never bare `npx sass`.

## Acceptance criteria

- `spec/css/custom_properties` passes: 0 failures, down from 6.
- Compressed output for a multi-line custom property matches dart-sass
  `--style=compressed`, which is the folded path and which no spec fixture
  covers.
- The `frameworks` and `bootstrap` CI jobs still report no colour-value
  differences. They are the check that ordinary single-line values are
  unaffected, which matters here because this change touches how *every*
  declaration is written, not only the multi-line ones.
