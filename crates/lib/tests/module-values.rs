//! First-class module values: `meta.get-module`, `meta.load` and `meta.css`,
//! added in dart-sass 1.105.0.
//!
//! Every expectation here was checked against the dart-sass 1.105.1 release
//! binary. The cases mirror fixtures under `sass-spec/spec/core_functions/
//! meta/{get_module,load,css}` or probe what those fixtures leave open.

use std::io::Write;

#[macro_use]
mod macros;

test!(
    type_of_a_module_is_module,
    "@use \"sass:meta\";\n@use \"sass:math\";\na {b: meta.type-of(meta.get-module(math))}",
    "a {\n  b: module;\n}\n"
);
test!(
    inspect_prints_the_default_namespace,
    "@use \"sass:meta\";\n@use \"sass:math\" as m;\na {b: meta.inspect(meta.get-module(m))}",
    "a {\n  b: get-module(\"math\");\n}\n"
);
test!(
    a_module_is_truthy,
    "@use \"sass:meta\";\n@use \"sass:math\";\n$m: meta.get-module(math);\na {b: not $m; c: if($m, yes, no)}",
    "a {\n  b: false;\n  c: yes;\n}\n"
);
test!(
    inspect_prints_a_module_inside_a_list_and_a_map,
    "@use \"sass:meta\";\n@use \"sass:math\";\n$m: meta.get-module(math);\na {b: meta.inspect(($m, 1)); c: meta.inspect((k: $m))}",
    "a {\n  b: get-module(\"math\"), 1;\n  c: (k: get-module(\"math\"));\n}\n"
);
// Two namespaces for one built-in module are one module, and so is loading
// it by URL. dart-sass keeps one instance per built-in module.
test!(
    builtin_module_aliases_are_one_module,
    "@use \"sass:meta\";\n@use \"sass:math\";\n@use \"sass:math\" as m;\na {\n  b: meta.get-module(math) == meta.get-module(m);\n  c: meta.get-module(math) == meta.load(\"sass:math\");\n  d: meta.get-module(math) != meta.get-module(m);\n  e: meta.get-module(math) == meta.get-module(meta);\n}",
    "a {\n  b: true;\n  c: true;\n  d: false;\n  e: false;\n}\n"
);
test!(
    module_value_as_module_argument,
    "@use \"sass:meta\";\n@use \"sass:math\";\n@use \"sass:color\";\na {\n  b: meta.global-variable-exists(\"e\", $module: meta.get-module(\"math\"));\n  c: meta.function-exists(\"red\", meta.get-module(\"color\"));\n  d: meta.function-exists(\"c\", meta.get-module(\"color\"));\n  e: meta.call(meta.get-function(\"abs\", $module: meta.get-module(math)), -1);\n}",
    "a {\n  b: true;\n  c: true;\n  d: false;\n  e: 1;\n}\n"
);
error!(
    a_module_is_not_a_css_value,
    "@use \"sass:meta\";\n@use \"sass:math\";\na {b: meta.get-module(math)}",
    "Error: get-module(\"math\") isn't a valid CSS value."
);
error!(
    a_module_is_not_a_css_value_in_interpolation,
    "@use \"sass:meta\";\n@use \"sass:math\";\na {b: #{meta.get-module(math)}}",
    "Error: get-module(\"math\") isn't a valid CSS value."
);
error!(
    adding_modules_is_undefined,
    "@use \"sass:meta\";\n@use \"sass:math\";\n$m: meta.get-module(math);\na {b: $m + $m}",
    "Error: Undefined operation \"get-module(\"math\") + get-module(\"math\")\"."
);
error!(
    a_module_on_the_right_of_minus_is_undefined,
    "@use \"sass:meta\";\n@use \"sass:math\";\na {b: 1 - meta.get-module(math)}",
    "Error: Undefined operation \"1 - get-module(\"math\")\"."
);
error!(
    a_module_on_the_right_of_a_string_minus_is_undefined,
    "@use \"sass:meta\";\n@use \"sass:math\";\na {b: \"x\" - meta.get-module(math)}",
    "Error: Undefined operation \"\"x\" - get-module(\"math\")\"."
);
// The one operator that serializes instead: a string's `+` writes its right
// operand as CSS, and a module has no CSS form.
error!(
    a_string_plus_a_module_serializes_the_module,
    "@use \"sass:meta\";\n@use \"sass:math\";\na {b: \"x\" + meta.get-module(math)}",
    "Error: get-module(\"math\") isn't a valid CSS value."
);
error!(
    dividing_by_a_module_is_undefined,
    "@use \"sass:meta\";\n@use \"sass:math\";\na {b: 1 / meta.get-module(math)}",
    "Error: Undefined operation \"1 / get-module(\"math\")\"."
);
error!(
    comparing_a_module_is_undefined,
    "@use \"sass:meta\";\n@use \"sass:math\";\na {b: meta.get-module(math) < 1}",
    "Error: Undefined operation \"get-module(\"math\") < 1\"."
);
// dart-sass parenthesizes the operand of unary minus and of nothing else.
error!(
    negating_a_module_is_undefined,
    "@use \"sass:meta\";\n@use \"sass:math\";\n$m: meta.get-module(math);\na {b: -$m}",
    "Error: Undefined operation \"-(get-module(\"math\"))\"."
);
error!(
    unary_plus_on_a_module_is_undefined,
    "@use \"sass:meta\";\n@use \"sass:math\";\na {b: +meta.get-module(math)}",
    "Error: Undefined operation \"+get-module(\"math\")\"."
);
error!(
    unary_slash_on_a_module_is_undefined,
    "@use \"sass:meta\";\n@use \"sass:math\";\na {b: /meta.get-module(math)}",
    "Error: Undefined operation \"/get-module(\"math\")\"."
);
error!(
    get_module_rejects_a_non_string,
    "@use \"sass:meta\";\na {b: meta.get-module(null)}",
    "Error: $module: null is neither a string nor a module reference."
);
error!(
    get_module_of_an_unknown_namespace,
    "@use \"sass:meta\";\na {b: meta.get-module(does-not-exist)}",
    "Error: There is no module with namespace \"does-not-exist\"."
);
error!(
    get_module_of_a_builtin_that_was_not_loaded,
    "@use \"sass:meta\";\na {b: meta.get-module(\"color\")}",
    "Error: There is no module with namespace \"color\"."
);
error!(
    get_module_of_a_wildcard_use,
    "@use \"sass:meta\";\n@use \"sass:math\" as *;\na {b: meta.get-module(\"math\")}",
    "Error: There is no module with namespace \"math\"."
);
// A namespace given as a string is looked up as written. Member access
// through the namespace stays dash-insensitive, which dart-sass is not.
error!(
    get_module_namespace_string_is_dash_sensitive,
    "@use \"sass:color\" as a-b;\n@use \"sass:meta\";\nc {d: meta.get-module(\"a_b\")}",
    "Error: There is no module with namespace \"a_b\"."
);
error!(
    module_functions_namespace_string_is_dash_sensitive,
    "@use \"sass:color\" as a-b;\n@use \"sass:meta\";\nc {d: meta.inspect(meta.module-functions(\"a_b\"))}",
    "Error: There is no module with namespace \"a_b\"."
);
error!(
    css_rejects_a_number,
    "@use \"sass:meta\";\n@include meta.css(1);",
    "Error: $module: 1 is neither a string nor a module reference."
);
error!(
    load_rejects_a_non_string_url,
    "@use \"sass:meta\";\n$_: meta.load(1);", "Error: $url: 1 is not a string."
);
error!(
    load_rejects_a_non_map_with,
    "@use \"sass:meta\";\n$_: meta.load(\"other\", $with: 1);", "Error: $with: 1 is not a map."
);
error!(
    load_cannot_configure_a_builtin_module,
    "@use \"sass:meta\";\n$_: meta.load(\"sass:color\", $with: (a: b));",
    "Error: Built-in module sass:color can't be configured."
);
error!(
    load_of_a_missing_file,
    "@use \"sass:meta\";\n$_: meta.load(\"mv_does_not_exist\");",
    "Error: Can't find stylesheet to import."
);

