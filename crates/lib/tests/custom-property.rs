#[macro_use]
mod macros;

test!(
    interpolated_null_is_removed,
    "a {\n  --btn-font-family: #{null};\n}\n",
    "a {\n  --btn-font-family: ;\n}\n"
);
test!(
    no_space_after_colon,
    "a {\n  --btn-font-family:null;\n}\n",
    "a {\n  --btn-font-family:null;\n}\n"
);
test!(
    only_whitespace,
    "a {\n  --btn-font-family: ;\n}\n",
    "a {\n  --btn-font-family: ;\n}\n"
);
test!(
    silent_comment,
    "a {\n  --btn-font-family: // ;\n}\n",
    "a {\n  --btn-font-family: // ;\n}\n"
);
test!(
    interpolated_name_isnt_custom_property,
    "a {\n  #{--prop}:0.75;\n}\n",
    "a {\n  --prop: 0.75;\n}\n"
);
test!(
    interpolated_name_is_custom_property_if_dashes_not_part_of_interpolation,
    "a {\n  --#{prop}:0.75;\n}\n",
    "a {\n  --prop:0.75;\n}\n"
);
test!(
    prop_value_starts_with_u,
    "a {\n  --prop: underline;\n}\n",
    "a {\n  --prop: underline;\n}\n"
);
test!(
    prop_value_is_url,
    "a {\n  --prop: url();\n}\n",
    "a {\n  --prop: url();\n}\n"
);
test!(
    prop_value_starts_with_url,
    "a {\n  --prop: urlaa;\n}\n",
    "a {\n  --prop: urlaa;\n}\n"
);
test!(
    prop_value_is_url_without_parens,
    "a {\n  --prop: url;\n}\n",
    "a {\n  --prop: url;\n}\n"
);
test!(
    #[ignore = "we don't preserve newlines or indentation here"]
    preserves_newlines_in_value,
    "a {\n    --without-semicolon: {\n        a: b\n    }\n}\n",
    "a {\n  --without-semicolon: {\n      a: b\n  } ;\n}\n"
);
// dart-sass 1.103.1 allows a custom property to have an empty value, per the
// CSS spec; it used to be an error.
test!(
    nothing_after_colon,
    "a {\n  --btn-font-family:;\n}\n",
    "a {\n  --btn-font-family:;\n}\n"
);
error!(
    #[ignore = "dart-sass crashes on this input https://github.com/sass/dart-sass/issues/1857"]
    child_in_declaration_block_is_custom_property,
    "a {
        color: {
            --foo: bar;
        }
    }",
    ""
);
error!(
    // NOTE: https://github.com/sass/dart-sass/issues/1857
    child_in_declaration_block_is_declaration_block_with_child_custom_property,
    "a {
        color: {
            --a: {
                foo: bar;
            }
        }
    }",
    r#"Error: Declarations whose names begin with "--" may not be nested."#
);
error!(
    // NOTE: https://github.com/sass/dart-sass/issues/1857
    custom_property_double_nested_custom_property_with_non_string_value_before_decl_block,
    "a {
        color: {
            --a: 2 {
                --foo: bar;
            }
        }
    }",
    r#"Error: Declarations whose names begin with "--" may not be nested."#
);
// As above: an interpolation that resolves to nothing leaves the custom
// property with an empty value rather than raising an error.
test!(
    empty_value,
    "a {
        --color:#{null};
    }",
    "a {\n  --color:;\n}\n"
);
test!(
    // The declared value keeps its slashes, which is the contrast the
    // `supports_custom_property_drops_a_comment` test draws.
    //
    // The newline is where this deliberately differs from dart-sass 1.103.1,
    // which preserves it (`--a: b //c\n  d;`) because a custom property's
    // value is raw text. This compiler collapses it to a space. The
    // difference predates the silent-comment work -- a build of `dc4d59b^1`
    // prints the same. The pinned spec revision covers it in
    // `spec/css/custom_properties/{simple,indentation}.hrx`, both of which
    // fail; this test pins the shape locally so a change to it shows up in
    // `cargo test` rather than only in the advisory spec tallies.
    silent_comment_with_following_line,
    "e {--a: b //c\n d}\n",
    "e {\n  --a: b //c d;\n}\n"
);

// A custom property's value is raw text, which cannot be spliced into an
// enclosing declaration's name, so a `--` name nested beneath another
// declaration is an error whatever follows the colon. An ordinary
// declaration nested there is fine, and a `--` name at the top of a style
// rule is an ordinary custom property. dart-sass 1.103.1 produced every
// expectation; the fixtures are
// `spec/css/propset/error/custom_property/{simple,nested/complex}`.
error!(
    custom_property_nested_in_a_declaration_is_an_error,
    "a { b: { --d: e } }\n",
    r#"Error: Declarations whose names begin with "--" may not be nested."#
);
error!(
    custom_property_nested_in_a_declaration_with_a_block_is_an_error,
    "a { b: { --d: e {--f: g} } }\n",
    r#"Error: Declarations whose names begin with "--" may not be nested."#
);
// It reaches the visitor too, when the declaration arrives through a mixin
// rather than being written in place.
error!(
    custom_property_included_into_a_declaration_is_an_error,
    "@mixin m { --a: b }\nc { d: { @include m } }\n",
    r#"Error: Declarations whose names begin with "--" may not be nested."#
);
test!(
    an_ordinary_declaration_may_still_be_nested,
    "a { b: { c: d } }\n",
    "a {\n  b-c: d;\n}\n"
);
test!(
    a_custom_property_at_the_top_of_a_rule_is_unaffected,
    "a { --d: {e: f} }\n",
    "a {\n  --d: {e: f} ;\n}\n"
);
