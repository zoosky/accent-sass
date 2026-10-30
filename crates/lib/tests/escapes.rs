//! Escapes as dart-sass 1.106.0 handles them.
//!
//! An identifier the parser reads now holds its contents with escapes
//! resolved, and statement-level text keeps escapes as written. In a SassScript
//! expression, an escape of U+0000, of a surrogate or of a code point beyond
//! U+10FFFF is U+FFFD, except that `\0` on its own stays `\0 ` for CSS hacks.
//! A keyword or a function name matches with its escapes resolved, so
//! `!\69 mportant`, `\61 nd` and `\75 rl(` are what they spell, and a unit
//! that would read back as an exponent is written with an escaped `e`.
//!
//! Every expectation here is the output of the dart-sass 1.107.0 release
//! binary for the same input. Some cases print what 1.105.2 printed too; they
//! guard behaviour the port could have broken, such as `@\6d edia` still being
//! a media rule.

#[macro_use]
mod macros;

test!(
    zero_escape_at_start,
    "a {b: \\0xyz}",
    "@charset \"UTF-8\";\na {\n  b: \u{FFFD}xyz;\n}\n"
);
test!(
    zero_escape_in_middle,
    "a {b: x\\0y}",
    "@charset \"UTF-8\";\na {\n  b: x\u{FFFD}y;\n}\n"
);
test!(
    zero_escape_with_trailing_space,
    "a {b: \\0 x}",
    "@charset \"UTF-8\";\na {\n  b: \u{FFFD}x;\n}\n"
);
test!(
    zero_escape_alone_is_the_css_hack,
    "a {b: \\0}",
    "a {\n  b: \\0 ;\n}\n"
);
test!(
    zero_escape_alone_with_leading_zeros,
    "a {b: \\00}",
    "a {\n  b: \\0 ;\n}\n"
);
test!(
    zero_escape_after_a_number_is_a_unit,
    "a {b: 0\\0}",
    "@charset \"UTF-8\";\na {\n  b: 0\u{FFFD};\n}\n"
);
test!(
    zero_escape_after_a_hack_unit,
    "a {b: 1px\\9\\0}",
    "@charset \"UTF-8\";\na {\n  b: 1px\\9 \u{FFFD};\n}\n"
);
test!(
    zero_escape_interpolated,
    "a {b: #{\\0}; c: \\0#{1}; d: a#{\\0}b}",
    "@charset \"UTF-8\";\na {\n  b: \\0 ;\n  c: \u{FFFD}1;\n  d: a\\0 b;\n}\n"
);
test!(
    zero_escape_is_not_a_string_of_zero,
    "a {b: \\0 == \"\\0\"; c: \\0 == \\0}",
    "a {\n  b: false;\n  c: true;\n}\n"
);
test!(
    zero_escape_quoted,
    "@use \"sass:string\"; a {b: string.quote(\\0); c: string.length(\\0x)}",
    "a {\n  b: \"\\\\0 \";\n  c: 2;\n}\n"
);
test!(
    surrogate_escape,
    "a {b: \\D800}",
    "@charset \"UTF-8\";\na {\n  b: \u{FFFD};\n}\n"
);
test!(
    escape_beyond_max_code_point,
    "a {b: \\110000}",
    "@charset \"UTF-8\";\na {\n  b: \u{FFFD};\n}\n"
);
test!(
    max_code_point_in_string,
    "a {b: \"\\10FFFF\"}",
    "a {\n  b: \"\\10ffff\";\n}\n"
);
test!(
    zero_escape_in_url,
    "a {b: url(\\0)}",
    "@charset \"UTF-8\";\na {\n  b: url(\u{FFFD});\n}\n"
);
test!(escaped_e_unit, "a {b: 1\\65 1}", "a {\n  b: 1\\65 1;\n}\n");
test!(
    uppercase_escaped_e_unit,
    "a {b: 1\\45 1}",
    "a {\n  b: 1\\65 1;\n}\n"
);
test!(
    unit_after_exponent,
    "a {b: 1e1e1}",
    "a {\n  b: 10\\65 1;\n}\n"
);
test!(
    unit_after_negative_exponent,
    "a {b: 1e-1e1}",
    "a {\n  b: 0.1\\65 1;\n}\n"
);
test!(
    exponent_unit_survives_arithmetic,
    "@use \"sass:math\"; a {b: math.div(1e2e2, 1); c: (1 * 1e1e1)}",
    "a {\n  b: 100\\65 2;\n  c: 10\\65 1;\n}\n"
);
test!(escaped_dash_unit, "a {b: 1\\2d x}", "a {\n  b: 1-x;\n}\n");
test!(escaped_digit_unit, "a {b: 1\\31}", "a {\n  b: 1\\31 ;\n}\n");
test!(escaped_percent_unit, "a {b: 1\\25}", "a {\n  b: 1\\%;\n}\n");
test!(hack_unit, "a {b: 1px\\9}", "a {\n  b: 1px\\9 ;\n}\n");
test!(
    escaped_important,
    "a {b: c !\\69 mportant}",
    "a {\n  b: c !important;\n}\n"
);
test!(
    escaped_uppercase_important,
    "a {b: c !\\49 MPORTANT}",
    "a {\n  b: c !important;\n}\n"
);
test!(
    escaped_and_operator,
    "a {b: true \\61 nd false}",
    "a {\n  b: false;\n}\n"
);
test!(
    escaped_or_operator,
    "a {b: false \\6f r true}",
    "a {\n  b: true;\n}\n"
);
test!(
    escaped_underscore_variable_name,
    "$a\\5f b: 1; c {d: $a-b}",
    "c {\n  d: 1;\n}\n"
);
test!(
    variable_name_with_escaped_underscore,
    "$a_b: 1; c {d: $a\\5f b}",
    "c {\n  d: 1;\n}\n"
);
test!(
    escaped_function_name,
    "@function f\\6f o() {@return 1} a {b: foo()}",
    "a {\n  b: 1;\n}\n"
);
test!(
    escaped_mixin_rule,
    "@\\6d ixin b {c: d} e {@include b}",
    "e {\n  c: d;\n}\n"
);
test!(
    escaped_media_rule,
    "@\\6d edia screen {a {b: c}}",
    "@media screen {\n  a {\n    b: c;\n  }\n}\n"
);
test!(
    escaped_named_color_keeps_its_escape,
    "a {b: \\62 lue; c: blu\\65}",
    "a {\n  b: \\62 lue;\n  c: blu\\65;\n}\n"
);
test!(
    escaped_rgb_call,
    "a {b: \\72 gb(1, 2, 3)}",
    "a {\n  b: rgb(1, 2, 3);\n}\n"
);
test!(unknown_at_rule_name, "@f\\6Fo;", "@f\\6Fo;\n");
test!(
    unknown_at_rule_url_value,
    "@a \\75 rl(http://example.org);",
    "@a \\75 rl(http://example.org);\n"
);
test!(
    unknown_at_rule_value,
    "@a b\\0c; @b \\61;",
    "@a b\\0c;\n@b \\61;\n"
);
test!(
    keyframes_name,
    "@keyframes \\61 {from {b: c}}",
    "@keyframes \\61 {\n  from {\n    b: c;\n  }\n}\n"
);
test!(
    page_selector,
    "@page :\\66 irst {a: b}",
    "@page :\\66 irst {\n  a: b;\n}\n"
);
test!(
    layer_name,
    "@layer \\61 {a {b: c}}",
    "@layer \\61 {\n  a {\n    b: c;\n  }\n}\n"
);
test!(
    property_rule_name,
    "@property --\\61 {syntax: \"*\"}",
    "@property --\\61 {\n  syntax: \"*\";\n}\n"
);
test!(
    namespace_rule,
    "@namespace \\61 \"b\";",
    "@namespace \\61 \"b\";\n"
);
test!(
    escaped_charset_rule,
    "@\\63 harset \"a\";",
    "@\\63 harset \"a\";\n"
);
test!(
    scope_rule,
    "@scope (.\\31) {a {b: c}}",
    "@scope (.\\31) {\n  a {\n    b: c;\n  }\n}\n"
);
test!(
    import_media_query,
    "@import \"a\" \\73 creen;",
    "@import \"a\" \\73 creen;\n"
);
test!(
    import_modifier,
    "@import url(x) \\0;",
    "@import url(x) \\0;\n"
);
test!(
    escaped_property_name,
    "a {\\63 olor: red; c\\6flor: blue}",
    "a {\n  \\63 olor: red;\n  c\\6flor: blue;\n}\n"
);
test!(
    escaped_vendor_property_name,
    "a {-\\77 ebkit-b: c}",
    "a {\n  -\\77 ebkit-b: c;\n}\n"
);
test!(
    escaped_property_does_not_merge,
    "a {b: c; \\62: d}",
    "a {\n  b: c;\n  \\62: d;\n}\n"
);
test!(
    escaped_custom_property,
    "a {--\\61: b; --c: \\61; --d: \\75 rl(x)}",
    "a {\n  --\\61: b;\n  --c: \\61;\n  --d: \\75 rl(x);\n}\n"
);
test!(
    escaped_nested_property,
    "a {b: {c\\5f d: e}; \\62: {c: d}}",
    "a {\n  b-c\\5f d: e;\n  \\62-c: d;\n}\n"
);
test!(
    interpolated_property_with_escape,
    "a {#{b}\\0: c}",
    "a {\n  b\\0: c;\n}\n"
);
test!(
    escaped_url_import,
    "@import \\75 rl(x);",
    "@import url(x);\n"
);
test!(
    selector_zero_escape,
    ".\\0 {c: d} .a\\0 b {c: d}",
    "@charset \"UTF-8\";\n.\u{FFFD} {\n  c: d;\n}\n\n.a\u{FFFD}b {\n  c: d;\n}\n"
);
test!(
    selector_private_use,
    ".\\E000 {c: d}",
    ".\\e000  {\n  c: d;\n}\n"
);
test!(
    selector_escaped_dash,
    ".\\-a {c: d} .\\2d {e: f} .\\2d x {g: h}",
    ".-a {\n  c: d;\n}\n\n.\\2d  {\n  e: f;\n}\n\n.-x {\n  g: h;\n}\n"
);
test!(
    selector_escape_beyond_max_code_point,
    ".\\110000 {c: d}",
    "@charset \"UTF-8\";\n.\u{FFFD} {\n  c: d;\n}\n"
);
test!(
    selector_parent_suffix_zero_escape,
    ".a {&\\0 {c: d}}",
    "@charset \"UTF-8\";\n.a\u{FFFD} {\n  c: d;\n}\n"
);
test!(
    selector_interpolated_zero_escape,
    "$a: \\0; .#{$a} {b: c}",
    "@charset \"UTF-8\";\n.\u{FFFD} {\n  b: c;\n}\n"
);
test!(
    attribute_value_with_backslash,
    "[a=\"b\\\\c\"] {c: d}",
    "[a=\"b\\\\c\"] {\n  c: d;\n}\n"
);
test!(
    attribute_value_with_escaped_space,
    "[a=b\\ c] {c: d}",
    "[a=\"b c\"] {\n  c: d;\n}\n"
);
test!(
    attribute_empty_namespace,
    "[|b=c] {c: d}",
    "[|b=c] {\n  c: d;\n}\n"
);
test!(
    an_plus_b_escaped_odd,
    ":nth-child(\\6f dd) {c: d}",
    ":nth-child(odd) {\n  c: d;\n}\n"
);
test!(
    an_plus_b_escaped_even,
    ":nth-child(\\65 ven) {c: d}",
    ":nth-child(even) {\n  c: d;\n}\n"
);
test!(
    an_plus_b_escaped_uppercase,
    ":nth-child(\\4f DD) {c: d}",
    ":nth-child(odd) {\n  c: d;\n}\n"
);
test!(
    selector_parse_zero_escape,
    "@use \"sass:selector\"; a {b: selector.parse(\".a\\\\0\")}",
    "@charset \"UTF-8\";\na {\n  b: .a\u{FFFD};\n}\n"
);
test!(
    media_query_zero_escape,
    "@media \\0 screen {a {b: c}} @media screen\\0 {a {b: c}}",
    "@charset \"UTF-8\";\n@media \u{FFFD}screen {\n  a {\n    b: c;\n  }\n}\n@media screen\u{FFFD} {\n  a {\n    b: c;\n  }\n}\n"
);
test!(
    media_query_hack_value,
    "@media (min-width: 0\\0) {a {b: c}}",
    "@charset \"UTF-8\";\n@media (min-width: 0\u{FFFD}) {\n  a {\n    b: c;\n  }\n}\n"
);
error!(
    media_query_escaped_and_is_not_and,
    "@media screen \\61 nd (color) {a {b: c}}", "Error: expected \"{\"."
);
test!(
    supports_zero_escape,
    "@supports (a: \\0) {a {b: c}}",
    "@supports (a: \\0 ) {\n  a {\n    b: c;\n  }\n}\n"
);
error!(
    keyframes_escaped_exponent,
    "@keyframes a {\n  1\\65 1% {b: c}\n}", "Error: expected \"%\"."
);