fn compile(input: &str) -> String {
    accent_sass::from_string(input.to_string(), &accent_sass::Options::default()).expect(input)
}

fn compile_err(input: &str) -> String {
    match accent_sass::from_string(input.to_string(), &accent_sass::Options::default()) {
        Ok(output) => panic!("did not fail: {output}"),
        Err(e) => e.to_string().chars().take_while(|c| *c != '\n').collect(),
    }
}

// `meta.load` returns the module, not a copy: a change made through a
// namespace shows in the value (spec `load/live`).
#[test]
fn load_returns_a_live_module() {
    let input = r#"@use "sass:meta";
@use "mv_live";
$module: meta.load("mv_live");
a {
  before: meta.inspect(meta.module-variables($module));
  mv_live.$value: b;
  after: meta.inspect(meta.module-variables($module));
}"#;
    tempfile!("_mv_live.scss", "$value: c;");
    assert_eq!(
        "a {\n  before: (\"value\": c);\n  after: (\"value\": b);\n}\n",
        compile(input)
    );
}

// `load` emits nothing; `css` emits a copy each time it is included.
#[test]
fn load_emits_nothing_and_css_emits_a_copy_each_time() {
    tempfile!("_mv_twice_css.scss", "a {b: c}");
    assert_eq!(
        "",
        compile("@use \"sass:meta\";\n$_: meta.load(\"mv_twice_css\");")
    );
    assert_eq!(
        "a {\n  b: c;\n}\n\na {\n  b: c;\n}\n",
        compile(
            "@use \"sass:meta\";\n$m: meta.load(\"mv_twice_css\");\n@include meta.css($m);\n@include meta.css($m);"
        )
    );
}

