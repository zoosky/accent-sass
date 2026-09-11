use accent_sass_compiler::OutputStyle;

#[macro_use]
mod macros;

test!(
    escape_leading_zeros,
    "a {\n  color: ax \\61x \\61 x \\061x \\0061x \\00061x;\n}\n",
    "a {\n  color: ax ax ax ax ax ax;\n}\n"
);
test!(
    escape_start_non_hex,
    "a {\n  color: \\xx;\n}\n",
    "a {\n  color: xx;\n}\n"
);
test!(
    escape_start_non_ascii,
    "a {\n  color: ☃x \\☃x \\2603x;\n}\n",
    "@charset \"UTF-8\";\na {\n  color: ☃x ☃x ☃x;\n}\n"
);
test!(
    escape_hyphen_in_middle,
    "a {\n  color: a\\2dx a\\-x;\n}\n",
    "a {\n  color: a-x a-x;\n}\n"
);
test!(
    escape_hyphen_at_start,
    "a {\n  color: \\2dx \\-x;\n}\n",
    "a {\n  color: \\-x \\-x;\n}\n"
);
test!(
    escape_digit_in_middle,
    "a {\n  color: a\\31x a\\31 x;\n}\n",
    "a {\n  color: a1x a1x;\n}\n"
);
test!(
    escape_digit_at_start,
    "a {\n  color: \\31x \\31 x;\n}\n",
    "a {\n  color: \\31 x \\31 x;\n}\n"
);
test!(
    escape_non_printable_characters,
    "a {\n  color: \\0x \\1x \\2x \\3x \\4x \\5x \\6x \\7x \\8x \\Bx \\Ex \\Fx \\10x \\11x \\12x \\13x \\14x \\15x \\16x \\17x \\18x \\19x \\1Ax \\1Bx \\1Cx \\1Dx \\1Ex \\1Fx \\7Fx;\n}\n",
    "a {\n  color: \\0 x \\1 x \\2 x \\3 x \\4 x \\5 x \\6 x \\7 x \\8 x \\b x \\e x \\f x \\10 x \\11 x \\12 x \\13 x \\14 x \\15 x \\16 x \\17 x \\18 x \\19 x \\1a x \\1b x \\1c x \\1d x \\1e x \\1f x \\7f x;\n}\n"
);
test!(
    escape_newlines,
    "a {\n  color: \\ax \\cx \\dx;\n}\n",
    "a {\n  color: \\a x \\c x \\d x;\n}\n"
);
test!(
    escape_tabs,
    "a {\n  color: \\	x \\9x;\n}\n",
    "a {\n  color: \\9 x \\9 x;\n}\n"
);
test!(
    escape_interpolation_start,
    "a {\n  color: \\-#{foo};\n}\n",
    "a {\n  color: \\-foo;\n}\n"
);
test!(
    escape_interpolation_middle,
    "a {\n  color: #{foo}\\-#{bar};\n}\n",
    "a {\n  color: foo-bar;\n}\n"
);
test!(
    escape_interpolation_end,
    "a {\n  color: #{foo}\\-;\n}\n",
    "a {\n  color: foo-;\n}\n"
);
test!(
    escape_in_middle,
    "a {\n  color: b\\6cue;\n}\n",
    "a {\n  color: blue;\n}\n"
);
test!(
    escape_at_end,
    "a {\n  color: blu\\65;\n}\n",
    "a {\n  color: blue;\n}\n"
);
test!(
    double_escape_is_preserved,
    "a {\n  color: r\\\\65;\n}\n",
    "a {\n  color: r\\\\65;\n}\n"
);
test!(
    semicolon_in_string,
    "a {\n  color: \";\";\n}\n",
    "a {\n  color: \";\";\n}\n"
);
test!(
    single_character_escape_sequence_has_space,
    "a {\n  color: \\fg1;\n}\n",
    "a {\n  color: \\f g1;\n}\n"
);
test!(
    single_character_escape_sequence_removes_slash_when_not_hex_digit,
    "a {\n  color: \\g1;\n}\n",
    "a {\n  color: g1;\n}\n"
);
test!(
    single_character_escape_sequence_has_space_after,
    "a {\n  color: \\a;\n}\n",
    "a {\n  color: \\a ;\n}\n"
);
test!(
    escapes_non_hex_in_string,
    "a {\n  color: \"\\g\";\n}\n",
    "a {\n  color: \"g\";\n}\n"
);
test!(
    escapes_hex_in_string_no_trailing_space,
    "a {\n  color: \"\\b\\c\\d\\e\\f\\g\\h\\i\\j\\k\\l\\m\\n\\o\\p\\q\\r\\s\\t\\u\\v\\w\\x\\y\\z\";\n}\n",
    "a {\n  color: \"\\b\\c\\d\\e\\fghijklmnopqrstuvwxyz\";\n}\n"
);
test!(
    interpolated_inside_string_does_not_produce_unquoted_output,
    "a {\n  color: \"#{\"\\b\"}\";\n}\n",
    "a {\n  color: \"\\b\";\n}\n"
);
test!(
    unquote_quoted_backslash_single_lowercase_hex_char,
    "a {\n  color: #{\"\\b\"};\n}\n",
    "a {\n  color: \x0b;\n}\n"
);
test!(
    unquoted_escape_equality,
    "a {\n  color: foo == f\\6F\\6F;\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    quoted_escape_zero,
    "a {\n  color: \"\\0\";\n}\n",
    "@charset \"UTF-8\";\na {\n  color: \"�\";\n}\n"
);
test!(
    unquoted_escape_zero,
    "a {\n  color: \\0;\n}\n",
    "a {\n  color: \\0 ;\n}\n"
);
test!(
    quote_escape,
    "a {\n  color: quote(\\b);\n}\n",
    "a {\n  color: \"\\\\b \";\n}\n"
);
test!(
    escaped_backslash,
    "a {\n  color: \"\\\\\";\n}\n",
    "a {\n  color: \"\\\\\";\n}\n"
);
test!(
    double_quotes_when_containing_single_quote,
    "a {\n  color: '\\\'';\n}\n",
    "a {\n  color: \"'\";\n}\n"
);
test!(
    allows_escaped_quote_at_start_of_ident,
    "a {\n  color: \\\"c\\\";\n}\n",
    "a {\n  color: \\\"c\\\";\n}\n"
);
test!(
    quoted_escaped_newline_unchanged,
    "a {\n  color: \"\\a\";\n}\n",
    "a {\n  color: \"\\a\";\n}\n"
);
test!(
    unquoted_escape_minus_unquoted,
    "a {\n  color: \\a - foo;\n}\n",
    "a {\n  color: \\a - foo;\n}\n"
);
test!(
    quoted_escaped_tab,
    "a {\n  color: \"\\9\";\n}\n",
    "a {\n  color: \"\t\";\n}\n"
);
test!(
    unquoted_escaped_tab,
    "a {\n  color: \\9;\n}\n",
    "a {\n  color: \\9 ;\n}\n"
);
error!(
    escape_sequence_does_not_fit_inside_char,
    "a {\n  color: \\110000;\n}\n", "Error: Invalid Unicode code point."
);
test!(
    escaped_newline_in_quoted_string,
    "a {\n  color: \"foo\\\nbar\";\n}\n",
    "a {\n  color: \"foobar\";\n}\n"
);
test!(
    escaped_value_over_0xf_in_quoted_string,
    "a {\n  color: \"#{\"\\1f\"}\";\n}\n",
    "a {\n  color: \"\\1f\";\n}\n"
);
test!(
    escaped_value_over_0xf_in_quoted_string_with_trailing_space,
    "a {\n  color: \"#{\"\\1f\"} \";\n}\n",
    "a {\n  color: \"\\1f  \";\n}\n"
);
error!(
    newline_after_escape,
    "a {\n  color: \\\n", "Error: Expected escape sequence."
);

