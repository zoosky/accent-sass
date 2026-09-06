#[macro_use]
mod macros;

test!(
    px_mod_px,
    "a {\n  color: 10px % 2px;\n}\n",
    "a {\n  color: 0px;\n}\n"
);
test!(
    px_mod_in,
    "a {\n  color: 10px % 2in;\n}\n",
    "a {\n  color: 10px;\n}\n"
);
test!(
    px_mod_none,
    "a {\n  color: 10px % 2;\n}\n",
    "a {\n  color: 0px;\n}\n"
);
test!(
    none_mod_px,
    "a {\n  color: 10 % 2px;\n}\n",
    "a {\n  color: 0px;\n}\n"
);
test!(
    none_mod_none,
    "a {\n  color: 10 % 2;\n}\n",
    "a {\n  color: 0;\n}\n"
);
test!(
    zero_mod_zero,
    "a {\n  color: 0 % 0;\n}\n",
    "a {\n  color: calc(NaN);\n}\n"
);
test!(
    positive_mod_zero,
    "a {\n  color: 1 % 0;\n}\n",
    "a {\n  color: calc(NaN);\n}\n"
);
test!(
    positive_unit_mod_zero,
    "a {\n  color: 1px % 0;\n}\n",
    "a {\n  color: calc(NaN * 1px);\n}\n"
);
test!(
    positive_mod_zero_unit,
    "a {\n  color: 1 % 0px;\n}\n",
    "a {\n  color: calc(NaN * 1px);\n}\n"
);
test!(
    positive_unit_mod_zero_unit_same,
    "a {\n  color: 1px % 0px;\n}\n",
    "a {\n  color: calc(NaN * 1px);\n}\n"
);
test!(
    positive_unit_mod_zero_unit_different_compatible_takes_first_1,
    "a {\n  color: 1px % 0in;\n}\n",
    "a {\n  color: calc(NaN * 1px);\n}\n"
);
test!(
    positive_unit_mod_zero_unit_different_compatible_takes_first_2,
    "a {\n  color: 1in % 0px;\n}\n",
    "a {\n  color: calc(NaN * 1in);\n}\n"
);
error!(
    positive_unit_mod_zero_unit_incompatible_units,
    "a {\n  color: 1rem % 0px;\n}\n", "Error: Incompatible units rem and px."
);
test!(
    positive_mod_negative,
    "a {\n  color: 1 % -4;\n}\n",
    "a {\n  color: -3;\n}\n"
);
test!(
    negative_mod_positive,
    "a {\n  color: -4 % 3;\n}\n",
    "a {\n  color: 2;\n}\n"
);
test!(
    negative_mod_negative,
    "a {\n  color: -4 % -3;\n}\n",
    "a {\n  color: -1;\n}\n"
);
test!(
    big_negative_mod_positive,
    "a {\n  color: -99999990000099999999999999 % 2;\n}\n",
    "a {\n  color: 0;\n}\n"
);
test!(
    big_int_result_is_equal_to_small_int,
    "a {\n  color: (6 % 2) == 0;\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    comparable_units_denom_0,
    "a {\n  color: 1in % 0px;\n}\n",
    "a {\n  color: calc(NaN * 1in);\n}\n"
);
test!(
    comparable_units_negative_denom_0,
    "a {\n  color: -1in % 0px;\n}\n",
    "a {\n  color: calc(NaN * 1in);\n}\n"
);
test!(
    comparable_units_both_positive,
    "a {\n  color: 1in % 1px;\n}\n",
    "a {\n  color: 0in;\n}\n"
);
test!(
    comparable_units_denom_negative,
    "a {\n  color: 1in % -1px;\n}\n",
    "a {\n  color: -0.0104166667in;\n}\n"
);
test!(
    comparable_units_both_negative,
    "a {\n  color: -1in % -1px;\n}\n",
    "a {\n  color: 0in;\n}\n"
);
test!(
    comparable_units_numer_negative,
    "a {\n  color: -1in % 1px;\n}\n",
    "a {\n  color: 0.0104166667in;\n}\n"
);
test!(
    comparable_units_both_0,
    "a {\n  color: 0in % 0px;\n}\n",
    "a {\n  color: calc(NaN * 1in);\n}\n"
);
test!(
    nan_mod_positive_finite,
    "a {\n  color: (0/0) % 5;\n}\n",
    "a {\n  color: calc(NaN);\n}\n"
);
test!(
    nan_mod_negative_finite,
    "a {\n  color: (0/0) % -5;\n}\n",
    "a {\n  color: calc(NaN);\n}\n"
);
test!(
    infinity_mod_positive_finite,
    "a {\n  color: (1/0) % 5;\n}\n",
    "a {\n  color: calc(NaN);\n}\n"
);
test!(
    infinity_mod_negative_finite,
    "a {\n  color: (1/0) % -5;\n}\n",
    "a {\n  color: calc(NaN);\n}\n"
);
test!(
    positive_finite_mod_nan,
    "a {\n  color: 5 % (0/0);\n}\n",
    "a {\n  color: calc(NaN);\n}\n"
);
test!(
    negative_finite_mod_nan,
    "a {\n  color: -5 % (0/0);\n}\n",
    "a {\n  color: calc(NaN);\n}\n"
);
test!(
    positive_finite_mod_infinity,
    "a {\n  color: 5 % (1/0);\n}\n",
    "a {\n  color: 5;\n}\n"
);
test!(
    negative_finite_mod_infinity,
    "a {\n  color: -5 % (1/0);\n}\n",
    "a {\n  color: calc(NaN);\n}\n"
);
test!(
    positive_finite_mod_negative_infinity,
    "a {\n  color: 5 % (-1/0);\n}\n",
    "a {\n  color: calc(NaN);\n}\n"
);
test!(
    negative_finite_mod_negative_infinity,
    "a {\n  color: -5 % (-1/0);\n}\n",
    "a {\n  color: -5;\n}\n"
);
test!(
    zero_mod_negative,
    "a {\n  color: 0 % -5;\n}\n",
    "a {\n  color: 0;\n}\n"
);
error!(
    calculation_mod_calculation,
    "a {\n  color: calc(1rem + 1px) % calc(1rem + 1px);\n}\n",
    r#"Error: Undefined operation "calc(1rem + 1px) % calc(1rem + 1px)"."#
);
error!(
    number_mod_color,
    "a {\n  color: 5 % red;\n}\n", r#"Error: Undefined operation "5 % red"."#
);

