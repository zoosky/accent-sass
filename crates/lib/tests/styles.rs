#[macro_use]
mod macros;

test!(
    basic_style,
    "a {\n  color: red;\n}\n",
    "a {\n  color: red;\n}\n"
);
test!(
    two_styles,
    "a {\n  color: red;\n  color: blue;\n}\n",
    "a {\n  color: red;\n  color: blue;\n}\n"
);
test!(
    two_inner_rulesets,
    "a {\n  b {\n  color: red;\n}\n  c {\n  color: white;\n}\n}\n",
    "a b {\n  color: red;\n}\na c {\n  color: white;\n}\n"
);
test!(
    two_rulesets,
    "a {\n  color: red;\n}\n\nc {\n  color: white;\n}\n",
    "a {\n  color: red;\n}\n\nc {\n  color: white;\n}\n"
);
test!(
    two_rulesets_first_no_semicolon,
    "a {\n  color: red\n}\nc {\n  color: white;\n}\n",
    "a {\n  color: red;\n}\n\nc {\n  color: white;\n}\n"
);
test!(
    two_inner_outer_rulesets,
    "a {\n  b {\n  color: red;\n}\n  c {\n  color: white;\n}\n}\na {\n  b {\n  color: red;\n}\n  c {\n  color: white;\n}\n}\n",
    "a b {\n  color: red;\n}\na c {\n  color: white;\n}\n\na b {\n  color: red;\n}\na c {\n  color: white;\n}\n"
);
test!(
    removes_empty_outer_styles,
    "a {\n  b {\n    color: red;\n    }\n  }\n",
    "a b {\n  color: red;\n}\n"
);
error!(
    removes_empty_outer_styles_missing_closing_curly_brace,
    "a {\n  b {\n    color: red;\n  }\n", "Error: expected \"}\"."
);
test!(removes_empty_styles, "a {}\n", "");
test!(
    doesnt_eat_style_after_ruleset, // Expectation corrected against dart-sass 1.103.1: a declaration written
    // after a nested rule stays after it, so the parent rule splits.
    "a {\n  b {\n  color: red;\n}\n  color: blue;\n}\n",
    "a b {\n  color: red;\n}\na {\n  color: blue;\n}\n"
);
test!(
    multiline_style,
    "a {\n  color: red\n  blue;\n}\n",
    "a {\n  color: red blue;\n}\n"
);
test!(
    hyphenated_style_property,
    "a {\n  font-family: Arial;\n}\n",
    "a {\n  font-family: Arial;\n}\n"
);
test!(
    hyphenated_style_value,
    "a {\n  color: Open-Sans;\n}\n",
    "a {\n  color: Open-Sans;\n}\n"
);
test!(
    space_separated_style_value,
    "a {\n  border: solid red;\n}\n",
    "a {\n  border: solid red;\n}\n"
);
test!(
    single_quoted_style_value,
    "a {\n  font: 'Open-Sans';\n}\n",
    "a {\n  font: \"Open-Sans\";\n}\n"
);
test!(
    double_quoted_style_value,
    "a {\n  font: \"Open-Sans\";\n}\n",
    "a {\n  font: \"Open-Sans\";\n}\n"
);
test!(
    comma_style_value,
    "a {\n  font: Open-Sans, sans-serif;\n}\n",
    "a {\n  font: Open-Sans, sans-serif;\n}\n"
);
test!(
    style_interpolation_start,
    "a {\n  #{c}olor: red;\n}\n",
    "a {\n  color: red;\n}\n"
);
test!(
    style_interpolation_middle,
    "a {\n  co#{l}or: red;\n}\n",
    "a {\n  color: red;\n}\n"
);
test!(
    style_interpolation_end,
    "a {\n  colo#{r}: red;\n}\n",
    "a {\n  color: red;\n}\n"
);
test!(
    style_interpolation_variable,
    "$a: foo;\na {\n  co#{$a}lor: red;\n}\n",
    "a {\n  cofoolor: red;\n}\n"
);
test!(
    style_val_interpolation_start,
    "a {\n  color: #{r}ed;\n}\n",
    "a {\n  color: red;\n}\n"
);
test!(
    style_val_interpolation_middle,
    "a {\n  color: r#{e}d;\n}\n",
    "a {\n  color: red;\n}\n"
);
test!(
    style_val_interpolation_end,
    "a {\n  color: re#{d};\n}\n",
    "a {\n  color: red;\n}\n"
);
test!(
    style_val_interpolation_variable,
    "$a: foo;\na {\n  color: r#{$a}ed;\n}\n",
    "a {\n  color: rfooed;\n}\n"
);
test!(
    style_whitespace,
    "a {\n     color      :       red    ;    \n}\n",
    "a {\n  color: red;\n}\n"
);
test!(
    triple_nested_preceding_ruleset,
    "a {\n  b {\n    foo: bar;\n    c {}\n  }\n}\n",
    "a b {\n  foo: bar;\n}\n"
);
test!(
    triple_nested_following_ruleset,
    "a {\n  b {\n    c {}\n    foo: bar;\n  }\n}\n",
    "a b {\n  foo: bar;\n}\n"
);
test!(
    single_nested_styles,
    "a {\n  webkit: {\n    color: red;\n    color: orange\n  }\n}\n",
    "a {\n  webkit-color: red;\n  webkit-color: orange;\n}\n"
);
test!(
    multiple_nested_styles,
    "a {\n  webkit: {\n    webkit: {\n     color: red;\n    }\n  }\n}\n",
    "a {\n  webkit-webkit-color: red;\n}\n"
);
test!(
    no_space_after_colon_before_nested_style,
    "a {\n    foo:{\n        bar: baz\n    }\n}\n",
    "a {\n  foo-bar: baz;\n}\n"
);
test!(
    no_space_between_colon,
    "a {\n  color:red;\n}\n",
    "a {\n  color: red;\n}\n"
);
test!(
    no_space_between_colon_no_semicolon,
    "a {\n  color:red\n}\n",
    "a {\n  color: red;\n}\n"
);
test!(removes_null_value, "a {\n  color: null;\n}\n", "");
test!(
    namespace_before_open_brace,
    "foo {\n  a: b {\n    c: d;\n  }\n}\n",
    "foo {\n  a: b;\n  a-c: d;\n}\n"
);
test!(
    namespace_before_open_brace_nested,
    "foo {\n  a: b {\n    c: d {\n      e: f;\n    }\n  }\n}\n",
    "foo {\n  a: b;\n  a-c: d;\n  a-c-e: f;\n}\n"
);
test!(
    curly_braces_in_quotes,
    "a {\n  color: \"{foo}\";\n}\n",
    "a {\n  color: \"{foo}\";\n}\n"
);
test!(
    escaped_interpolation,
    "a {\n  color: \"\\#{foo}\";\n}\n",
    "a {\n  color: \"#{foo}\";\n}\n"
);
test!(
    styles_after_quoted,
    "a {\n  color: \"red\";\n  color: blue;\n}\n",
    "a {\n  color: \"red\";\n  color: blue;\n}\n"
);
test!(
    emits_leading_whitespace,
    "a {\n  color: unquote(\" foo\");\n}\n",
    "a {\n  color:  foo;\n}\n"
);
test!(
    emits_trailing_whitespace,
    "a {\n  color: unquote(\"foo  \");\n}\n",
    "a {\n  color: foo  ;\n}\n"
);
test!(
    multiline_comment_after_style_property,
    "a {\n  color  /**/  : red;\n}\n",
    "a {\n  color: red;\n}\n"
);
test!(
    style_begins_with_asterisk_without_whitespace,
    "a {\n  *zoom: 1;\n}\n",
    "a {\n  *zoom: 1;\n}\n"
);
test!(
    style_begins_with_asterisk_with_whitespace,
    "a {\n  *   zoom: 1;\n}\n",
    "a {\n  *   zoom: 1;\n}\n"
);
test!(
    style_begins_with_asterisk_with_newline,
    "a {\n  * \n  zoom: 1;\n}\n",
    "a {\n  * \n  zoom: 1;\n}\n"
);
test!(
    no_newline_after_child_ruleset_ends_with_silent_child,
    "a {
        position: relative;

        b {}
    }

    c {
        white-space: nowrap;
    }",
    "a {\n  position: relative;\n}\nc {\n  white-space: nowrap;\n}\n"
);
test!(
    symbol_before_property_name_hacks,
    "a {
        .color: foo;
        #color: foo;
        :color: foo;
        *color: foo;
        .--color: foo;
        #--color: foo;
        :--color: foo;
        *--color: foo;
    }",
    "a {\n  .color: foo;\n  #color: foo;\n  :color: foo;\n  *color: foo;\n  .--color: foo;\n  #--color: foo;\n  :--color: foo;\n  *--color: foo;\n}\n"
);
error!(
    media_inside_nested_declaration,
    "a {
        color: {
            @media foo {}
        }
    }",
    "Error: This at-rule is not allowed here."
);
error!(
    media_inside_nested_declaration_from_mixin,
    "@mixin foo() {
        @media foo {}
    }

    a {
        color: {
            @include foo();
        }
    }",
    "Error: Media rules may not be used within nested declarations."
);
error!(
    ruleset_inside_nested_declaration_from_mixin,
    "@mixin foo() {
        a {}
    }

    a {
        color: {
            @include foo();
        }
    }",
    "Error: Style rules may not be used within nested declarations."
);
error!(
    style_at_the_toplevel_from_mixin,
    "@mixin foo() {
        color: red;
    }

    @include foo();",
    "Error: Declarations may only be used within style rules."
);

