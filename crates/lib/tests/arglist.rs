#[macro_use]
mod macros;

test!(
    length_of_empty_arglist,
    "@mixin foo($a...) {\n    color: length($list: $a);\n}\na {\n    @include foo;\n}\n",
    "a {\n  color: 0;\n}\n"
);
test!(
    length_of_arglist_in_mixin,
    "@mixin foo($a...) {\n    color: length($list: $a);\n}\na {\n    @include foo(a, 2, c);\n}\n",
    "a {\n  color: 3;\n}\n"
);
test!(
    arglist_in_at_each,
    "@function sum($numbers...) {
        $sum: 0;
    
        @each $number in $numbers {
            $sum: $sum + $number;
        }
        @return $sum;
    }
    
    a {
        width: sum(50px, 30px, 100px);
    }",
    "a {\n  width: 180px;\n}\n"
);
error!(
    emit_empty_arglist,
    "@function foo($a...) {
        @return $a;
    }
    
    a {
        color: foo();
    }",
    "Error: () isn't a valid CSS value."
);
test!(
    inspect_empty_arglist,
    "@function foo($a...) {
        @return inspect($a);
    }
    
    a {
        color: foo();
    }",
    "a {\n  color: ();\n}\n"
);
test!(
    empty_arglist_is_allowed_in_map_functions,
    "@function foo($a...) {
        @return map-get($map: $a, $key: foo);
    }

    a {
        color: inspect(foo());
    }",
    "a {\n  color: null;\n}\n"
);
test!(
    inspect_arglist_with_one_arg,
    "@function foo($a...) {
        @return inspect($a);
    }
    
    a {
        color: inspect(foo(1));
    }",
    "a {\n  color: (1,);\n}\n"
);
error!(
    empty_arglist_is_error,
    "@function foo($a...) {
        @return $a;
    }

    a {
        color: foo();
    }",
    "Error: () isn't a valid CSS value."
);
test!(
    arglist_of_only_null_is_null,
    "@function foo($a...) {
        @return $a;
    }
    a {
        color: foo(null, null);
    }",
    ""
);
test!(
    keyword_args_no_positional,
    "@mixin foo($a...) {
        pos: inspect($a);
        kw: inspect(keywords($a));
    }

    a {
        @include foo($a: b);
    }",
    "a {\n  pos: ();\n  kw: (a: b);\n}\n"
);
test!(
    keyword_args_one_positional,
    "@mixin foo($a...) {
        pos: inspect($a);
        kw: inspect(keywords($a));
    }

    a {
        @include foo(a, $b: c);
    }",
    "a {\n  pos: (a,);\n  kw: (b: c);\n}\n"
);
test!(
    keyword_args_length_no_positional,
    "@mixin foo($a...) {
        pos: length($a);
        kw: length(keywords($a));
    }

    a {
        @include foo($a: b);
    }",
    "a {\n  pos: 0;\n  kw: 1;\n}\n"
);
error!(
    keyword_args_no_positional_is_invalid,
    "@mixin foo($a...) {
        pos: $a;
        kw: length(keywords($a));
    }

    a {
        @include foo($a: b);
    }",
    "Error: () isn't a valid CSS value."
);

// An arglist takes the separator of the list splatted into it, and a comma
// when nothing decided one: an arglist is a list, and `foo(1, 2, (3 4 5)...)`
// gives `$b: 2 3 4 5`, not `2, 3, 4, 5`. The separator then decides how the
// arglist prints, what `list.separator` reports, what `list.join` picks under
// `$separator: auto`, and what it compares equal to. dart-sass 1.103.1
// produced every expectation below; the fixtures are
// `non_conformant/scss-tests/{071,090}_*` and
// `non_conformant/sass/var-args/success`.
test!(
    splatting_a_space_list_gives_a_space_separated_arglist,
    "@mixin foo($a, $b...) {\n  a: $a;\n  b: $b;\n}\n$list: 3 4 5;\n.foo {@include foo(1, 2, $list...)}\n",
    ".foo {\n  a: 1;\n  b: 2 3 4 5;\n}\n"
);
test!(
    splatting_a_comma_list_gives_a_comma_separated_arglist,
    "@mixin foo($a, $b...) {\n  b: $b;\n}\n$list: (3, 4, 5);\n.foo {@include foo(1, 2, $list...)}\n",
    ".foo {\n  b: 2, 3, 4, 5;\n}\n"
);
// No splat, so nothing decided a separator and it falls back to a comma.
test!(
    an_arglist_with_no_splat_is_comma_separated,
    "@mixin foo($a, $b...) {\n  b: $b;\n}\n.foo {@include foo(1, 2, 3)}\n",
    ".foo {\n  b: 2, 3;\n}\n"
);
test!(
    a_function_s_arglist_keeps_the_separator_too,
    "@function foo($a, $b...) {\n  @return \"a: #{$a}, b: #{$b}\";\n}\n$list: 3 4 5;\n.foo {val: foo(1, 2, $list...)}\n",
    ".foo {\n  val: \"a: 1, b: 2 3 4 5\";\n}\n"
);
test!(
    list_separator_reports_the_arglist_s_own_separator,
    "@use \"sass:list\";\n@mixin m($a...) {\n  sep: list.separator($a);\n}\n.a {@include m((1 2 3)...)}\n.b {@include m((1, 2, 3)...)}\n",
    ".a {\n  sep: space;\n}\n\n.b {\n  sep: comma;\n}\n"
);
test!(
    list_join_takes_the_arglist_s_separator_under_auto,
    "@use \"sass:list\";\n@mixin m($a...) {\n  join: list.join($a, (9, 10));\n}\n.a {@include m((1 2 3)...)}\n",
    ".a {\n  join: 1 2 3 9 10;\n}\n"
);
// An arglist inside a map needs parentheses only when it is comma-separated,
// on the same rule a plain list follows.
test!(
    a_space_separated_arglist_needs_no_parentheses_in_a_map,
    "@use \"sass:meta\";\n@mixin m($a...) {\n  in-map: meta.inspect((k: $a));\n}\n.a {@include m((1 2 3)...)}\n.b {@include m((1, 2, 3)...)}\n",
    ".a {\n  in-map: (k: 1 2 3);\n}\n\n.b {\n  in-map: (k: (1, 2, 3));\n}\n"
);
// Equality compares the separator and the brackets, in both directions.
test!(
    an_arglist_equals_a_list_with_the_same_separator,
    "@mixin m($a...) {\n  eq-space: ($a == (1 2 3));\n  eq-comma: ($a == (1, 2, 3));\n  eq-rev-space: ((1 2 3) == $a);\n  eq-rev-comma: ((1, 2, 3) == $a);\n  eq-bracket: ($a == [1, 2, 3]);\n}\n.a {@include m((1 2 3)...)}\n",
    ".a {\n  eq-space: true;\n  eq-comma: false;\n  eq-rev-space: true;\n  eq-rev-comma: false;\n  eq-bracket: false;\n}\n"
);
// An empty arglist is comma-separated and `()` is undecided, so the two are
// not equal even though both are empty.
test!(
    an_empty_arglist_does_not_equal_an_empty_list,
    "@mixin m($a...) {\n  eq: ($a == ());\n}\n.a {@include m()}\n",
    ".a {\n  eq: false;\n}\n"
);
