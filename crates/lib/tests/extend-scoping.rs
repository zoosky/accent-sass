//! `@extend` scoped to a module's upstream closure.
//!
//! Each case mirrors a fixture under `sass-spec/spec/directives/use/extend/`.
//! Every expectation below was checked against dart-sass 1.103.1 directly.
//!
//! The spec fixtures for this area do not all pass yet: three under
//! `use/extend/scope/` still fail, all of them cases where `@import` and
//! `@use` mix, because an imported file shares the importer's extension
//! store and so widens the closure. See the pull request for the tallies.

use std::io::Write;

#[macro_use]
mod macros;

// An extension reaches the CSS of modules the extending file loaded.
#[test]
fn extend_reaches_upstream() {
    let input = r#"@use "es_up";
downstream {@extend es-upstream}"#;
    tempfile!("es_up.scss", "es-upstream {a: b}");
    assert_eq!(
        "es-upstream, downstream {\n  a: b;\n}\n",
        &accent_sass::from_string(input.to_string(), &accent_sass::Options::default())
            .expect(input)
    );
}

// An extension in one module never reaches a sibling module the extending
// module did not load.
#[test]
fn extend_does_not_reach_siblings() {
    let input = r#"@use "es_sib_a";
@use "es_sib_b";"#;
    tempfile!("es_sib_a.scss", "in-a {a: b}");
    tempfile!(
        "es_sib_b.scss",
        "in-b {c: d}\nin-b-ext {@extend in-a !optional}"
    );
    assert_eq!(
        "in-a {\n  a: b;\n}\n\nin-b {\n  c: d;\n}\n",
        &accent_sass::from_string(input.to_string(), &accent_sass::Options::default())
            .expect(input)
    );
}

// An upstream module's extension never reaches downstream CSS.
#[test]
fn extend_does_not_reach_downstream() {
    let input = r#"@use "es_down";
in-root {a: b}"#;
    tempfile!(
        "es_down.scss",
        "up-ext {@extend in-root !optional}\nup {c: d}"
    );
    assert_eq!(
        "up {\n  c: d;\n}\n\nin-root {\n  a: b;\n}\n",
        &accent_sass::from_string(input.to_string(), &accent_sass::Options::default())
            .expect(input)
    );
}

// A mandatory extension whose target exists only in a sibling module errors:
// the target is scoped away, so it was not found.
#[test]
fn mandatory_extend_scoped_away_is_an_error() {
    let input = r#"@use "es_err_a";
@use "es_err_b";"#;
    tempfile!("es_err_a.scss", "only-in-a {a: b}");
    tempfile!("es_err_b.scss", "b-ext {@extend only-in-a}");
    assert_err!("Error: The target selector was not found.", input);
}

// A mandatory extension satisfied upstream through a forward chain is found.
#[test]
fn mandatory_extend_through_forward_chain() {
    let input = r#"@use "es_chain_mid";
chain-ext {@extend chain-up}"#;
    tempfile!("es_chain_mid.scss", "@forward \"es_chain_up\";");
    tempfile!("es_chain_up.scss", "chain-up {a: b}");
    assert_eq!(
        "chain-up, chain-ext {\n  a: b;\n}\n",
        &accent_sass::from_string(input.to_string(), &accent_sass::Options::default())
            .expect(input)
    );
}