// The copy nests under the rule the include is written in, as `load-css`
// does (spec `css/nested`).
#[test]
fn css_nests_under_the_include() {
    tempfile!("_mv_nested.scss", "b {c: d}");
    assert_eq!(
        "a b {\n  c: d;\n}\n",
        compile("@use \"sass:meta\";\na {@include meta.css(meta.load(\"mv_nested\"))}")
    );
}

// A module `@use`d and then passed to `css` prints twice: once for the
// `@use`, once for the copy (spec `css/direct/get_module`, `css/string`).
#[test]
fn css_of_a_used_module_prints_a_second_copy() {
    tempfile!("_mv_used.scss", "a {b: c}");
    assert_eq!(
        "a {\n  b: c;\n}\n\na {\n  b: c;\n}\n",
        compile(
            "@use \"sass:meta\";\n@use \"mv_used\";\n@include meta.css(meta.get-module(\"mv_used\"));"
        )
    );
    assert_eq!(
        "a {\n  b: c;\n}\n\na {\n  b: c;\n}\n",
        compile("@use \"sass:meta\";\n@use \"mv_used\";\n@include meta.css(\"mv_used\");")
    );
}

// A module first executed by `meta.load` and then `@use`d prints its CSS
// once, where the `@use` is, after whatever came before it.
#[test]
fn use_after_load_emits_the_css_once_at_the_use() {
    let input = "@use \"sass:meta\";\n@use \"mv_mid\";\n@use \"mv_pending\";\nz {y: x}";
    tempfile!("_mv_pending.scss", "a {b: c}");
    tempfile!(
        "_mv_mid.scss",
        "@use \"sass:meta\";\n$m: meta.load(\"mv_pending\");\nq {r: s}"
    );
    assert_eq!(
        "q {\n  r: s;\n}\n\na {\n  b: c;\n}\n\nz {\n  y: x;\n}\n",
        compile(input)
    );
}

// The two values are one module (spec `load/shares_state`).
#[test]
fn load_and_use_share_one_module() {
    let input = "@use \"sass:meta\";\n@use \"mv_mid2\";\n@use \"mv_shared_state\";\na {b: mv_mid2.$m == meta.get-module(mv_shared_state)}";
    tempfile!("_mv_shared_state.scss", "a {b: c}");
    tempfile!(
        "_mv_mid2.scss",
        "@use \"sass:meta\";\n$m: meta.load(\"mv_shared_state\");"
    );
    assert_eq!("a {\n  b: c;\n}\n\na {\n  b: true;\n}\n", compile(input));
}

// `css` prints the module's upstream modules too, even when one of them
// has already been printed for a `@use`: it is a copy of the module's whole
// CSS, the way dart-sass composes it.
#[test]
fn css_prints_the_upstream_modules_as_well() {
    let input =
        "@use \"sass:meta\";\n@use \"mv_shared\";\n@include meta.css(meta.load(\"mv_withdep\"));";
    tempfile!("_mv_shared.scss", "s {t: u}");
    tempfile!("_mv_withdep.scss", "@use \"mv_shared\";\nx {y: z}");
    assert_eq!(
        "s {\n  t: u;\n}\n\ns {\n  t: u;\n}\n\nx {\n  y: z;\n}\n",
        compile(input)
    );
}

