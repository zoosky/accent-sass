//! `@use` and `@forward` member visibility and conflicts.
//!
//! Every expectation here was compared against the dart-sass 1.103.1 binary
//! before being committed.

use std::io::Write;

#[macro_use]
mod macros;

// `show` and `hide` were parsed and then dropped, so a hidden member stayed
// reachable through the forwarding module.
#[test]
fn forward_show_hides_the_rest() {
    let input = r#"@use "ms_show_midstream";
a {@include ms_show_midstream.d}"#;
    tempfile!(
        "ms_show_midstream.scss",
        "@forward \"ms_show_upstream\" show c;"
    );
    tempfile!(
        "ms_show_upstream.scss",
        "@mixin c() {b: c}\n@mixin d() {a: c}"
    );
    assert_err!("Error: Undefined mixin.", input);
}

#[test]
fn forward_show_keeps_the_named_member() {
    let input = r#"@use "ms_show2_midstream";
a {@include ms_show2_midstream.c}"#;
    tempfile!(
        "ms_show2_midstream.scss",
        "@forward \"ms_show2_upstream\" show c;"
    );
    tempfile!(
        "ms_show2_upstream.scss",
        "@mixin c() {b: c}\n@mixin d() {a: c}"
    );
    assert_eq!(
        "a {\n  b: c;\n}\n",
        &grass::from_string(input.to_string(), &grass::Options::default()).expect(input)
    );
}

// A member two `@use ... as *` modules both provide is ambiguous, and naming it
// is an error rather than a silent choice between them.
#[test]
fn global_module_member_conflict() {
    let input = r#"@use "ms_conflict1" as *;
@use "ms_conflict2" as *;
a {b: $ms-member}"#;
    tempfile!("ms_conflict1.scss", "$ms-member: from other1;");
    tempfile!("ms_conflict2.scss", "$ms-member: from other2;");
    assert_err!(
        "Error: This variable is available from multiple global modules.",
        input
    );
}

// Loading the same file twice is one member seen twice, not a conflict.
#[test]
fn global_module_same_file_twice_is_not_a_conflict() {
    let input = r#"@use "ms_same" as *;
@use "ms_same" as *;
a {b: $ms-same}"#;
    tempfile!("ms_same.scss", "$ms-same: d;");
    assert_eq!(
        "a {\n  b: d;\n}\n",
        &grass::from_string(input.to_string(), &grass::Options::default()).expect(input)
    );
}

// Forwarded conflicts are reported as soon as the second `@forward` runs, even
// if nothing ever names the member.
#[test]
fn forwarded_member_conflict() {
    let input = r#"@use "ms_fwd_mid";"#;
    tempfile!(
        "ms_fwd_mid.scss",
        "@forward \"ms_fwd1\";\n@forward \"ms_fwd2\";"
    );
    tempfile!("ms_fwd1.scss", "$ms-fwd: from other1;");
    tempfile!("ms_fwd2.scss", "$ms-fwd: from other2;");
    assert_err!(
        "Error: Two forwarded modules both define a variable named $ms-fwd.",
        input
    );
}

// Two modules that re-export the same declaration are not in conflict.
#[test]
fn forwarding_the_same_module_twice_is_not_a_conflict() {
    let input = r#"@use "ms_twice_mid";
a {b: ms_twice_mid.$ms-twice}"#;
    tempfile!(
        "ms_twice_mid.scss",
        "@forward \"ms_twice_up\";\n@forward \"ms_twice_up\";"
    );
    tempfile!("ms_twice_up.scss", "$ms-twice: d;");
    assert_eq!(
        "a {\n  b: d;\n}\n",
        &grass::from_string(input.to_string(), &grass::Options::default()).expect(input)
    );
}

// A partial beside its non-partial spelling is ambiguous rather than resolved
// by whichever the loader happened to check first.
#[test]
fn ambiguous_partial_and_non_partial() {
    let input = r#"@import "ms_ambiguous";"#;
    tempfile!("_ms_ambiguous.scss", "a {partial: true}");
    tempfile!("ms_ambiguous.scss", "a {partial: false}");
    assert_err!("Error: It's not clear which file to import. Found:", input);
}
