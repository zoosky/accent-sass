//! Tests for where a newline is whitespace in the indented syntax.
//!
//! A newline ends a statement in `.sass`, except at a position where a
//! statement cannot end -- dart-sass calls that the `consumeNewlines`
//! parameter, and it is decided per call site rather than per region. These
//! tests pin both sides: the headers that may be split across lines, and the
//! ones that may not. Every expectation was checked against dart-sass 1.103.1.

use accent_sass::InputSyntax;

#[macro_use]
mod macros;

test!(
    function_rule_name_on_next_line,
    "@function
  a()
  @return 1

b
  c: a()
",
    "b {\n  c: 1;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);
test!(
    function_rule_arguments_on_next_line,
    "@function a
  ()
  @return 2

b
  c: a()
",
    "b {\n  c: 2;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);
test!(
    for_rule_variable_on_next_line,
    "@for
  $i from 1 through 2
  a
    b: $i
",
    "a {\n  b: 1;\n}\n\na {\n  b: 2;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);
test!(
    for_rule_through_on_next_line,
    "@for $i from 1
  through 2
  a
    b: $i
",
    "a {\n  b: 1;\n}\n\na {\n  b: 2;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);
test!(
    for_rule_silent_comment_after_from,
    "@for $i from //
  1 through 2
  a
    b: $i
",
    "a {\n  b: 1;\n}\n\na {\n  b: 2;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);
test!(
    each_rule_variable_on_next_line,
    "@each
  $i in 1, 2
  a
    b: $i
",
    "a {\n  b: 1;\n}\n\na {\n  b: 2;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);
test!(
    mixin_and_include_names_on_next_line,
    "@mixin
  m($a)
  b: $a

a
  @include
    m(1)
",
    "a {\n  b: 1;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);

// The other half: rules whose header may not be split, because a statement can
// end where the newline falls.
error!(
    media_query_may_not_start_on_next_line,
    "@media
  screen
  a
    b: c
",
    "Error: Expected identifier.",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);
error!(
    supports_condition_may_not_start_on_next_line,
    "@supports
  (color: red)
  a
    b: c
",
    r#"Error: expected "("."#,
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);
