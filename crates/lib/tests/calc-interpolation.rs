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
