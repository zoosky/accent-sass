//! Two plain CSS gaps: the CSS `if()` function, and `//` inside a value.
//!
//! Every expectation here was compared against the dart-sass 1.103.1 binary
//! before being committed.

use accent_sass::InputSyntax;

#[macro_use]
mod macros;

// There are no silent comments in plain CSS, so `//` in a value is two
// slashes. The parser used to look for a comment there and reject the file.
test!(
    slash_slash_in_a_value,
    "a {b: 1///bar}\n",
    "a {\n  b: 1///bar;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    separated_slashes_in_a_value,
    "a {b: 1/ / /bar}\n",
    "a {\n  b: 1///bar;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    slashes_with_intermediate_values,
    "a {b: 1/2/foo/bar}\n",
    "a {\n  b: 1/2/foo/bar;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);

// Outside a value, `//` is still the comment plain CSS forbids.
error!(
    silent_comment_at_statement_level,
    "// c\na {b: c}\n",
    "Error: Silent comments aren't allowed in plain CSS.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);

// `if()` with CSS-style conditions works in a `.css` file too. The plain CSS
// parser did not reach that code path and stopped at the first `:`.
test!(
    css_if,
    "a {b: if(css(1): c; css(2): d; else: e)}\n",
    "a {\n  b: if(css(1): c; css(2): d; else: e);\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);

// A `sass()` condition is settled at compile time, which a `.css` file has no
// business doing.
error!(
    css_if_rejects_a_sass_condition,
    "a {b: if(sass(true): c)}\n",
    "Error: sass() conditions aren't allowed in plain CSS",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    css_if_rejects_a_negated_sass_condition,
    "a {b: if(not sass(true): c)}\n",
    "Error: sass() conditions aren't allowed in plain CSS",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    css_if_rejects_a_parenthesized_sass_condition,
    "a {b: if((sass(true)): c)}\n",
    "Error: sass() conditions aren't allowed in plain CSS",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);

// There is no legacy Sass `if()` to fall back to in plain CSS, so a
// comma-separated argument list fails on its first argument.
error!(
    legacy_if_call,
    "a {b: if(true, c, d)}\n",
    "Error: expected \"(\".",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
