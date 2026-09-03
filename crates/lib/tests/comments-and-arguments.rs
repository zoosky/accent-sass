//! Comment positions and argument-list syntax.
//!
//! Every expectation here was compared against the dart-sass 1.103.1 binary
//! before being committed.

#[macro_use]
mod macros;

// A trailing comma is allowed after a rest argument and after a rest
// parameter, not just after ordinary ones.
test!(
    trailing_comma_after_keyword_rest_argument,
    "@use \"sass:meta\";\n@mixin m($args...) {\n  b: meta.inspect(meta.keywords($args));\n}\na {\n  @include m(1, (c: 2)..., );\n}\n",
    "a {\n  b: (c: 2);\n}\n"
);
test!(
    trailing_comma_after_rest_parameter_of_mixin,
    "@mixin m($b..., ) {\n  c: $b;\n}\na {\n  @include m(1, 2);\n}\n",
    "a {\n  c: 1, 2;\n}\n"
);
test!(
    trailing_comma_after_rest_parameter_of_function,
    "@function f($g..., ) {\n  @return $g;\n}\na {\n  b: f(3, 4);\n}\n",
    "a {\n  b: 3, 4;\n}\n"
);

// An argument list is a comma-separated list, so it needs parentheses when it
// is a map value -- otherwise the map reads as having extra entries.
test!(
    argument_list_in_map_is_parenthesized,
    "@use \"sass:meta\";\n@function f($args...) {\n  @return meta.inspect((positional: $args));\n}\na {\n  b: f(1, 2);\n}\n",
    "a {\n  b: (positional: (1, 2));\n}\n"
);

// A comment may sit between the end of a rule's value and its `;`.
test!(
    comment_after_extend_flag,
    "a {\n  b: c;\n}\nd {\n  @extend a !optional /**/;\n}\n",
    "a, d {\n  b: c;\n}\n"
);
test!(
    silent_comment_after_content_arguments,
    "@mixin a {\n  @content() //\n  ;\n}\nb {\n  @include a {c: d};\n}\n",
    "b {\n  c: d;\n}\n"
);

// In the indented syntax a newline ends a statement, except inside
// parentheses, where an argument list may be broken across lines.
test!(
    indented_newline_in_parameter_list,
    "@function a(\n  $b, $c)\n  @return $b\n\nd\n  e: a(1, 2)\n",
    "d {\n  e: 1;\n}\n",
    grass::Options::default().input_syntax(grass::InputSyntax::Sass)
);
test!(
    indented_newline_in_argument_list,
    "@function a($b, $c)\n  @return $b\n\nd\n  e: a(\n    1, 2)\n",
    "d {\n  e: 1;\n}\n",
    grass::Options::default().input_syntax(grass::InputSyntax::Sass)
);
test!(
    indented_newline_in_include_arguments,
    "@mixin f($g,\n  $h)\n  i: $g\n\nj\n  @include f(1,\n    2)\n",
    "j {\n  i: 1;\n}\n",
    grass::Options::default().input_syntax(grass::InputSyntax::Sass)
);