// An infinite divisor keeps the dividend when the operands share a sign and
// is NaN otherwise; an infinite dividend is always NaN. Cross-checked against
// Dart Sass 1.103.1.
test!(
    positive_finite_mod_negative_infinity_is_nan,
    "@use \"sass:math\";\na {\n  color: 5 % math.div(-1, 0);\n}\n",
    "a {\n  color: calc(NaN);\n}\n"
);
test!(
    negative_finite_mod_negative_infinity_keeps_dividend,
    "@use \"sass:math\";\na {\n  color: -5 % math.div(-1, 0);\n}\n",
    "a {\n  color: -5;\n}\n"
);
test!(
    zero_mod_infinity_is_nan,
    "@use \"sass:math\";\na {\n  color: 0 % math.div(1, 0);\n}\n",
    "a {\n  color: calc(NaN);\n}\n"
);

// A lone `%` is an unquoted string value, not the modulo operator. Every
// expectation below was checked against dart-sass 1.103.1.
test!(bare_percent_alone, "a {b: %}\n", "a {\n  b: %;\n}\n");
test!(
    bare_percent_before_value,
    "a {b: % c}\n",
    "a {\n  b: % c;\n}\n"
);
test!(
    // No operand follows the `%`, so it joins the space-separated list
    // rather than becoming an operator.
    bare_percent_after_value,
    "a {b: c %}\n",
    "a {\n  b: c %;\n}\n"
);
test!(
    bare_percent_in_function_argument,
    "a {b: c(%)}\n",
    "a {\n  b: c(%);\n}\n"
);
test!(
    // A `%` never joins the identifier next to it.
    bare_percent_adjacent_to_identifier,
    "a {b: a%}\n",
    "a {\n  b: a %;\n}\n"
);
test!(bare_percent_twice, "a {b: %%}\n", "a {\n  b: % %;\n}\n");
test!(
    bare_percent_after_unary_minus,
    "a {b: -%}\n",
    "a {\n  b: -%;\n}\n"
);
test!(
    // Whitespace before the operand is optional, so this is still modulo.
    percent_is_still_modulo_without_trailing_space,
    "a {b: 5 %2}\n",
    "a {\n  b: 1;\n}\n"
);
test!(
    // `5%` is a percentage, so this is a two-element list, not modulo.
    percentage_number_beats_modulo,
    "a {b: 5%2}\n",
    "a {\n  b: 5% 2;\n}\n"
);
test!(
    // dart-sass takes each parenthesized element as its own expression, so
    // the `%` is first in its own parse and is accepted.
    bare_percent_in_parenthesized_list,
    "a {b: (1, %)}\n",
    "a {\n  b: 1, %;\n}\n"
);
test!(
    bare_percent_before_a_comma,
    "a {b: 1 %, 2}\n",
    "a {\n  b: 1 %, 2;\n}\n"
);
error!(
    // dart-sass rejects a `%` value once the expression has consumed a
    // comma, while accepting `%, 2`. Verified against 1.103.1.
    bare_percent_after_a_comma,
    "a {b: 1, %, 2}\n", "Error: Expected expression."
);
error!(
    bare_percent_after_a_comma_in_brackets,
    "a {b: [1, %]}\n", "Error: Expected expression."
);
test!(
    // The `%` value ends the slash-separated list, so the `/` is division.
    // Without that, `1/2` stayed a slash list and printed verbatim.
    bare_percent_after_a_slash_list,
    "a {b: 1/2 %}\n",
    "a {\n  b: 0.5 %;\n}\n"
);
test!(
    bare_percent_after_a_slash_list_in_parens,
    "a {b: (1/2 %)}\n",
    "a {\n  b: 0.5 %;\n}\n"
);
test!(
    // A comment between the `%` and the end of the value does not make the
    // `%` an operator.
    bare_percent_before_a_loud_comment,
    "a {b: c % /* d */}\n",
    "a {\n  b: c %;\n}\n"
);
test!(
    bare_percent_before_a_silent_comment,
    "a {b: c %// d\n}\n",
    "a {\n  b: c %;\n}\n"
);
test!(
    // `!default` ends the value, so the `%` before it is a value; only
    // `!important` and `!=` continue the expression.
    bare_percent_before_bang_default,
    "$x: c % !default;\na {b: $x}\n",
    "a {\n  b: c %;\n}\n"
);
error!(
    // An operand follows the `%`, so it stays the operator and fails in
    // evaluation the way dart-sass does.
    percent_is_operator_before_a_star,
    "a {b: 1 %*2}\n", "Error: Undefined operation \"% * 2\"."
);
