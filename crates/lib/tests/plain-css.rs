use accent_sass::InputSyntax;

#[macro_use]
mod macros;

test!(
    function_call,
    "a {
        color: rotate(-45deg);
    }",
    "a {\n  color: rotate(-45deg);\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    retains_null,
    "a {
        color: null;
    }",
    "a {\n  color: null;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    does_not_evaluate_and,
    "a {
        color: 1 and 2;
    }",
    "a {\n  color: 1 and 2;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    does_not_evaluate_or,
    "a {
        color: 1 or 2;
    }",
    "a {\n  color: 1 or 2;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    simple_calculation,
    "a {
        color: calc(1 + 1);
    }",
    "a {\n  color: 2;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    simple_url_import,
    r#"@import url("foo");"#,
    "@import url(\"foo\");\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    import_no_file_extension,
    r#"@import "foo";"#,
    "@import \"foo\";\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    import_with_condition,
    r#"@import "foo" screen and (foo: bar);"#,
    "@import \"foo\" screen and (foo: bar);\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    does_not_evaluate_not,
    "a {
        color: not 2;
        color: not true;
        color: not false;
    }",
    "a {\n  color: not 2;\n  color: not true;\n  color: not false;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    denies_silent_comment,
    "// silent",
    "Error: Silent comments aren't allowed in plain CSS.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    denies_function_rule,
    "@function foo() {
        @return 2;
    }",
    "Error: This at-rule isn't allowed in plain CSS.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    denies_content_rule,
    "@content",
    "Error: This at-rule isn't allowed in plain CSS.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    allows_media_rule,
    "@media (foo) {
        a {
            color: red;
        }
    }",
    "@media (foo) {\n  a {\n    color: red;\n  }\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    allows_var_empty_second_arg,
    "a {
        color: var(1, );
    }",
    "a {\n  color: var(1, );\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    disallows_empty_second_arg_in_non_var_function,
    "a {
        color: foo(1, );
    }",
    "Error: Expected expression.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
// `if()` in plain CSS is the CSS function, which takes conditions rather than
// a comma-separated argument list, so the legacy Sass call fails on its first
// argument. dart-sass 1.103.1 raises the same error at the same point.
error!(
    disallows_if_function,
    "a {
        color: if(true, a, b);
    }",
    "Error: expected \"(\".",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    disallows_map_get_function,
    "a {
        color: map-get(true, a, b);
    }",
    "Error: This function isn't allowed in plain CSS.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    disallows_plus_operator,
    "a {
        color: 1 + 2;
    }",
    "Error: Operators aren't allowed in plain CSS.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    disallows_parens,
    "a {
        color: (a b);
    }",
    "Error: Parentheses aren't allowed in plain CSS.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    disallows_variable_expr,
    "a {
        color: $a;
    }",
    "Error: Sass variables aren't allowed in plain CSS.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    disallows_variable_decl,
    "$bar: red;",
    "Error: Sass variables aren't allowed in plain CSS.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    disallows_parent_selector_expr,
    "a {
        color: &;
    }",
    "Error: The parent selector isn't allowed in plain CSS.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    disallows_unary_plus,
    "a {
        color: +(1);
    }",
    "Error: Operators aren't allowed in plain CSS.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    disallows_unary_minus,
    "a {
        color: -(1);
    }",
    "Error: Operators aren't allowed in plain CSS.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    disallows_interpolation,
    "a {
        color: a#{b}c;
    }",
    "Error: Interpolation isn't allowed in plain CSS.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    disallows_placeholder_selector,
    "%a {
        color: red;
    }",
    "Error: Placeholder selectors aren't allowed in plain CSS.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    allows_rgb_function,
    "a {
        color: rgb(true, a, b);
    }",
    "a {\n  color: rgb(true, a, b);\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    simple_supports,
    "@supports (foo) {
        a {
            color: red;
        }
    }",
    "@supports (foo) {\n  a {\n    color: red;\n  }\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    custom_property,
    "a {
        --foo: /* */;
    }",
    "a {\n  --foo: /* */;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    single_nested_property,
    "a {
        b: {
            c: d;
        }
    }",
    "Error: Nested declarations aren't allowed in plain CSS.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    single_nested_property_with_expression,
    "a {
        b: 2 {
            c: d;
        }
    }",
    "Error: Nested declarations aren't allowed in plain CSS.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);

