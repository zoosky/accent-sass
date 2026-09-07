//! Interpolation inside a calculation.
//!
//! A calculation whose whole argument is one interpolation keeps that argument
//! as opaque text: the interpolated value reaches the output verbatim and
//! nothing about it is simplified. An interpolation that only appears inside a
//! larger expression does not -- that argument is parsed as an operation with
//! the interpolation as one operand. Which of the two applies decides whether
//! the source's whitespace survives and whether the operation keeps the
//! parentheses its precedence requires once a surrounding `calc()` is dropped.
//!
//! An interpolation written against an identifier is part of that identifier
//! and not an operand of its own, which is what makes `calc(x#{$a})` print
//! `x1`. Such an identifier is opaque text, so it names neither a calc
//! constant nor a function.
//!
//! Every expectation here was compared against the dart-sass 1.103.1 binary
//! before being committed.

#[macro_use]
mod macros;

test!(
    // The whole argument is one interpolation, so the text is opaque: the two
    // spaces the interpolated string carries reach the output as written.
    lone_interpolation_keeps_its_whitespace,
    "a {\n  b: calc(#{\"1px  +  2px\"});\n}\n",
    "a {\n  b: calc(1px  +  2px);\n}\n"
);
test!(
    // Here the interpolation is one operand of a parsed operation, so the
    // source's whitespace is re-serialized rather than carried through.
    interpolation_in_an_operation_normalizes_whitespace,
    "$a: 1px;\na {\n  b: calc(#{$a}  +  2px);\n}\n",
    "a {\n  b: calc(1px + 2px);\n}\n"
);
test!(
    // A newline is whitespace like any other. Preserving it used to split the
    // declaration across two lines of output.
    interpolation_in_an_operation_collapses_a_newline,
    "$r: 0.5rem;\n$w: 2px;\na {\n  b: calc(#{$r} -\n        #{$w});\n}\n",
    "a {\n  b: calc(0.5rem - 2px);\n}\n"
);
test!(
    // Being a real operation is also what lets a negative right-hand side flip
    // the operator, which opaque text cannot do.
    interpolation_in_an_operation_flips_a_negative_operand,
    "$a: 2.5rem;\na {\n  b: calc(#{$a} + -0.5rem);\n}\n",
    "a {\n  b: calc(2.5rem - 0.5rem);\n}\n"
);

test!(
    // Inlining the nested `calc()` puts an addition where a division binds
    // tighter, so the parentheses have to stay: without them the declaration
    // means `2.5rem - (0.5rem / 2)`, a different length.
    nested_calc_operand_keeps_the_parens_its_precedence_needs,
    "$a: 2.5rem;\na {\n  b: calc(calc(#{$a} - 0.5rem) / 2);\n}\n",
    "a {\n  b: calc((2.5rem - 0.5rem) / 2);\n}\n"
);
test!(
    nested_calc_operand_keeps_the_parens_on_the_right_of_a_minus,
    "$a: 2.5rem;\na {\n  b: calc(2px - calc(#{$a} - 0.5rem));\n}\n",
    "a {\n  b: calc(2px - (2.5rem - 0.5rem));\n}\n"
);
test!(
    // Equal precedence on the left of the operator needs none, and dart-sass
    // emits the flatter form.
    nested_calc_operand_drops_redundant_parens,
    "$a: 2.5rem;\na {\n  b: calc(calc(#{$a} - 0.5rem) - 2px);\n}\n",
    "a {\n  b: calc(2.5rem - 0.5rem - 2px);\n}\n"
);
test!(
    nested_calc_operand_of_a_division_needs_none_either,
    "$a: 2.5rem;\na {\n  b: calc(calc(#{$a} / 2) * 3px);\n}\n",
    "a {\n  b: calc(2.5rem / 2 * 3px);\n}\n"
);

test!(
    // Opaque text gets parentheses on the character that could bind to a
    // neighbour, which for an inlined `calc()` means whitespace, `*` or `/`.
    inlined_opaque_text_is_parenthesized_around_whitespace,
    "a {\n  b: calc(calc(#{\"1px + 2px\"}) * 3);\n}\n",
    "a {\n  b: calc((1px + 2px) * 3);\n}\n"
);
test!(
    inlined_opaque_text_is_parenthesized_around_a_slash,
    "a {\n  b: calc(calc(#{\"1px/2\"}) * 3);\n}\n",
    "a {\n  b: calc((1px/2) * 3);\n}\n"
);
test!(
    // A `+` or `-` is only an operator with whitespace on both sides, so one
    // written tight against its operands cannot bind across the splice.
    inlined_opaque_text_ignores_a_tight_sign,
    "a {\n  b: calc(calc(#{\"-1px\"}) * 3);\n}\n",
    "a {\n  b: calc(-1px * 3);\n}\n"
);
test!(
    // Nor can a comma, a bracket, or a parenthesis: text that is already
    // parenthesized needs no second pair.
    inlined_opaque_text_ignores_brackets_and_commas,
    "a {\n  b: calc(calc(#{\"[a]\"}) * 3);\n  c: calc(calc(#{\"1,2\"}) * 3);\n  d: calc(calc(#{\"(a)\"}) * 3);\n}\n",
    "a {\n  b: calc([a] * 3);\n  c: calc(1,2 * 3);\n  d: calc((a) * 3);\n}\n"
);

