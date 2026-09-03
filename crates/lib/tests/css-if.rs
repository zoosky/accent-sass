//! The CSS `if()` function.
//!
//! Every expectation here was compared against the dart-sass 1.103.1 binary
//! before being committed.

#[macro_use]
mod macros;

// A `sass()` condition is settled at compile time and the `if()` collapses to
// the matching branch.
test!(
    sass_true,
    "a {\n  b: if(sass(true): c; else: d);\n}\n",
    "a {\n  b: c;\n}\n"
);
test!(
    sass_false_takes_else,
    "a {\n  b: if(sass(false): c; else: d);\n}\n",
    "a {\n  b: d;\n}\n"
);
test!(
    sass_expression,
    "$a: true;\nb {\n  c: if(sass($a): d; else: e);\n}\n",
    "b {\n  c: d;\n}\n"
);
// With nothing left to match and no `else`, the result is null, so the
// declaration is elided.
test!(
    no_matching_branch_is_null,
    "a {\n  b: if(sass(false): c) == null;\n}\n",
    "a {\n  b: true;\n}\n"
);
test!(
    else_alone,
    "a {\n  b: if(else: c);\n}\n",
    "a {\n  b: c;\n}\n"
);

test!(
    not_inverts,
    "a {\n  b: if(not sass(true): c; else: d);\n}\n",
    "a {\n  b: d;\n}\n"
);
test!(
    and_chain,
    "a {\n  b: if(sass(true) and sass(false): c; else: d);\n}\n",
    "a {\n  b: d;\n}\n"
);
test!(
    or_chain,
    "a {\n  b: if(sass(false) or sass(true): c; else: d);\n}\n",
    "a {\n  b: c;\n}\n"
);

// A condition Sass cannot settle is emitted for the browser, with the branch
// values still evaluated.
test!(
    opaque_condition_passes_through,
    "a {\n  b: if(css(): c; else: d);\n}\n",
    "a {\n  b: if(css(): c; else: d);\n}\n"
);
// An operand that is statically true adds nothing to an `and`, so it is
// dropped rather than emitted.
test!(
    settled_operand_is_dropped_from_and,
    "a {\n  b: if(sass(true) and css(): c; else: d);\n}\n",
    "a {\n  b: if(css(): c; else: d);\n}\n"
);
test!(
    settled_operand_is_dropped_from_or,
    "a {\n  b: if(sass(false) or css(): c; else: d);\n}\n",
    "a {\n  b: if(css(): c; else: d);\n}\n"
);
// A substitution beside other terms is opaque as a whole: Sass cannot tell
// whether it contains an operator.
test!(
    substitution_run_is_opaque,
    "a {\n  b: if(css(1) var(--and) css(2): c);\n}\n",
    "a {\n  b: if(css(1) var(--and) css(2): c);\n}\n"
);
test!(
    parens_are_preserved,
    "a {\n  b: if((css(1) and css(2)): c);\n}\n",
    "a {\n  b: if((css(1) and css(2)): c);\n}\n"
);
test!(
    interpolated_condition,
    "a {\n  b: if(#{css()} and sass(true): c; else: d);\n}\n",
    "a {\n  b: if(css(): c; else: d);\n}\n"
);
test!(
    branch_value_keeps_quotes,
    "a {\n  b: if(css(): \"c\");\n}\n",
    "a {\n  b: if(css(): \"c\");\n}\n"
);
// Argument text reaches the browser exactly as written, so an empty
// single-quoted string is not re-quoted.
test!(
    opaque_argument_is_verbatim,
    "a {\n  b: if(css(''): c);\n}\n",
    "a {\n  b: if(css(''): c);\n}\n"
);

// Branches after a settled one are never looked at, so an undefined variable
// in one of them is not an error.
test!(
    later_branches_are_not_evaluated,
    "a {\n  b: if(sass(true): c; css(#{$undefined}): d);\n}\n",
    "a {\n  b: c;\n}\n"
);
test!(
    and_short_circuits,
    "a {\n  b: if(sass(false) and sass($undefined): c);\n}\n",
    ""
);
test!(
    or_short_circuits,
    "a {\n  b: if(sass(true) or sass($undefined): c);\n}\n",
    "a {\n  b: c;\n}\n"
);

// The Sass ternary keeps working, including with named arguments.
test!(
    legacy_ternary,
    "$a: true;\na {\n  b: if($a, 1, 2);\n}\n",
    "a {\n  b: 1;\n}\n"
);
test!(
    legacy_ternary_named_arguments,
    "a {\n  b: if($condition: true, $if-true: 1, $if-false: 2);\n}\n",
    "a {\n  b: 1;\n}\n"
);

// CSS rejects a keyword used as a function name rather than reinterpreting it.
error!(
    not_as_function_name,
    "a {\n  b: if(not(css()): d);\n}\n", "Error: Whitespace is required between \"not\" and \"(\""
);
// `and` and `or` may not mix without parentheses.
error!(
    mixed_operators,
    "a {\n  b: if(css(1) and css(2) or css(3): c);\n}\n", "Error: expected \":\"."
);
// `not` takes a single term, so it cannot head a chain.
error!(
    not_before_chain,
    "a {\n  b: if(css(1) and not css(2): c);\n}\n", "Error: expected \"(\"."
);
// A substitution could expand to anything, so Sass will not guess how it
// combines with a condition it has to resolve itself.
error!(
    substitution_run_with_sass,
    "a {\n  b: if(sass(true) var(--and-clause): c);\n}\n",
    "Error: if() conditions with arbitrary substitutions may not contain sass() expressions."
);
error!(
    else_inside_parens,
    "a {\n  b: if((else): c);\n}\n", "Error: expected \"(\"."
);
// Branches are separated by `;`, never by `,`.
error!(
    comma_between_branches,
    "a {\n  b: if(css(1): c, css(2): d);\n}\n", "Error: expected \")\"."
);
error!(
    empty_branch,
    "a {\n  b: if(css(1): c;; css(2): d);\n}\n", "Error: Expected identifier."
);
