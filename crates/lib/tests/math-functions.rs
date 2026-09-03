//! CSS math functions and calculation constants.
//!
//! Every expectation here was compared against the dart-sass 1.103.1 binary
//! before being committed.

#[macro_use]
mod macros;

test!(
    round_one_argument,
    "a {\n  b: round(1.5);\n}\n",
    "a {\n  b: 2;\n}\n"
);
test!(
    round_two_arguments,
    "a {\n  b: round(10px, 3px);\n}\n",
    "a {\n  b: 9px;\n}\n"
);
test!(
    round_strategy_up,
    "a {\n  b: round(up, 2.2, 1);\n}\n",
    "a {\n  b: 3;\n}\n"
);
// `to-zero` rounds the quotient by the number's sign, so a negative step can
// take the result further from zero than `down` would.
test!(
    round_strategy_to_zero_negative_step,
    "a {\n  b: round(to-zero, -120px, -25px);\n}\n",
    "a {\n  b: -125px;\n}\n"
);
test!(
    round_unsimplified_keeps_strategy,
    "a {\n  b: round(nearest, var(--c), 1px);\n}\n",
    "a {\n  b: round(nearest, var(--c), 1px);\n}\n"
);
// A single argument the calculation cannot take still reaches the Sass
// function, which allows a unitless number beside a length.
test!(
    round_falls_back_to_sass_function,
    "a {\n  b: round(1 + 1.5px);\n}\n",
    "a {\n  b: 3px;\n}\n"
);
error!(
    round_bad_strategy,
    "a {\n  b: round(bogus, 1px, 2px);\n}\n",
    "Error: bogus must be either nearest, up, down or to-zero."
);
error!(
    round_strategy_without_step,
    "a {\n  b: round(nearest, 5);\n}\n", "Error: If strategy is not null, step is required."
);

// `mod()` follows the divisor's sign, `rem()` the dividend's.
test!(
    mod_positive,
    "a {\n  b: mod(10, 3);\n}\n",
    "a {\n  b: 1;\n}\n"
);
test!(
    mod_negative_divisor,
    "a {\n  b: mod(10, -3);\n}\n",
    "a {\n  b: -2;\n}\n"
);
test!(
    rem_negative_dividend,
    "a {\n  b: rem(-10, 3);\n}\n",
    "a {\n  b: -1;\n}\n"
);
error!(
    mod_unitless_and_real,
    "a {\n  b: mod(16px, 5);\n}\n", "Error: 16px and 5 are incompatible."
);

test!(
    hypot_same_unit,
    "a {\n  b: hypot(3px, 4px);\n}\n",
    "a {\n  b: 5px;\n}\n"
);
// A percentage cannot be resolved at compile time, so it is left to the browser.
test!(
    hypot_percent_unsimplified,
    "a {\n  b: hypot(3%, 4%);\n}\n",
    "a {\n  b: hypot(3%, 4%);\n}\n"
);

test!(pow_simple, "a {\n  b: pow(2, 3);\n}\n", "a {\n  b: 8;\n}\n");
test!(sqrt_simple, "a {\n  b: sqrt(9);\n}\n", "a {\n  b: 3;\n}\n");
test!(
    log_with_base,
    "a {\n  b: log(8, 2);\n}\n",
    "a {\n  b: 3;\n}\n"
);
test!(exp_zero, "a {\n  b: exp(0);\n}\n", "a {\n  b: 1;\n}\n");
error!(
    sqrt_with_units,
    "a {\n  b: sqrt(9px);\n}\n", "Error: Expected 9px to have no units."
);
error!(
    pow_missing_argument,
    "a {\n  b: pow(1);\n}\n", "Error: 2 arguments required, but only 1 was passed."
);

test!(
    sin_degrees,
    "a {\n  b: sin(30deg);\n}\n",
    "a {\n  b: 0.5;\n}\n"
);
test!(cos_zero, "a {\n  b: cos(0);\n}\n", "a {\n  b: 1;\n}\n");
test!(
    tan_degrees,
    "a {\n  b: tan(45deg);\n}\n",
    "a {\n  b: 1;\n}\n"
);
test!(asin_one, "a {\n  b: asin(1);\n}\n", "a {\n  b: 90deg;\n}\n");
test!(acos_one, "a {\n  b: acos(1);\n}\n", "a {\n  b: 0deg;\n}\n");
test!(atan_one, "a {\n  b: atan(1);\n}\n", "a {\n  b: 45deg;\n}\n");
test!(
    atan2_equal,
    "a {\n  b: atan2(1, 1);\n}\n",
    "a {\n  b: 45deg;\n}\n"
);
error!(
    sin_without_angle_unit,
    "a {\n  b: sin(30px);\n}\n",
    "Error: $number: Expected 30px to have an angle unit (deg, grad, rad, turn)."
);
error!(
    sin_too_many_arguments,
    "a {\n  b: sin(1, 2);\n}\n", "Error: Only 1 argument allowed, but 2 were passed."
);

