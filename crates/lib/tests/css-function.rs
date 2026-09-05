//! The plain CSS `@function` rule.
//!
//! `@function` declares a Sass function unless its name begins with `--`, in
//! which case it declares a CSS custom function that Sass passes through
//! untouched. Every expectation here was compared against the dart-sass 1.103.1
//! binary before being committed.

use accent_sass::InputSyntax;

#[macro_use]
mod macros;

// The rule is written out as it was spelled, parameters and all.
test!(
    passthrough,
    "@function --a(--b <color>) {result: c}\n",
    "@function --a(--b <color>) {\n  result: c;\n}\n"
);
test!(
    return_type,
    "@function --a() returns <ident> {result: b}\n",
    "@function --a() returns <ident> {\n  result: b;\n}\n"
);
test!(
    interpolation_in_name,
    "@function --#{a}() {result: b}\n",
    "@function --a() {\n  result: b;\n}\n"
);
test!(
    body_may_nest_a_style_rule,
    "@function --a() {b {c: d}}\n",
    "@function --a() {\n  b {\n    c: d;\n  }\n}\n"
);
test!(without_a_block, "@function --a();\n", "@function --a();\n");

// The `result` descriptor takes its value verbatim, the way a custom property
// does: SassScript in it is text, but interpolation still resolves.
test!(
    result_is_not_sass_script,
    "@function --a() {\n  result: $b;\n}\n",
    "@function --a() {\n  result: $b;\n}\n"
);
test!(
    result_takes_any_characters,
    "@function --a() {\n  result: {}#&%^*;\n}\n",
    "@function --a() {\n  result: {}#&%^*;\n}\n"
);
test!(
    result_resolves_interpolation,
    "@function --a() {\n  result: #{1 + 1};\n}\n",
    "@function --a() {\n  result: 2;\n}\n"
);

// Both the rule name and the descriptor match case-insensitively, and both keep
// the case they were written in.
test!(
    uppercase_rule_name,
    "@FUNCTION --a() {\n  result: $b;\n}\n",
    "@FUNCTION --a() {\n  result: $b;\n}\n"
);
test!(
    uppercase_result,
    "@function --a() {\n  RESULT: $b;\n}\n",
    "@function --a() {\n  RESULT: $b;\n}\n"
);

// An interpolated name -- of the rule or of the descriptor -- is not plain at
// parse time, so neither reaches the verbatim path and the value is SassScript.
test!(
    interpolated_rule_name_leaves_result_as_sass_script,
    "@#{function} --a() {\n  result: 1 + 1;\n}\n",
    "@function --a() {\n  result: 2;\n}\n"
);
test!(
    interpolated_result_name_is_sass_script,
    "@function --a() {\n  #{result}: 1 + 1;\n}\n",
    "@function --a() {\n  result: 2;\n}\n"
);

// `result` is only special inside the rule; elsewhere it is an ordinary
// property.
test!(
    result_in_a_style_rule_is_sass_script,
    ".a {\n  result: 1 + 1;\n}\n",
    ".a {\n  result: 2;\n}\n"
);

// The body is plain CSS, so Sass at-rules have no place in it.
error!(
    return_is_not_allowed_in_the_body,
    "@function --a() {@return 1}\n", "Error: This at-rule is not allowed here."
);

// A `--`-prefixed call names a CSS custom function and never a Sass one, even
// though identifier normalisation makes `__a` and `--a` the same name.
test!(
    custom_ident_call_does_not_reach_a_sass_function,
    "@function __a() {@return 1}\nb {c: --a()}\n",
    "b {\n  c: --a();\n}\n"
);
test!(
    a_sass_function_is_still_declared,
    "@function a() {@return 1}\nb {c: a()}\n",
    "b {\n  c: 1;\n}\n"
);

// In a `.css` file the rule exists only in its CSS form.
test!(
    plain_css_passthrough,
    "@function --a(--b <color>) {result: c}\n",
    "@function --a(--b <color>) {\n  result: c;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    plain_css_result_is_not_sass_script,
    "@function --a() {\n  result: $b;\n}\n",
    "@function --a() {\n  result: $b;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    plain_css_rejects_a_sass_function,
    "@function a() {result: b}\n",
    "Error: This at-rule isn't allowed in plain CSS.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);

// The indented syntax cannot nest anything beneath `result`, because the value
// runs to the end of the line.
error!(
    indented_result_may_not_be_nested,
    "@function --a()\n  result:\n    b: c\n",
    "Error: Nothing may be indented beneath a @function result.",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);
test!(
    indented_interpolated_result_may_be_nested,
    "@function --a()\n  #{result}:\n    b: c\n",
    "@function --a() {\n  result-b: c;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Sass)
);
