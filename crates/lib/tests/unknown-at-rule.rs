#[macro_use]
mod macros;

test!(
    basic_unknown_at_rule,
    "@foo {\n  a {\n    color: red;\n  }\n}\n",
    "@foo {\n  a {\n    color: red;\n  }\n}\n"
);
test!(
    unknown_at_rule_no_selector,
    "@foo {\n  color: red;\n}\n",
    "@foo {\n  color: red;\n}\n"
);
test!(unknown_at_rule_no_body, "@foo;\n", "@foo;\n");
test!(unknown_at_rule_empty_body, "@foo {}\n", "@foo {}\n");
test!(unknown_at_rule_no_body_eof, "@foo", "@foo;\n");
test!(
    unknown_at_rule_interpolated_eof_no_body,
    "@#{()if(0,0<0,0)}",
    "@false;\n"
);
test!(nothing_after_hash, "@foo #", "@foo #;\n");
test!(
    style_following,
    "@foo (a: b) {
        a {
            color: red;
        }
    }

    a {
        color: green;
    }",
    "@foo (a: b) {\n  a {\n    color: red;\n  }\n}\na {\n  color: green;\n}\n"
);
test!(
    no_semicolon_no_params_no_body,
    "a {
      @b
    }
    
    a {
      color: red;
    }",
    "a {\n  @b;\n}\n\na {\n  color: red;\n}\n"
);
test!(
    no_semicolon_has_params_no_body,
    "a {
      @foo bar
    }

    a {
      color: red;
    }",
    "a {\n  @foo bar;\n}\n\na {\n  color: red;\n}\n"
);
test!(
    no_body_remains_inside_style_rule,
    "a {
      @box-shadow: $btn-focus-box-shadow, $btn-active-box-shadow;
    }
    
    a {
      color: red;
    }",
    "a {\n  @box-shadow : $btn-focus-box-shadow, $btn-active-box-shadow;\n}\n\na {\n  color: red;\n}\n"
);
test!(
    empty_body_moves_outside_style_rule,
    "a {
      @b {}
    }
    
    a {
      color: red;
    }",
    "@b {}\n\na {\n  color: red;\n}\n"
);
test!(
    parent_selector_moves_inside_rule,
    "a {
     @foo {
       b: c
     }
    }",
    "@foo {\n  a {\n    b: c;\n  }\n}\n"
);
test!(
    parent_selector_moves_inside_rule_and_is_parent_to_inner_selector,
    "a {
     @foo {
       f {
         b: c
       }
     }
    }",
    "@foo {\n  a f {\n    b: c;\n  }\n}\n"
);
test!(
    // The silent comment is dropped and takes the rest of the line with it,
    // including the semicolon. Verified against dart-sass 1.103.1, which
    // prints the same; the old expectation kept the comment.
    params_contain_silent_comment_and_semicolon,
    "a {
      @box-shadow: $btn-focus-box-shadow, // $btn-active-box-shadow;
    }",
    "a {\n  @box-shadow : $btn-focus-box-shadow,;\n}\n"
);
test!(contains_multiline_comment, "@foo /**/;\n", "@foo;\n");
error!(
    unknown_at_rule_inside_declaration_body,
    "@mixin foo {
        @foo;
    }

    a {
        color: {
            @include foo;
        }
    }",
    "Error: At-rules may not be used within nested declarations."
);

// todo: test scoping in rule
test!(
    // The same two fixes reach unknown at-rule values, which are parsed by
    // `almost_any_value`. Verified against dart-sass 1.103.1.
    value_keeps_single_quotes,
    "@asdf 'foo bar baz';\n",
    "@asdf 'foo bar baz';\n"
);
test!(value_drops_a_silent_comment, "@a b //c\n;\n", "@a b;\n");
test!(value_keeps_a_loud_comment, "@a b /*c*/;\n", "@a b /*c*/;\n");
test!(
    // `url()` and `url-prefix()` hold a raw URL, so the `//` in a scheme is
    // not a comment. This is what `@-moz-document` relies on.
    url_prefix_keeps_a_scheme,
    "@a url-prefix(http://x);\n",
    "@a url-prefix(http://x);\n"
);
test!(
    url_keeps_a_scheme,
    "@a url(http://x);\n",
    "@a url(http://x);\n"
);

// A nested at-rule normally gets a copy of the enclosing style rule, so that
// declarations written directly inside it have somewhere to go. dart-sass
// 1.103.1 exempts `@font-face`, whose descriptors belong to the at-rule
// itself, and the exemption is on the plain name: neither case-insensitive
// nor unvendored. All four expectations come from the binary.
test!(
    font_face_does_not_take_the_parent_selector,
    "a {\n  b: c;\n  @font-face {\n    d: e;\n  }\n}\n",
    "a {\n  b: c;\n}\n@font-face {\n  d: e;\n}\n"
);
test!(
    font_face_from_a_mixin_does_not_take_the_parent_selector,
    "@mixin a {\n  @font-face {\n    b: c;\n  }\n}\nd {\n  e: f;\n  @include a;\n}\n",
    "d {\n  e: f;\n}\n@font-face {\n  b: c;\n}\n"
);
test!(
    vendor_prefixed_font_face_still_takes_the_parent_selector,
    "f {\n  @-moz-font-face {\n    g: h;\n  }\n}\n",
    "@-moz-font-face {\n  f {\n    g: h;\n  }\n}\n"
);
test!(
    uppercase_font_face_still_takes_the_parent_selector,
    "i {\n  @FONT-FACE {\n    j: k;\n  }\n}\n",
    "@FONT-FACE {\n  i {\n    j: k;\n  }\n}\n"
);
test!(
    interpolated_font_face_does_not_take_the_parent_selector,
    "l {\n  @#{\"font-face\"} {\n    m: n;\n  }\n}\n",
    "@font-face {\n  m: n;\n}\n"
);
