#[macro_use]
mod macros;

test!(
    map_set_nested_empty,
    "@use 'sass:map'; a {b: inspect(map.set((c: ()), c, d, e, f))}",
    "a {\n  b: (c: (d: (e: f)));\n}\n"
);
test!(
    map_set_update_existing,
    "@use 'sass:map'; a {b: inspect(map.set((c: (d: e)), c, d, f))}",
    "a {\n  b: (c: (d: f));\n}\n"
);
test!(
    map_set_new_key,
    "@use 'sass:map'; a {b: inspect(map.set((c: (d: e)), c, f, g))}",
    "a {\n  b: (c: (d: e, f: g));\n}\n"
);
test!(
    map_set_value_is_not_map,
    "@use 'sass:map'; a {b: inspect(map.set((c: 1), c, d, f))}",
    "a {\n  b: (c: (d: f));\n}\n"
);
test!(
    map_merge_merge_into_map_with_many_keys,
    r#"
    @use "sass:map";

    $fonts: (
      "Helvetica": (
        "weights": (
          "regular": 400,
          "medium": 500,
          "bold": 700
        )
      )
    );

    a {
        color: inspect(map.merge($fonts, "Helvetica", "weights", "regular", (a: 300)));
    }"#,
    "a {\n  color: (\"Helvetica\": (\"weights\": (\"regular\": (a: 300), \"medium\": 500, \"bold\": 700)));\n}\n"
);
test!(
    map_merge_nested,
    r#"
    @use "sass:map";

    $fonts: (
      "Helvetica": (
        "weights": (
          "regular": 400,
          "medium": 500,
          "bold": 700
        )
      )
    );

    a {
        color: inspect(map.set($fonts, "Helvetica", "weights", "regular", 300));
    }"#,
    "a {\n  color: (\"Helvetica\": (\"weights\": (\"regular\": 300, \"medium\": 500, \"bold\": 700)));\n}\n"
);
test!(
    deep_merge_no_nesting,
    r#"@use "sass:map";

    a {
        color: inspect(map.deep-merge($map1: (c: d), $map2: (1: 2)));
    }"#,
    "a {\n  color: (c: d, 1: 2);\n}\n"
);
test!(
    deep_merge_positional,
    r#"@use "sass:map";

    a {
        color: inspect(map.deep-merge((a: b), (c: d)));
    }"#,
    "a {\n  color: (a: b, c: d);\n}\n"
);
test!(
    deep_merge_empty_maps,
    r#"@use "sass:map";

    a {
        color: inspect(map.deep-merge((), ()));
    }"#,
    "a {\n  color: ();\n}\n"
);
test!(
    deep_merge_empty_maps_bracketed_list,
    r#"@use "sass:map";

    a {
        color: inspect(map.deep-merge([], []));
    }"#,
    "a {\n  color: ();\n}\n"
);
test!(
    deep_merge_empty_first,
    r#"@use "sass:map";

    a {
        color: inspect(map.deep-merge((a: b), ()));
    }"#,
    "a {\n  color: (a: b);\n}\n"
);
test!(
    deep_merge_empty_second,
    r#"@use "sass:map";

    a {
        color: inspect(map.deep-merge((), (a: b)));
    }"#,
    "a {\n  color: (a: b);\n}\n"
);
test!(
    deep_merge_empty_deep,
    r#"@use "sass:map";

    a {
        color: inspect(map.deep-merge((c: (d: e)), (c: ())));
    }"#,
    "a {\n  color: (c: (d: e));\n}\n"
);
test!(
    deep_merge_empty_bracketed_list_deep,
    r#"@use "sass:map";

    a {
        color: inspect(map.deep-merge((c: (d: e)), (c: [])));
    }"#,
    "a {\n  color: (c: (d: e));\n}\n"
);
test!(
    deep_remove_key_dne,
    r#"@use "sass:map";

    a {
        color: inspect(map.deep-remove((a: b), 1));
    }"#,
    "a {\n  color: (a: b);\n}\n"
);
test!(
    deep_remove_nested_remove,
    r#"@use "sass:map";

    a {
        color: inspect(map.deep-remove((c: (d: e)), c, d));
    }"#,
    "a {\n  color: (c: ());\n}\n"
);
test!(
    deep_remove_nested_keys_dne,
    r#"@use "sass:map";

    a {
        color: inspect(map.deep-remove((c: (d: e)), c, d, e, f, g));
    }"#,
    "a {\n  color: (c: (d: e));\n}\n"
);
// The builtins are checked against the parameter lists dart-sass 1.103.1
// declares. Each expectation below was verified against it.
//
// `map.remove` is `$map | $map, $key, $keys...`, so `$key` is a parameter
// and reaches the function by name.
test!(
    remove_key_by_name,
    r#"@use "sass:map";
    a {
        color: inspect(map.remove($map: (c: d), $key: c));
    }"#,
    "a {\n  color: ();\n}\n"
);
test!(
    remove_key_by_name_after_positional_map,
    r#"@use "sass:map";
    a {
        color: inspect(map.remove((c: d, e: f), $key: c));
    }"#,
    "a {\n  color: (e: f);\n}\n"
);
// `$keys` is the rest parameter, which cannot be passed by name.
error!(
    remove_rest_parameter_by_name,
    r#"@use "sass:map";
    a {
        color: inspect(map.remove((c: d), $keys: c));
    }"#,
    "Error: No parameter named $keys."
);
// Three positional arguments and a stray name match `$map, $args...`, which
// reads no names.
error!(
    set_args_form_rejects_unknown_name,
    r#"@use "sass:map";
    a {
        color: inspect(map.set((c: d), c, e, $x: 1));
    }"#,
    "Error: No parameter named $x."
);
// `$map2` after a second positional argument is not the two-map form, so the
// call is `$map1, $args...` with a single argument.
error!(
    merge_args_form_needs_a_map,
    r#"@use "sass:map";
    a {
        color: inspect(map.merge((a: b), c, $map2: (d: e)));
    }"#,
    "Error: Expected $args to contain a map."
);
