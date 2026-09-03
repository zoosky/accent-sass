//! First-class mixins from `sass:meta`.
//!
//! Every expectation here was compared against the dart-sass 1.103.1 binary
//! before being committed.

#[macro_use]
mod macros;

test!(
    get_mixin_type_of,
    "@use \"sass:meta\";\n@mixin a() {}\nb {\n  c: meta.type-of(meta.get-mixin(a));\n}\n",
    "b {\n  c: mixin;\n}\n"
);
test!(
    get_mixin_inspect,
    "@use \"sass:meta\";\n@mixin a() {}\nb {\n  c: meta.inspect(meta.get-mixin(a));\n}\n",
    "b {\n  c: get-mixin(\"a\");\n}\n"
);
// A mixin value is the declaration it came from, so redefining the name gives a
// value that is not equal to a reference taken beforehand.
test!(
    get_mixin_equality_after_redefinition,
    "@use \"sass:meta\";\n@mixin a() {}\n$first: meta.get-mixin(a);\n@mixin a() {}\n$second: meta.get-mixin(a);\nb {\n  c: $first == $second;\n}\n",
    "b {\n  c: false;\n}\n"
);
test!(
    get_mixin_equality_same_declaration,
    "@use \"sass:meta\";\n@mixin a() {}\nb {\n  c: meta.get-mixin(a) == meta.get-mixin(a);\n}\n",
    "b {\n  c: true;\n}\n"
);

test!(
    apply_forwards_arguments,
    "@use \"sass:meta\";\n@mixin a($x: 1) {b: $x}\nc {\n  @include meta.apply(meta.get-mixin(a), 5);\n}\n",
    "c {\n  b: 5;\n}\n"
);
test!(
    apply_forwards_content,
    "@use \"sass:meta\";\n@mixin a() {@content}\nb {\n  @include meta.apply(meta.get-mixin(a)) {x: y}\n}\n",
    "b {\n  x: y;\n}\n"
);

test!(
    accepts_content_true,
    "@use \"sass:meta\";\n@mixin a() {@content}\nb {\n  c: meta.accepts-content(meta.get-mixin(a));\n}\n",
    "b {\n  c: true;\n}\n"
);
test!(
    accepts_content_false,
    "@use \"sass:meta\";\n@mixin a() {}\nb {\n  c: meta.accepts-content(meta.get-mixin(a));\n}\n",
    "b {\n  c: false;\n}\n"
);
// Built-in mixins report it too: `meta.apply` takes a content block, and
// `meta.load-css` does not.
test!(
    accepts_content_builtin_apply,
    "@use \"sass:meta\";\na {\n  b: meta.accepts-content(meta.get-mixin(apply, meta));\n}\n",
    "a {\n  b: true;\n}\n"
);
test!(
    accepts_content_builtin_load_css,
    "@use \"sass:meta\";\na {\n  b: meta.accepts-content(meta.get-mixin(load-css, meta));\n}\n",
    "a {\n  b: false;\n}\n"
);
error!(
    accepts_content_wrong_type,
    "@use \"sass:meta\";\na {\n  b: meta.accepts-content(meta.get-function(\"red\"));\n}\n",
    "Error: $mixin: get-function(\"red\") is not a mixin reference."
);

test!(
    module_mixins_empty,
    "@use \"sass:meta\";\na {\n  b: meta.inspect(meta.module-mixins(\"meta\")) != null;\n}\n",
    "a {\n  b: true;\n}\n"
);
test!(
    module_mixins_type,
    "@use \"sass:meta\";\na {\n  b: meta.type-of(meta.module-mixins(\"meta\"));\n}\n",
    "a {\n  b: map;\n}\n"
);

// A first-class mixin has no plain CSS form.
error!(
    mixin_in_css,
    "@use \"sass:meta\";\n@mixin a() {}\nb {\n  c: meta.get-mixin(a);\n}\n",
    "Error: get-mixin(\"a\") isn't a valid CSS value."
);