test!(
    // Adjacency binds looser than any operator, so a space-separated operand
    // is always grouped.
    space_separated_operand_is_parenthesized,
    "$a: 2.5rem;\na {\n  b: calc(calc(#{$a} 0.5rem) * 3px);\n}\n",
    "a {\n  b: calc((2.5rem 0.5rem) * 3px);\n}\n"
);

// An interpolation that is part of a larger identifier binds to it. The
// operand is the identifier's whole text, so `x#{$a}` is `x1` rather than `x`
// next to `1`, and `-#{$a}` is one value rather than a minus sign.

test!(
    interpolation_after_an_identifier_stays_one_operand,
    "$a: 1;\na {\n  b: calc(x#{$a} + 2px);\n}\n",
    "a {\n  b: calc(x1 + 2px);\n}\n"
);
test!(
    interpolation_between_two_pieces_of_an_identifier,
    "$a: 1;\na {\n  b: calc(a#{$a}b + 2px);\n}\n",
    "a {\n  b: calc(a1b + 2px);\n}\n"
);
test!(
    interpolation_after_a_custom_property_prefix,
    "$a: x;\na {\n  b: calc(--#{$a} + 2px);\n}\n",
    "a {\n  b: calc(--x + 2px);\n}\n"
);
test!(
    // The leading `-` belongs to the interpolated value, not to an operator,
    // so this is a division of one operand and not a parse error.
    interpolation_after_a_leading_minus,
    "$g: 1rem;\na {\n  b: calc(-#{$g} / 2);\n}\n",
    "a {\n  b: calc(-1rem / 2);\n}\n"
);
test!(
    interpolation_after_an_identifier_survives_a_nested_calc,
    "$a: 1;\na {\n  b: calc(calc(x#{$a}) + 2px);\n}\n",
    "a {\n  b: calc(x1 + 2px);\n}\n"
);
test!(
    interpolation_after_an_identifier_inside_a_math_function,
    "$a: 1;\na {\n  b: min(1px, x#{$a});\n}\n",
    "a {\n  b: min(1px, x1);\n}\n"
);
test!(
    // Text with an interpolation in it is opaque, so it never names a calc
    // constant however it spells out: `e#{""}` is the string `e`, not Euler's
    // number.
    interpolated_identifier_is_never_a_constant,
    "a {\n  b: calc(e#{\"\"} + 1);\n  c: calc(#{\"pi\"} + 1);\n}\n",
    "a {\n  b: calc(e + 1);\n  c: calc(pi + 1);\n}\n"
);
error!(
    // For the same reason it never names a function, even when the text spells
    // one a calculation would otherwise accept.
    interpolated_identifier_is_never_a_function,
    "a {\n  b: calc(m#{\"in\"}(1px, 2px));\n}\n",
    "Error: This expression can't be used in a calculation."
);
error!(
    interpolated_identifier_is_never_a_namespace,
    "@use \"sass:math\";\na {\n  b: calc(mat#{\"h\"}.div(6px, 2));\n}\n",
    "Error: Interpolation isn't allowed in namespaces."
);

// Adjacency binds looser than any operator, so an operand that is a
// space-separated list always came from a grouping the source wrote.

test!(
    // `+` binds tighter than the space, so the list is `1` next to the
    // unsimplifiable `px + 2px` and needs no parentheses.
    adjacency_binds_looser_than_addition,
    "$a: 1;\na {\n  b: calc(#{$a} px + 2px);\n}\n",
    "a {\n  b: calc(1 px + 2px);\n}\n"
);
test!(
    // Which is also what lets the addition be simplified away.
    adjacency_lets_the_operator_simplify_first,
    "$a: 1px;\na {\n  b: calc(#{$a} 2px + 3px);\n}\n",
    "a {\n  b: calc(1px 5px);\n}\n"
);
test!(
    parenthesized_adjacency_keeps_its_parens,
    "a {\n  b: calc((var(--c) 1) + 2px);\n}\n",
    "a {\n  b: calc((var(--c) 1) + 2px);\n}\n"
);
