//! Negative zero, which dart-sass prints as `-0` from 1.104.0 on.
//!
//! Only exact negative zero keeps its sign, so it stays negative when used in
//! a CSS calculation. Anything dart-sass computes as an integer has no negative
//! zero and prints `0`: `math.round`, `math.ceil`, `math.floor` and the
//! calculation `round()`.
//!
//! Every expectation here was compared against the dart-sass 1.104.0 binary
//! before being committed.

#[macro_use]
mod macros;

test!(
    zero_times_negative_one,
    "a {\n  b: 0 * -1;\n}\n",
    "a {\n  b: -0;\n}\n"
);
test!(
    negative_zero_with_unit,
    "a {\n  b: -0px;\n}\n",
    "a {\n  b: -0px;\n}\n"
);
test!(
    negative_zero_calc_simplifies,
    "a {\n  b: calc(-0 * 1px);\n}\n",
    "a {\n  b: -0px;\n}\n"
);
test!(
    math_div_negative_zero,
    "@use \"sass:math\";\na {\n  b: math.div(-0, 5);\n}\n",
    "a {\n  b: -0;\n}\n"
);
test!(
    math_sin_negative_zero,
    "@use \"sass:math\";\na {\n  b: math.sin(-0);\n}\n",
    "a {\n  b: -0;\n}\n"
);
test!(
    math_round_to_zero_is_positive,
    "@use \"sass:math\";\na {\n  b: math.round(-0.4);\n}\n",
    "a {\n  b: 0;\n}\n"
);
test!(
    math_ceil_to_zero_is_positive,
    "@use \"sass:math\";\na {\n  b: math.ceil(-0.5);\n}\n",
    "a {\n  b: 0;\n}\n"
);
test!(
    calc_round_to_zero_is_positive,
    "a {\n  b: round(-0.4);\n}\n",
    "a {\n  b: 0;\n}\n"
);
test!(
    calc_round_up_to_zero_is_positive,
    "a {\n  b: round(up, -0.5, 1);\n}\n",
    "a {\n  b: 0;\n}\n"
);
test!(
    calc_round_to_zero_strategy_is_positive,
    "a {\n  b: round(to-zero, -0.5, 1);\n}\n",
    "a {\n  b: 0;\n}\n"
);
test!(
    calc_round_down_negative_zero_is_positive,
    "a {\n  b: round(down, -0, 1);\n}\n",
    "a {\n  b: 0;\n}\n"
);
test!(
    math_asin_negative_zero,
    "@use \"sass:math\";\na {\n  b: math.asin(-0.0);\n}\n",
    "a {\n  b: -0deg;\n}\n"
);
test!(
    math_atan_negative_zero,
    "@use \"sass:math\";\na {\n  b: math.atan(-0.0);\n}\n",
    "a {\n  b: -0deg;\n}\n"
);
// An infinite divisor keeps the dividend when the two share a sign, and only
// negative zero counts as negative. This compiler used to count every zero as
// negative, which gave the opposite for `0`; the native dart-sass binary, 1.103.1
// included, never did. See item 07, section 1.
test!(
    modulo_zero_by_infinity,
    "@use \"sass:math\";\na {\n  b: 0 % math.div(1, 0);\n}\n",
    "a {\n  b: 0;\n}\n"
);
test!(
    modulo_zero_by_negative_infinity,
    "@use \"sass:math\";\na {\n  b: 0 % math.div(-1, 0);\n}\n",
    "a {\n  b: calc(NaN);\n}\n"
);
test!(
    modulo_negative_zero_by_negative_infinity,
    "@use \"sass:math\";\na {\n  b: -0 % math.div(-1, 0);\n}\n",
    "a {\n  b: -0;\n}\n"
);
test!(
    calc_mod_zero_by_infinity,
    "a {\n  b: mod(0, infinity);\n}\n",
    "a {\n  b: 0;\n}\n"
);
test!(
    calc_mod_zero_by_negative_infinity,
    "a {\n  b: mod(0, -infinity);\n}\n",
    "a {\n  b: calc(NaN);\n}\n"
);
// `math.log` divides two natural logarithms, and `ln(0)` is `-infinity`.
test!(
    math_log_base_zero_above_one,
    "@use \"sass:math\";\na {\n  b: math.log(2, 0);\n}\n",
    "a {\n  b: -0;\n}\n"
);
test!(
    math_log_base_zero_below_one,
    "@use \"sass:math\";\na {\n  b: math.log(0.5, 0);\n}\n",
    "a {\n  b: 0;\n}\n"
);
test!(
    math_log_zero_base_zero,
    "@use \"sass:math\";\na {\n  b: math.log(0, 0);\n}\n",
    "a {\n  b: calc(NaN);\n}\n"
);