test!(
    abs_preserves_unit,
    "a {\n  b: abs(-3px);\n}\n",
    "a {\n  b: 3px;\n}\n"
);
test!(
    sign_negative,
    "a {\n  b: sign(-5.6);\n}\n",
    "a {\n  b: -1;\n}\n"
);
// `sign()` keeps the argument's unit, so the sign can be multiplied back out.
test!(
    sign_preserves_units,
    "a {\n  b: sign(-7px / 4em) * 1em;\n}\n",
    "a {\n  b: -1px;\n}\n"
);

// Constants are only constants inside a calculation, and are matched
// case-insensitively.
test!(
    constant_pi,
    "a {\n  b: calc(pi);\n}\n",
    "a {\n  b: 3.1415926536;\n}\n"
);
test!(
    constant_e,
    "a {\n  b: calc(e);\n}\n",
    "a {\n  b: 2.7182818285;\n}\n"
);
test!(
    constant_infinity,
    "a {\n  b: calc(infinity);\n}\n",
    "a {\n  b: calc(infinity);\n}\n"
);
test!(
    constant_minus_infinity,
    "a {\n  b: calc(-infinity);\n}\n",
    "a {\n  b: calc(-infinity);\n}\n"
);
test!(
    constant_nan,
    "a {\n  b: calc(NaN);\n}\n",
    "a {\n  b: calc(NaN);\n}\n"
);
test!(
    constant_case_insensitive,
    "a {\n  b: calc(InFiNiTy);\n}\n",
    "a {\n  b: calc(infinity);\n}\n"
);
test!(
    constant_outside_calculation_is_an_identifier,
    "a {\n  b: infinity;\n}\n",
    "a {\n  b: infinity;\n}\n"
);
// A degenerate number inside a calculation is written inline, not as a nested
// `calc()`.
test!(
    infinity_inlined_in_calculation,
    "a {\n  b: calc(infinity * (1% + 1px));\n}\n",
    "a {\n  b: calc(infinity * (1% + 1px));\n}\n"
);
// A `calc(infinity)` channel flows through the color functions.
test!(
    infinite_color_channel,
    "a {\n  b: rgb(calc(infinity), 0, 0, 0.5);\n}\n",
    "a {\n  b: rgba(255, 0, 0, 0.5);\n}\n"
);

test!(
    bare_identifier_in_calculation,
    "a {\n  b: calc(1px + foo);\n}\n",
    "a {\n  b: calc(1px + foo);\n}\n"
);
test!(
    calc_size_passthrough,
    "a {\n  b: calc-size(auto, size);\n}\n",
    "a {\n  b: calc-size(auto, size);\n}\n"
);
test!(
    calc_size_single_opaque_argument,
    "a {\n  b: calc-size(var(--foo));\n}\n",
    "a {\n  b: calc-size(var(--foo));\n}\n"
);
// Values written next to each other are legal when one side is opaque.
test!(
    calculation_space_separated,
    "a {\n  b: calc(var(--c) 1);\n}\n",
    "a {\n  b: calc(var(--c) 1);\n}\n"
);
error!(
    calculation_space_separated_all_known,
    "a {\n  b: calc(1 2);\n}\n", "Error: Missing math operator."
);
// Parentheses around opaque text are preserved; only the browser knows whether
// they matter.
test!(
    calculation_parens_around_opaque_value,
    "a {\n  b: calc((var(--c)));\n}\n",
    "a {\n  b: calc((var(--c)));\n}\n"
);
test!(
    unsimplified_math_function,
    "a {\n  b: sqrt(var(--a));\n}\n",
    "a {\n  b: sqrt(var(--a));\n}\n"
);
// A user-defined function shadows the CSS math function of the same name.
test!(
    user_function_overrides_math_function,
    "@function sin($arg) {@return $arg}\na {\n  b: sin(1);\n}\n",
    "a {\n  b: 1;\n}\n"
);
// `abs()` and `round()` are Sass functions first, so a slash beside them is
// division rather than the separator a real calculation keeps.
test!(
    slash_after_abs_is_division,
    "b {\n  a: 2px / abs(1.5);\n}\n",
    "b {\n  a: 1.3333333333px;\n}\n"
);
