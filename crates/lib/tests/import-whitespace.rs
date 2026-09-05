//! Whitespace in `@import` modifiers, and the trailing semicolon the indented
//! syntax tolerates.
//!
//! Every expectation here was compared against the dart-sass 1.103.1 binary
//! before being committed.

use accent_sass::InputSyntax;

#[macro_use]
mod macros;

// The whole `supports(...)` query sits inside parentheses, so whitespace --
// including a newline in the indented syntax -- may fall anywhere in it.
test!(
    supports_newline_after_open_paren,
    "@import \"a.css\" supports(\n  a: b)\n",
    "@import \"a.css\" supports(a: b);\n"
);
test!(
    supports_space_after_open_paren,
    "@import \"a.css\" supports( a: b)\n",
    "@import \"a.css\" supports(a: b);\n"
);
test!(
    indented_supports_newline_after_open_paren,
    "@import \"a.css\" supports(\n  a: b)\n",
    "@import \"a.css\" supports(a: b);\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);
test!(
    indented_supports_newline_after_key,
    "@import \"a.css\" supports(a\n  : b)\n",
    "@import \"a.css\" supports(a: b);\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);
test!(
    indented_supports_newline_after_colon,
    "@import \"a.css\" supports(a:\n  b)\n",
    "@import \"a.css\" supports(a: b);\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);
test!(
    indented_supports_newline_before_and,
    "@import \"a.css\" supports((a: b) \n  and (c: d))\n",
    "@import \"a.css\" supports((a: b) and (c: d));\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);
test!(
    indented_supports_newline_after_not,
    "@import \"a.css\" supports(not\n  (a: b))\n",
    "@import \"a.css\" supports(not (a: b));\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);
test!(
    indented_supports_newline_inside_a_function,
    "@import \"a.css\" supports(a(\n  b))\n",
    "@import \"a.css\" supports(a( b));\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);

// A modifier that is not `supports()` gets the same treatment; its arguments
// are kept as written.
test!(
    indented_modifier_arguments_span_lines,
    "@import \"a\" b(\n  c)\n",
    "@import \"a\" b(\n  c);\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);

// The indented syntax ends a statement at the newline but tolerates a `;`
// before it.
test!(
    indented_import_trailing_semicolon,
    "@import \"a.css\";\n",
    "@import \"a.css\";\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);
test!(
    indented_import_modifier_trailing_semicolon,
    "@import \"a.css\" supports(calc(1));\n",
    "@import \"a.css\" supports(calc(1));\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);
test!(
    indented_declaration_trailing_semicolon,
    "a\n  b: c;\n",
    "a {\n  b: c;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);

// What the `;` does not buy is a second statement after it.
error!(
    indented_two_statements_on_one_line,
    "a\n  b: c; d: e\n",
    "Error: multiple statements on one line are not supported in the indented syntax.",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);

// `@import` now ends with a statement separator, so an indented block beneath
// it is named as the error dart-sass gives.
error!(
    indented_block_beneath_an_import,
    "@import \"a.css\"\n  print\n",
    "Error: Nothing may be indented beneath a @import rule.",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);