// In expanded mode dart-sass writes every character in a Unicode Private Use
// Area back as an escape, because there is no useful way to render one and a
// reader of a glyph font's stylesheet needs to tell them apart. Compressed
// mode writes the character itself, since nobody reads that output. The
// ranges are `U+E000..=U+F8FF` and `U+F0000..=U+10FFFF`; dart-sass 1.103.1
// produced every expectation below.
test!(
    private_use_character_is_escaped_in_a_quoted_string,
    "a {\n  b: \"\\e600\";\n}\n",
    "a {\n  b: \"\\e600\";\n}\n"
);
test!(
    private_use_character_is_escaped_in_an_unquoted_string,
    "a {\n  b: \\e600;\n}\n",
    "a {\n  b: \\e600;\n}\n"
);
// `libsass-closed-issues/issue_1231`: the escape reaches the output through
// interpolation and string concatenation, so it is the serializer that has to
// put it back, not the parser that has to keep it.
test!(
    private_use_character_survives_interpolation,
    "div::before {\n  content: #{\"\\\"\"+\\e600+\"\\\"\"};\n}\n",
    "div::before {\n  content: \"\\e600\";\n}\n"
);
// Escaping the character keeps the output ASCII, so no `@charset` is needed.
test!(
    private_use_character_does_not_force_a_charset,
    "a {\n  b: \"\\e600\";\n}\n",
    "a {\n  b: \"\\e600\";\n}\n"
);
// A supplementary private-use character is one character, not a surrogate
// pair, and escapes as its full code point.
test!(
    supplementary_private_use_character_is_escaped,
    "a {\n  b: \"\\f0000\";\n  c: \"\\10fffd\";\n}\n",
    "a {\n  b: \"\\f0000\";\n  c: \"\\10fffd\";\n}\n"
);
// One past each end of the Basic Multilingual Plane's area, and one past the
// end of the plane below the supplementary areas. `\dfff` is a surrogate and
// so not a code point at all, which is why it arrives as U+FFFD.
test!(
    characters_outside_the_private_use_areas_are_written_as_themselves,
    "a {\n  b: \"\\dfff\";\n  c: \"\\f900\";\n  d: \"\\effff\";\n}\n",
    "@charset \"UTF-8\";\na {\n  b: \"\u{fffd}\";\n  c: \"\u{f900}\";\n  d: \"\u{effff}\";\n}\n"
);
// An escape ends at the first character that cannot continue it, so a hex
// digit, space or tab after one needs a space to keep it out.
test!(
    private_use_escape_takes_a_space_before_a_hex_digit,
    "a {\n  b: \"\\e600 abc\";\n  c: \"\\e600zzz\";\n  d: \"\\e600  x\";\n}\n",
    "a {\n  b: \"\\e600 abc\";\n  c: \"\\e600zzz\";\n  d: \"\\e600  x\";\n}\n"
);
// `core_functions/string/split/private_use_character`: escaping is a
// serialization concern, so the string still holds one character.
test!(
    private_use_character_is_one_character_to_string_functions,
    "@use \"sass:string\";\na {b: string.split(\"\\E000\", \"\")}\n",
    "a {\n  b: [\"\\e000\"];\n}\n"
);
test!(
    private_use_character_is_not_escaped_when_compressed,
    "a {\n  b: \"\\e600\";\n}\n",
    "a{b:\"\u{e600}\"}",
    accent_sass::Options::default()
        .allows_charset(false)
        .style(OutputStyle::Compressed)
);
