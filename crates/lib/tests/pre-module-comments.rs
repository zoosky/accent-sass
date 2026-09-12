//! Loud comments written above `@use` and `@forward` rules.
//!
//! dart-sass records such a comment against the module the rule loads, and
//! writes it again before every module that loads that module. One comment
//! above Bulma's `@forward "shared"` is printed six times, because each of
//! the five form modules after it also uses `shared`.
//!
//! Every expectation was verified against the dart-sass 1.104.0 binary. The
//! repeats are dart-sass bug #2851, fixed in the unreleased 1.104.1; see
//! `specs/docs/features/26-pre-module-comment-repeats.md`, which says to
//! revert this behaviour when the reference moves.

use macros::TestFs;

#[macro_use]
mod macros;

/// Bulma's shape: the comment is registered for `shared` and written again
/// before each module that uses `shared`.
#[test]
fn repeats_before_each_module_that_uses_the_registered_one() {
    let mut fs = TestFs::new();

    fs.add_file(
        "_idx.scss",
        r#"/* C */
        @forward "shared";
        @forward "a";
        @forward "b";
    "#,
    );
    fs.add_file("_shared.scss", "s {x: 0}");
    fs.add_file("_a.scss", "@use \"shared\";\na {x: 1}");
    fs.add_file("_b.scss", "@use \"shared\";\nb {x: 2}");

    let input = r#"@use "idx";"#;

    assert_eq!(
        "/* C */\ns {\n  x: 0;\n}\n\n/* C */\na {\n  x: 1;\n}\n\n/* C */\nb {\n  x: 2;\n}\n",
        &accent_sass::from_string(input.to_string(), &accent_sass::Options::default().fs(&fs))
            .unwrap()
    );
}

/// Nothing is registered for a module without CSS, so the comment stays with
/// the first module that has some and is written once.
#[test]
fn no_repeat_when_the_registered_module_has_no_css() {
    let mut fs = TestFs::new();

    fs.add_file(
        "_idx.scss",
        r#"/* C */
        @forward "shared";
        @forward "a";
        @forward "b";
    "#,
    );
    fs.add_file("_shared.scss", "$v: 1;");
    fs.add_file("_a.scss", "@use \"shared\";\na {x: 1}");
    fs.add_file("_b.scss", "@use \"shared\";\nb {x: 2}");

    let input = r#"@use "idx";"#;

    assert_eq!(
        "/* C */\na {\n  x: 1;\n}\n\nb {\n  x: 2;\n}\n",
        &accent_sass::from_string(input.to_string(), &accent_sass::Options::default().fs(&fs))
            .unwrap()
    );
}

/// A comment belongs to the load that follows it, not to the first load in
/// the file.
#[test]
fn comment_registers_for_the_load_that_follows_it() {
    let mut fs = TestFs::new();

    fs.add_file("_a.scss", "a {x: 1}");
    fs.add_file("_b.scss", "b {y: 2}");

    let input = "@use \"a\";\n/* C */\n@use \"b\";";

    assert_eq!(
        "a {\n  x: 1;\n}\n\n/* C */\nb {\n  y: 2;\n}\n",
        &accent_sass::from_string(input.to_string(), &accent_sass::Options::default().fs(&fs))
            .unwrap()
    );
}

/// A module with no CSS takes no comment, which stays where it was written.
#[test]
fn module_without_css_leaves_the_comment_in_place() {
    let mut fs = TestFs::new();

    fs.add_file("_novars.scss", "$v: 1;");

    let input = "/* C */\n@use \"novars\";\nc {z: 3}";

    assert_eq!(
        "/* C */\nc {\n  z: 3;\n}\n",
        &accent_sass::from_string(input.to_string(), &accent_sass::Options::default().fs(&fs))
            .unwrap()
    );
}

/// dart-sass lists an upstream module once per rule that loads it, so a file
/// that both uses and forwards one module repeats the comment.
#[test]
fn use_and_forward_of_one_module_repeat_the_comment() {
    let mut fs = TestFs::new();

    fs.add_file(
        "_twice.scss",
        r#"/* T */
        @use "shared";
        @forward "shared";
        t {z: 1}
    "#,
    );
    fs.add_file("_shared.scss", "s {x: 0}");
    fs.add_file("_a.scss", "@use \"shared\";\na {x: 1}");

    let input = "@use \"twice\";\n@use \"a\";";

    assert_eq!(
        "/* T */\ns {\n  x: 0;\n}\n\n/* T */\nt {\n  z: 1;\n}\n\na {\n  x: 1;\n}\n",
        &accent_sass::from_string(input.to_string(), &accent_sass::Options::default().fs(&fs))
            .unwrap()
    );
}

/// Two `@use` rules for one module repeat it too, and the repeat comes after
/// that module's CSS.
#[test]
fn two_namespaces_for_one_module_repeat_the_comment() {
    let mut fs = TestFs::new();

    fs.add_file("_shared.scss", "s {x: 0}");

    let input = "/* U */\n@use \"shared\";\n@use \"shared\" as s2;";

    assert_eq!(
        "/* U */\ns {\n  x: 0;\n}\n\n/* U */\n",
        &accent_sass::from_string(input.to_string(), &accent_sass::Options::default().fs(&fs))
            .unwrap()
    );
}
