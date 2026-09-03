//! `meta.load-css` and `@use` sharing one module instance through the module
//! cache.
//!
//! Each case mirrors a fixture under
//! `sass-spec/spec/core_functions/meta/load_css/` or
//! `sass-spec/spec/directives/use/`, whose expected output records dart-sass
//! 1.103.1 behavior; the full fixtures pass through the sass-spec runner
//! against this build.

use std::io::Write;

#[macro_use]
mod macros;

// Loading a file through `@use` and again through `load-css` is one module:
// the second load replays the CSS the module emitted, it does not re-execute.
#[test]
fn load_css_shares_the_use_instance() {
    let input = r#"@use "sass:meta";
@use "lcmc_shared";
@include meta.load-css("lcmc_shared");"#;
    tempfile!("lcmc_shared.scss", "$v: first !default;\na {b: $v}");
    assert_eq!(
        "a {\n  b: first;\n}\n\na {\n  b: first;\n}\n",
        &grass::from_string(input.to_string(), &grass::Options::default()).expect(input)
    );
}

// The same file cannot be configured twice, even with an identical
// configuration. The `load-css` message names the resolved file.
#[test]
fn load_css_twice_with_configuration_is_an_error() {
    let input = r#"@use "sass:meta";
@include meta.load-css("lcmc_twice", $with: (a: b));
@include meta.load-css("lcmc_twice", $with: (a: b));"#;
    tempfile!("_lcmc_twice.scss", "$a: c !default;");
    assert_err!(
        "Error: _lcmc_twice.scss was already loaded, so it can't be configured using \"with\".",
        input
    );
}

// The `@use` flavor of the same error keeps the generic message.
#[test]
fn use_twice_with_configuration_is_an_error() {
    let input = r#"@use "lcmc_use2" as u1 with ($a: b);
@use "lcmc_use2" as u2 with ($a: b);"#;
    tempfile!("lcmc_use2.scss", "$a: c !default;");
    assert_err!(
        "Error: This module was already loaded, so it can't be configured using \"with\".",
        input
    );
}

// One `with` clause reaching a module along two `@forward` paths is that one
// clause seen twice, not a double configuration.
#[test]
fn one_clause_through_two_forwards_is_legal() {
    let input = r#"@use "lcmc_mid" with ($lcmc-dist: configured);
a {b: lcmc_mid.$lcmc-dist}"#;
    tempfile!(
        "lcmc_mid.scss",
        "@forward \"lcmc_left\";\n@forward \"lcmc_right\";"
    );
    tempfile!("lcmc_left.scss", "@forward \"lcmc_shared_up\";");
    tempfile!("lcmc_right.scss", "@forward \"lcmc_shared_up\";");
    tempfile!("lcmc_shared_up.scss", "$lcmc-dist: original !default;");
    assert_eq!(
        "a {\n  b: configured;\n}\n",
        &grass::from_string(input.to_string(), &grass::Options::default()).expect(input)
    );
}

// A configuration that names only the forwarding file's own variables could
// never have configured the file it forwards, so loading that file both ways
// stays legal.
#[test]
fn clause_that_could_not_apply_is_legal() {
    let input = r#"@use "sass:meta";
@include meta.load-css("lcmc_cfgless");
@include meta.load-css("lcmc_owner", $with: (a: overridden));"#;
    tempfile!(
        "_lcmc_owner.scss",
        "@forward \"lcmc_cfgless\";\n\n$a: default !default;\nb {mid: $a}"
    );
    tempfile!("_lcmc_cfgless.scss", "// defines no variables\n");
    assert_eq!(
        "b {\n  mid: overridden;\n}\n",
        &grass::from_string(input.to_string(), &grass::Options::default()).expect(input)
    );
}

// A second load of a cached module re-nests its recorded CSS under the rule
// enclosing the new load site.
#[test]
fn replayed_css_renests_per_load_site() {
    let input = r#"@use "sass:meta";
a {@include meta.load-css("lcmc_nest")}
b {@include meta.load-css("lcmc_nest")}"#;
    tempfile!("_lcmc_nest.scss", "c {d: e}");
    assert_eq!(
        "a c {\n  d: e;\n}\n\nb c {\n  d: e;\n}\n",
        &grass::from_string(input.to_string(), &grass::Options::default()).expect(input)
    );
}