// CSS nesting is passed through: a rule written inside another rule keeps its
// own selector and stays nested, because the browser resolves it, not Sass.
// Every expectation below was checked against dart-sass 1.103.1.
test!(
    nesting_one_level,
    "a {b {c: d}}",
    "a {\n  b {\n    c: d;\n  }\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    nesting_two_levels,
    "a {b {c {d: e}}}",
    "a {\n  b {\n    c {\n      d: e;\n    }\n  }\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    nesting_multiple_complex,
    "a, b {c, d {e: f}}",
    "a, b {\n  c, d {\n    e: f;\n  }\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    nesting_between_declarations,
    "a {b: c; d {e: f} g: h}",
    "a {\n  b: c;\n  d {\n    e: f;\n  }\n  g: h;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    nesting_leading_combinator,
    "a {+ b {c: d}}",
    "a {\n  + b {\n    c: d;\n  }\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);

// `&` is the CSS nesting selector rather than Sass's parent selector, so it is
// written out unresolved wherever it appears -- including at the top level.
test!(
    parent_selector_alone,
    "a {& {b: c}}",
    "a {\n  & {\n    b: c;\n  }\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    parent_selector_mid_compound,
    "a {.b&.c {d: e}}",
    "a {\n  .b&.c {\n    d: e;\n  }\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    top_level_parent_selector,
    "& {a: b}",
    "& {\n  a: b;\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    parent_selector_with_suffix,
    "a {&b {c: d}}",
    "Error: Parent selectors can't have suffixes in plain CSS.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);

// An at-rule directly inside the outermost rule still bubbles out of it, the
// way it does in Sass. Once nesting has been passed through, it stays where it
// was written: the stylesheet already requires a browser that supports nesting.
test!(
    at_rule_bubbles_out_of_outermost_rule,
    "a {@media b {c: d}}",
    "@media b {\n  a {\n    c: d;\n  }\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    media_stays_inside_nested_rule,
    "a {b {@media c {d: e}}}",
    "a {\n  b {\n    @media c {\n      d: e;\n    }\n  }\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    nested_media_queries_are_not_merged,
    "a {b {@media c {@media (d) {e: f}}}}",
    "a {\n  b {\n    @media c {\n      @media (d) {\n        e: f;\n      }\n    }\n  }\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    supports_stays_inside_nested_rule,
    "a {b {@supports (c: d) {e: f}}}",
    "a {\n  b {\n    @supports (c: d) {\n      e: f;\n    }\n  }\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
test!(
    unknown_at_rule_stays_inside_nested_rule,
    "a {b {@c {d: e}}}",
    "a {\n  b {\n    @c {\n      d: e;\n    }\n  }\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
// Bubbling resumes for a sibling written after the nested rule closes.
test!(
    at_rule_after_nested_rule_still_bubbles,
    "a {b {c: d} @media e {f: g}}",
    "a {\n  b {\n    c: d;\n  }\n}\n@media e {\n  a {\n    f: g;\n  }\n}\n",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);

// A combinator needs a rule to be relative to. Nested rules have one; the
// top level does not, and nothing follows a trailing combinator anywhere.
error!(
    top_level_leading_combinator,
    "> a {b: c}",
    "Error: Top-level leading combinators aren't allowed in plain CSS.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    trailing_combinator_without_nesting,
    "a > {b: c}",
    "Error: expected selector.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    trailing_combinator_with_nesting,
    "a > {b {c: d}}",
    "Error: expected selector.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
error!(
    placeholder_selector_in_nested_rule,
    "a {b {%c {d: e}}}",
    "Error: Placeholder selectors aren't allowed in plain CSS.",
    accent_sass::Options::default().input_syntax(InputSyntax::Css)
);
