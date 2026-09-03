//! Selector unification ordering and the rules that constrain it.
//!
//! Every expectation here was compared against the dart-sass 1.103.1 binary
//! before being committed.

#[macro_use]
mod macros;

// The first operand's simple selectors keep their places; the second operand's
// are merged in around them.
test!(
    complex_keeps_first_operand_order,
    "@use \"sass:selector\";\na {\n  b: selector.unify(\".c > .d\", \".e > .f\");\n}\n",
    "a {\n  b: .c.e > .d.f;\n}\n"
);
test!(
    overlapping_compounds_keep_order,
    "@use \"sass:selector\";\na {\n  b: selector.unify(\".c.s1-1 > .s1-2\", \".c.s2-1 > .s2-2\");\n}\n",
    "a {\n  b: .c.s1-1.s2-1 > .s1-2.s2-2;\n}\n"
);
// The `~` selector leads, so it leads in the unified compound too.
test!(
    next_sibling_and_sibling_order,
    "@use \"sass:selector\";\na {\n  b: selector.unify(\".c + .d\", \".e ~ .f\");\n}\n",
    "a {\n  b: .e ~ .c + .d.f, .e.c + .d.f;\n}\n"
);

// A pseudo-element and the pseudo-classes written after it stay together at the
// end of the compound.
test!(
    pseudo_element_tail_is_not_split,
    "@use \"sass:selector\";\na {\n  b: selector.unify(\".x\", \".y::scrollbar:horizontal\");\n}\n",
    "a {\n  b: .x.y::scrollbar:horizontal;\n}\n"
);
test!(
    same_pseudo_element_merges_its_classes,
    "@use \"sass:selector\";\na {\n  b: selector.unify(\"::foo:bar\", \"::foo:baz\");\n}\n",
    "a {\n  b: ::foo:bar:baz;\n}\n"
);
// A compound may carry only one pseudo-element.
test!(
    different_pseudo_elements_do_not_unify,
    "@use \"sass:meta\";\n@use \"sass:selector\";\na {\n  b: meta.inspect(selector.unify(\"::foo:bar\", \"::other:baz\"));\n}\n",
    "a {\n  b: null;\n}\n"
);

// `:host` matches a shadow root, so it only combines with other selectors that
// can: another host selector, or a pseudo-class taking a selector argument.
test!(
    host_unifies_with_selector_pseudo,
    "@use \"sass:selector\";\na {\n  b: selector.unify(\":host\", \":is(.c)\");\n}\n",
    "a {\n  b: :is(.c):host;\n}\n"
);
test!(
    host_does_not_unify_with_class,
    "@use \"sass:meta\";\n@use \"sass:selector\";\na {\n  b: meta.inspect(selector.unify(\":host\", \".c\"));\n}\n",
    "a {\n  b: null;\n}\n"
);
test!(
    host_does_not_unify_with_universal,
    "@use \"sass:meta\";\n@use \"sass:selector\";\na {\n  b: meta.inspect(selector.unify(\"*\", \":host\"));\n}\n",
    "a {\n  b: null;\n}\n"
);
// `:host` matches the outermost element of its tree, so nothing can be woven in
// front of it.
test!(
    host_stays_at_the_start,
    "@use \"sass:selector\";\na {\n  b: selector.unify(\":host .c\", \".d .e\");\n}\n",
    "a {\n  b: :host .d .c.e;\n}\n"
);

// The universal and type selectors subsume narrower namespaces.
test!(
    universal_namespace_is_a_superselector,
    "@use \"sass:selector\";\na {\n  b: selector.is-superselector(\"*|*\", \".d\");\n}\n",
    "a {\n  b: true;\n}\n"
);
test!(
    universal_namespace_type_is_a_superselector,
    "@use \"sass:selector\";\na {\n  b: selector.is-superselector(\"*|c\", \"d|c\");\n}\n",
    "a {\n  b: true;\n}\n"
);
test!(
    empty_namespace_matches_itself,
    "@use \"sass:selector\";\na {\n  b: selector.is-superselector(\"|*\", \"|d\");\n}\n",
    "a {\n  b: true;\n}\n"
);
test!(
    empty_namespace_is_not_implicit,
    "@use \"sass:selector\";\na {\n  b: selector.is-superselector(\"|*\", \"*\");\n}\n",
    "a {\n  b: false;\n}\n"
);
test!(
    explicit_namespace_is_not_universal,
    "@use \"sass:selector\";\na {\n  b: selector.is-superselector(\"c|d\", \"*|d\");\n}\n",
    "a {\n  b: false;\n}\n"
);

// A namespaced call names a member of that module and nothing else, even when a
// global function of the same name exists.
error!(
    namespaced_call_does_not_see_global_functions,
    "@use \"sass:selector\";\na {\n  b: selector.selector-append(c, d);\n}\n",
    "Error: Undefined function."
);