// `$with` configures the file's `!default` variables (spec `load/with`).
#[test]
fn load_with_configures_the_module() {
    let input = "@use \"sass:meta\";\n$module: meta.load(\"mv_with\", $with: (a: b));\nc {d: meta.inspect(meta.module-variables($module))}";
    tempfile!("_mv_with.scss", "$a: e !default;");
    assert_eq!("c {\n  d: (\"a\": b);\n}\n", compile(input));
}

#[test]
fn load_with_rejects_a_variable_without_default() {
    let input = "@use \"sass:meta\";\n$_: meta.load(\"mv_nodefault\", $with: (a: b));";
    tempfile!("_mv_nodefault.scss", "$a: c;");
    assert_eq!(
        "Error: $a was not declared with !default in the @used module.",
        compile_err(input)
    );
}

#[test]
fn load_twice_with_configuration_is_an_error() {
    let input = "@use \"sass:meta\";\n$_: meta.load(\"mv_twice\", $with: (a: b));\n$_: meta.load(\"mv_twice\", $with: (a: b));";
    tempfile!("_mv_twice.scss", "$a: c !default;");
    assert_eq!(
        "Error: _mv_twice.scss was already loaded, so it can't be configured using \"with\".",
        compile_err(input)
    );
}

// Members of a loaded module are not available to the loader (spec
// `load/error/member`).
#[test]
fn load_does_not_add_a_namespace() {
    let input = "@use \"sass:meta\";\n$_: meta.load(\"mv-member\");\na {b: mv-member.$c}";
    tempfile!("_mv-member.scss", "$c: d;");
    assert_eq!(
        "Error: There is no module with namespace \"mv-member\".",
        compile_err(input)
    );
}

// An `@extend` without a target in a loaded module is reported when its CSS
// is emitted, not when the module is loaded (spec `load/no_error/extend`,
// `load_css/error/extend`).
#[test]
fn a_missing_extend_target_waits_for_the_css() {
    tempfile!("_mv_extmissing.scss", "a {@extend missing}");
    assert_eq!(
        "a {\n  b: c;\n}\n",
        compile("@use \"sass:meta\";\n$_: meta.load(\"mv_extmissing\");\na {b: c}")
    );
    assert_eq!(
        "Error: The target selector was not found.",
        compile_err(
            "@use \"sass:meta\";\n$m: meta.load(\"mv_extmissing\");\n@include meta.css($m);"
        )
    );
    assert_eq!(
        "Error: The target selector was not found.",
        compile_err("@use \"sass:meta\";\n@include meta.load-css(\"mv_extmissing\");")
    );
}

// A module's own `@extend` is part of its CSS.
#[test]
fn css_carries_the_module_own_extensions() {
    tempfile!("_mv_selfext.scss", "a {b: c}\nd {@extend a}");
    assert_eq!(
        "a, d {\n  b: c;\n}\n",
        compile("@use \"sass:meta\";\n@include meta.css(meta.load(\"mv_selfext\"));")
    );
    assert_eq!(
        "a a, a d {\n  b: c;\n}\n",
        compile("@use \"sass:meta\";\na {@include meta.load-css(\"mv_selfext\")}")
    );
}

// An `@extend` where the CSS is emitted applies to the copy, whether it was
// written before or after the include (spec `load_css/extend/in_input`).
#[test]
fn an_extend_at_the_include_site_applies_to_the_copy() {
    tempfile!("_mv_target.scss", "a {b: c}");
    assert_eq!(
        "a, d {\n  b: c;\n}\n",
        compile("@use \"sass:meta\";\nd {@extend a}\n@include meta.load-css(\"mv_target\");")
    );
    assert_eq!(
        "a, d {\n  b: c;\n}\n",
        compile("@use \"sass:meta\";\n@include meta.css(meta.load(\"mv_target\"));\nd {@extend a}")
    );
}