// Source order across a nested rule. dart-sass splits the parent rule so a
// declaration written after a nested rule is emitted after it; this compiler
// used to hoist it back up beside the earlier declarations, which changes the
// cascade when both set the same property. Every expectation below was checked
// against dart-sass 1.103.1.
test!(
    declaration_after_nested_rule_splits_parent,
    ".a {\n  b: c;\n  .d {e: f}\n  g: h;\n}\n",
    ".a {\n  b: c;\n}\n.a .d {\n  e: f;\n}\n.a {\n  g: h;\n}\n"
);
test!(
    declaration_after_nested_rule_keeps_cascade_order,
    ".a {\n  color: red;\n  .b {color: blue}\n  color: green;\n}\n",
    ".a {\n  color: red;\n}\n.a .b {\n  color: blue;\n}\n.a {\n  color: green;\n}\n"
);
test!(
    declaration_before_nested_rule_does_not_split,
    ".a {\n  b: c;\n  d: e;\n  .f {g: h}\n}\n",
    ".a {\n  b: c;\n  d: e;\n}\n.a .f {\n  g: h;\n}\n"
);
test!(
    two_nested_rules_between_declarations,
    ".a {\n  b: c;\n  .d {e: f}\n  .g {h: i}\n  j: k;\n}\n",
    ".a {\n  b: c;\n}\n.a .d {\n  e: f;\n}\n.a .g {\n  h: i;\n}\n.a {\n  j: k;\n}\n"
);
