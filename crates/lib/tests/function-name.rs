//! What a Sass `@function` may be called.
//!
//! The rules follow the [function-name proposal]: a name is reserved only when
//! a call to it could never reach the function, which is far narrower than the
//! blanket list this fork used to carry. Every expectation here was compared
//! against the dart-sass 1.103.1 binary before being committed.
//!
//! [function-name proposal]: https://github.com/sass/sass/tree/main/accepted/function-name.md

#[macro_use]
mod macros;

// `calc()` and `clamp()` are ordinary names: a user-defined function of either
// name wins over the calculation.
test!(
    calc_may_be_redefined,
    "@function calc() {@return 1}\na {b: calc()}\n",
    "a {\n  b: 1;\n}\n"
);
test!(
    clamp_may_be_redefined,
    "@function clamp() {@return 1}\na {b: clamp()}\n",
    "a {\n  b: 1;\n}\n"
);
test!(
    calc_is_still_a_calculation_when_undefined,
    "a {b: calc(1px + 2px)}\n",
    "a {\n  b: 3px;\n}\n"
);
test!(
    clamp_is_still_a_calculation_when_undefined,
    "a {b: clamp(1px, 2px, 3px)}\n",
    "a {\n  b: 2px;\n}\n"
);

// A vendor prefix rescues every reserved name but `element`, whose prefixed
// form has special parsing of its own.
test!(
    vendor_prefixed_and,
    "@function -a-and() {@return 1}\nb {c: -a-and()}\n",
    "b {\n  c: 1;\n}\n"
);
test!(
    vendor_prefixed_or,
    "@function -a-or() {@return 1}\nb {c: -a-or()}\n",
    "b {\n  c: 1;\n}\n"
);
test!(
    vendor_prefixed_not,
    "@function -a-not() {@return 1}\nb {c: -a-not()}\n",
    "b {\n  c: 1;\n}\n"
);
test!(
    vendor_prefixed_type,
    "@function -a-type() {@return 1}\nb {c: -a-type()}\n",
    "b {\n  c: 1;\n}\n"
);
// These two may be declared, but the call still parses as the special function
// rather than reaching the declaration.
test!(
    vendor_prefixed_expression,
    "@function -a-expression() {@return 1}\nb {\n  c: -a-expression();\n}\n",
    "b {\n  c: -a-expression();\n}\n"
);
test!(
    vendor_prefixed_url,
    "@function -a-url() {@return 1}\nb {\n  c: -a-url();\n}\n",
    "b {\n  c: url();\n}\n"
);

// The name is checked as written. `_` is not `-`, so neither of these is the
// vendor-prefixed `element()` the rule is about, even though both normalise to
// the same identifier.
test!(
    underscore_where_a_vendor_prefix_would_be,
    "@function _moz-element() {@return 1}\nb {c: _moz-element()}\n",
    "b {\n  c: 1;\n}\n"
);
test!(
    underscore_inside_a_vendor_prefix,
    "@function -moz_element() {@return 1}\nb {c: -moz_element()}\n",
    "b {\n  c: 1;\n}\n"
);
test!(
    leading_double_underscore,
    "@function __a() {@return 1}\nb {c: __a()}\n",
    "b {\n  c: 1;\n}\n"
);

// The checks are case-sensitive, except `type`, which is reserved outright.
test!(
    uppercase_element,
    "@function ELEMENT() {@return 1}\na {\n  b: ELEMENT();\n}\n",
    "a {\n  b: element();\n}\n"
);
test!(
    uppercase_and,
    "@function AND() {@return 1}\na {b: AND()}\n",
    "a {\n  b: 1;\n}\n"
);

error!(
    type_is_reserved,
    "@function type() {@return 1}\na {b: type()}\n",
    "Error: This name is reserved for the plain-CSS function."
);
error!(
    uppercase_type_is_reserved,
    "@function TYPE() {@return 1}\na {b: TYPE()}\n",
    "Error: This name is reserved for the plain-CSS function."
);
error!(
    element_is_invalid,
    "@function element() {@return 1}\n", "Error: Invalid function name."
);
error!(
    vendor_prefixed_element_is_invalid,
    "@function -a-element() {@return 1}\n", "Error: Invalid function name."
);
error!(
    expression_is_invalid,
    "@function expression() {@return 1}\n", "Error: Invalid function name."
);
error!(
    url_is_invalid,
    "@function url() {@return 1}\n", "Error: Invalid function name."
);
error!(
    and_is_invalid,
    "@function and() {@return 1}\n", "Error: Invalid function name."
);
error!(
    or_is_invalid,
    "@function or() {@return 1}\n", "Error: Invalid function name."
);
error!(
    not_is_invalid,
    "@function not() {@return 1}\n", "Error: Invalid function name."
);