// An `@extend` inside the loaded module reaches nothing outside it (spec
// `load_css/extend/in_other`).
#[test]
fn an_extend_in_the_loaded_module_stays_inside_it() {
    tempfile!("_mv_extender.scss", "d {@extend a !optional}");
    assert_eq!(
        "a {\n  b: c;\n}\n",
        compile("@use \"sass:meta\";\na {b: c}\n@include meta.load-css(\"mv_extender\");")
    );
    assert_eq!(
        "a {\n  b: c;\n}\n",
        compile("@use \"sass:meta\";\n@include meta.load-css(\"mv_extender\");\na {b: c}")
    );
}

// Deliberately differs from dart-sass. A copy of a module that `@use`
// executed shares its selector handles with the original, so the extension
// `_mv_midext.scss` declares reaches both copies; dart-sass prints the
// second as `b {c: d}`, the file's own CSS plus the include site's
// extensions only (spec `load_css/twice/use/different_extend`, open). See
// item 30.
#[test]
fn an_upstream_extend_reaches_the_load_css_copy_of_a_used_module() {
    let input = "@use \"sass:meta\";\n@use \"mv_midext\";\n@include meta.load-css(\"mv_extended\")";
    tempfile!("_mv_extended.scss", "b {c: d}");
    tempfile!("_mv_midext.scss", "@use \"mv_extended\";\na {@extend b}");
    assert_eq!("b, a {\n  c: d;\n}\n\nb, a {\n  c: d;\n}\n", compile(input));
}

// Two modules each `load-css` the same file and extend into it: each copy
// gets its own module's extension (spec `load_css/twice/load_css/
// different_extend`).
#[test]
fn each_load_css_copy_takes_its_own_module_extensions() {
    let input = "@use \"mv_left\";\n@use \"mv_right\";";
    tempfile!("_mv_shared_target.scss", "a {b: c}");
    tempfile!(
        "_mv_left.scss",
        "@use \"sass:meta\";\n@include meta.load-css(\"mv_shared_target\");\nleft {@extend a}"
    );
    tempfile!(
        "_mv_right.scss",
        "@use \"sass:meta\";\n@include meta.load-css(\"mv_shared_target\");\nright {@extend a}"
    );
    assert_eq!(
        "a, left {\n  b: c;\n}\n\na, right {\n  b: c;\n}\n",
        compile(input)
    );
}

// `inspect` prints empty parentheses when the file name is not an
// identifier (spec `get_module/meta/inspect/no_namespace`).
#[test]
fn inspect_prints_no_namespace_for_a_numeric_file_name() {
    let input = "@use \"sass:meta\";\n@use \"1\" as one;\na {b: meta.inspect(meta.get-module(one)); c: meta.inspect(meta.load(\"1\"))}";
    tempfile!("_1.scss", "");
    assert_eq!(
        "a {\n  b: get-module();\n  c: get-module();\n}\n",
        compile(input)
    );
}

// Modules from different files differ; one file reached two ways is one
// module (spec `get_module/equality/user_defined`).
#[test]
fn user_defined_module_equality_is_identity() {
    let input = "@use \"sass:meta\";\n@use \"mv_eq_left\";\n@use \"mv_eq_right\";\n@use \"mv_eq_left\" as again;\na {b: meta.get-module(mv_eq_left) == meta.get-module(mv_eq_right); c: meta.get-module(mv_eq_left) == meta.get-module(again)}";
    tempfile!("_mv_eq_left.scss", "");
    tempfile!("_mv_eq_right.scss", "");
    assert_eq!("a {\n  b: false;\n  c: true;\n}\n", compile(input));
}

// `module-functions`, `module-mixins` and `module-variables` take the value
// (spec `module_*/module_value`).
#[test]
fn module_member_maps_take_a_module_value() {
    let input = "@use \"sass:meta\";\n@use \"mv_members\";\n$m: meta.get-module(\"mv_members\");\na {\n  b: meta.inspect(meta.module-variables($m));\n  c: meta.call(map-get(meta.module-functions($m), \"f\"));\n  d: meta.inspect(map-keys(meta.module-mixins($m)));\n}";
    tempfile!(
        "_mv_members.scss",
        "$v: value;\n@function f() {@return f value}\n@mixin m() {m: value}"
    );
    assert_eq!(
        "a {\n  b: (\"v\": value);\n  c: f value;\n  d: (\"m\",);\n}\n",
        compile(input)
    );
}
