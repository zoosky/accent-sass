//! Selectors whose combinators are not valid CSS.
//!
//! dart-sass calls these *bogus*. The ones that nesting cannot repair -- two
//! combinators in a row, more than one leading combinator, a trailing
//! combinator on a rule with declarations -- are left out of the output, and
//! a *useless* one, which can never match, may not act as an extender. A
//! single leading combinator is legal CSS nesting and is kept. dart-sass
//! also emits a `bogus-combinators` deprecation warning for each; this
//! compiler has no warning facility, so these tests check the output only.
//!
//! Every expectation was verified against dart-sass 1.104.0.

#[macro_use]
mod macros;

test!(two_combinators_in_a_row_is_omitted, ".a > + x {p: 1}", "");
test!(two_leading_combinators_is_omitted, "> > .d {p: 4}", "");
test!(
    sibling_combinators_in_a_row_is_omitted,
    ".g + ~ .h {p: 6}",
    ""
);
test!(
    trailing_combinator_with_declarations_is_omitted,
    ".b > {p: 2}",
    ""
);
test!(
    single_leading_combinator_is_kept,
    "> .c {p: 3}",
    "> .c {\n  p: 3;\n}\n"
);
test!(
    nested_leading_combinator_is_resolved,
    ".e { > .f {p: 5} }",
    ".e > .f {\n  p: 5;\n}\n"
);
test!(
    trailing_combinator_with_style_rule_child_is_kept,
    ".b > {c {p: 2}}",
    ".b > c {\n  p: 2;\n}\n"
);
test!(
    trailing_combinator_drops_only_its_own_declarations,
    ".b > {c {p: 2} q: 1}",
    ".b > c {\n  p: 2;\n}\n"
);
test!(
    nested_trailing_combinators_are_omitted,
    ".x ~ { .y ~ { p: 1 } }",
    ""
);
test!(
    trailing_combinator_inside_media_is_omitted,
    ".b > {@media screen {p: 1}}",
    ""
);
test!(
    only_the_bogus_complex_selector_is_omitted,
    ".a + ~ b, .c {p: 1}",
    ".c {\n  p: 1;\n}\n"
);
test!(bogus_selector_in_is_is_omitted, ":is(.a > + b) {p: 1}", "");
test!(
    bogus_selector_in_not_is_omitted,
    ":not(.a > + b) {p: 1}\n.c:not(.d + ~ e) {p: 2}",
    ""
);
test!(leading_combinator_in_is_is_omitted, ".x:is(> a) {p: 1}", "");
test!(
    leading_combinator_in_has_is_kept,
    ".x:has(> a) {p: 1}",
    ".x:has(> a) {\n  p: 1;\n}\n"
);
test!(
    leading_combinator_extender_is_kept,
    "a {b: c}\n> d {@extend a}",
    "a, > d {\n  b: c;\n}\n"
);
test!(
    trailing_combinator_extender_is_omitted,
    "a {b: c}\nd + {@extend a}",
    "a {\n  b: c;\n}\n"
);
test!(
    useless_extender_is_refused,
    ".a x {a: b}\n.b > + y {@extend x}",
    ".a x {\n  a: b;\n}\n"
);
test!(
    useless_extender_needs_no_target,
    "> > b {@extend .missing}",
    ""
);
error!(
    trailing_combinator_extender_needs_a_target,
    "d + {@extend .missing}", "Error: The target selector was not found."
);
test!(
    selector_value_keeps_bogus_combinators,
    "@use \"sass:selector\"; a {b: selector.parse(\"a > + b\")}",
    "a {\n  b: a > + b;\n}\n"
);
test!(
    selector_extend_drops_useless_result,
    "@use \"sass:selector\"; a {b: selector.extend(\".c ~ ~ .d\", \".c\", \".e\")}",
    "a {\n  b: .c ~ ~ .d;\n}\n"
);
test!(
    selector_extend_with_trailing_combinator_extender,
    "@use \"sass:selector\"; a {b: selector.extend(\".c .d\", \".c\", \".e >\")}",
    "a {\n  b: .c .d, .e > .d;\n}\n"
);
test!(
    selector_extend_unifies_trailing_combinator,
    "@use \"sass:selector\"; a {b: selector.extend(\".c.x .d\", \".c\", \".e >\")}",
    "a {\n  b: .c.x .d, .x.e > .d;\n}\n"
);
test!(
    selector_extend_unifies_leading_combinator,
    "@use \"sass:selector\"; a {b: selector.extend(\".c.x .d\", \".c\", \"> .e\")}",
    "a {\n  b: .c.x .d, > .x.e .d;\n}\n"
);
test!(
    bogus_selector_is_not_a_subselector,
    "@use \"sass:selector\"; a {b: selector.is-superselector(\"a\", \"a > + b\")}",
    "a {\n  b: false;\n}\n"
);
test!(
    bogus_argument_inside_is_omits_the_rule,
    ":is(.a > + .b, .c) {p: 1}",
    ""
);
// dart-sass hides bogus selectors only when writing CSS. In a SassScript
// value, the argument of a selector pseudo is printed in full.
test!(
    selector_value_keeps_bogus_argument_of_is,
    "@use \"sass:selector\"; a {b: selector.parse(\":is(.a > + .b)\")}",
    "a {\n  b: :is(.a > + .b);\n}\n"
);
test!(
    selector_value_keeps_every_argument_of_is,
    "@use \"sass:selector\"; a {b: selector.parse(\":is(.a > + .b, .c)\")}",
    "a {\n  b: :is(.a > + .b, .c);\n}\n"
);
test!(
    selector_value_keeps_trailing_combinator_in_where,
    "@use \"sass:selector\"; a {b: selector.parse(\":where(.b >)\")}",
    "a {\n  b: :where(.b >);\n}\n"
);
test!(
    selector_value_keeps_leading_combinators_in_has,
    "@use \"sass:selector\"; a {b: selector.parse(\":has(> > .b)\")}",
    "a {\n  b: :has(> > .b);\n}\n"
);
test!(
    selector_value_keeps_nested_bogus_argument,
    "@use \"sass:selector\"; a {b: selector.parse(\":is(:is(.a > + .b))\")}",
    "a {\n  b: :is(:is(.a > + .b));\n}\n"
);
test!(
    selector_extend_leaves_useless_argument_alone,
    "@use \"sass:selector\"; a {b: selector.extend(\":is(> > .d)\", \".d\", \".e\")}",
    "a {\n  b: :is(> > .d);\n}\n"
);
test!(
    selector_replace_keeps_what_survives,
    "@use \"sass:selector\"; a {b: selector.replace(\".c.x ~ ~ .d, e\", \".c\", \".e\")}",
    "a {\n  b: e;\n}\n"
);
error!(
    selector_replace_that_leaves_nothing_is_an_error,
    "@use \"sass:selector\"; a {b: selector.replace(\".c.x ~ ~ .d\", \".c\", \".e\")}",
    "Error: Invalid argument(s): components may not be empty."
);
error!(
    selector_replace_that_unifies_with_nothing_is_an_error,
    "@use \"sass:selector\"; a {b: selector.replace(\"a.b\", \".b\", \"c\")}",
    "Error: Invalid argument(s): components may not be empty."
);
