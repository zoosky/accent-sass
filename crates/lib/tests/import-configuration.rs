//! `@import` of files that `@forward` configured modules.
//!
//! Each case mirrors a fixture under
//! `sass-spec/spec/directives/import/configuration/`, whose expected output
//! records dart-sass 1.103.1 behavior; the full fixtures pass through the
//! sass-spec runner against this build.

use std::io::Write;

#[macro_use]
mod macros;

// An import-only file exists to give `@import` a different view of a module,
// so the `@forward "ic_twice"` inside `_ic_twice.import.scss` must resolve to
// `_ic_twice.scss` rather than back to the import-only file itself. The module
// executes once -- keeping the configuration from the first load -- but each
// `@import` emits its CSS again.
#[test]
fn importing_a_forwarding_file_twice_emits_css_twice() {
    let input = r#"$a: configured;
@import "ic_twice";
@import "ic_twice";"#;
    tempfile!("_ic_twice.import.scss", "@forward \"ic_twice\";");
    tempfile!("_ic_twice.scss", "$a: original !default;\nb {c: $a}");
    assert_eq!(
        "b {\n  c: configured;\n}\n\nb {\n  c: configured;\n}\n",
        &accent_sass::from_string(input.to_string(), &accent_sass::Options::default())
            .expect(input)
    );
}

// The module keeps the state of its first load: an assignment between the two
// imports updates the module's variable for later reads, but the CSS a second
// `@import` replays is what the module emitted when it executed.
#[test]
fn module_css_is_frozen_at_first_import() {
    let input = r#"@import "ic_still";
$a: changed;
@import "ic_still";

d {
  e: $a;
}"#;
    tempfile!("_ic_still.import.scss", "@forward \"ic_still\";");
    tempfile!("_ic_still.scss", "$a: original !default;\nb {c: $a}");
    assert_eq!(
        "b {\n  c: original;\n}\n\nb {\n  c: original;\n}\n\nd {\n  e: changed;\n}\n",
        &accent_sass::from_string(input.to_string(), &accent_sass::Options::default())
            .expect(input)
    );
}

// A variable forwarded by one `@import` configures a module forwarded by the
// next one, even though it lives in a module rather than a scope (the
// sass/dart-sass#2641 shape), and forwards from sibling imports are not the
// "two forwarded modules" conflict -- that check applies within one file.
#[test]
fn forwarded_variable_configures_a_later_import() {
    let input = r#"@import "ic_config_wrapper";
@import "ic_midstream";"#;
    tempfile!("_ic_config_wrapper.scss", "@forward \"ic_config\";");
    tempfile!("_ic_config.scss", "$a: configured;");
    tempfile!("_ic_midstream.scss", "@forward \"ic_upstream\";");
    tempfile!("_ic_upstream.scss", "$a: original !default;\nb {c: $a}");
    assert_eq!(
        "b {\n  c: configured;\n}\n",
        &accent_sass::from_string(input.to_string(), &accent_sass::Options::default())
            .expect(input)
    );
}

// A nested `@import` of a forwarding file puts the module's CSS where the
// import is written, under the enclosing rule, not at the root.
#[test]
fn nested_import_keeps_the_enclosing_rule() {
    let input = r#"a {
  $a: configured;
  @import "ic_nested_mid";
}"#;
    tempfile!("_ic_nested_mid.scss", "@forward \"ic_nested_up\";");
    tempfile!("_ic_nested_up.scss", "$a: original !default;\nb {c: $a}");
    assert_eq!(
        "a b {\n  c: configured;\n}\n",
        &accent_sass::from_string(input.to_string(), &accent_sass::Options::default())
            .expect(input)
    );
}
