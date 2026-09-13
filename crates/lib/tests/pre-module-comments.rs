//! Loud comments written above `@use` and `@forward` rules.
//!
//! A comment above such a rule is written once, where it stands. dart-sass
//! 1.104.0 repeated it before every module that also loaded the same module
//! (dart-sass bug #2851); 1.104.1 fixed that, and these tests pin the fixed
//! behaviour. See `specs/docs/features/26-pre-module-comment-repeats.md`.
//!
//! Every expectation was verified against the dart-sass 1.104.1 binary.

use macros::TestFs;

#[macro_use]
mod macros;

/// Bulma's shape: other modules that use `shared` do not repeat the comment
/// written above `@forward "shared"`.
#[test]
fn not_repeated_before_other_modules_that_use_the_same_one() {
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
        "/* C */\ns {\n  x: 0;\n}\n\na {\n  x: 1;\n}\n\nb {\n  x: 2;\n}\n",
        &accent_sass::from_string(input.to_string(), &accent_sass::Options::default().fs(&fs))
            .unwrap()
    );
}

/// A module without CSS takes no comment, so the comment stays with the first
/// module that has some and is written once.
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

/// A file that both uses and forwards one module writes the comment once.
#[test]
fn use_and_forward_of_one_module_write_the_comment_once() {
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
        "/* T */\ns {\n  x: 0;\n}\n\nt {\n  z: 1;\n}\n\na {\n  x: 1;\n}\n",
        &accent_sass::from_string(input.to_string(), &accent_sass::Options::default().fs(&fs))
            .unwrap()
    );
}

/// Two `@use` rules for one module write the comment once.
#[test]
fn two_namespaces_for_one_module_write_the_comment_once() {
    let mut fs = TestFs::new();

    fs.add_file("_shared.scss", "s {x: 0}");

    let input = "/* U */\n@use \"shared\";\n@use \"shared\" as s2;";

    assert_eq!(
        "/* U */\ns {\n  x: 0;\n}\n",
        &accent_sass::from_string(input.to_string(), &accent_sass::Options::default().fs(&fs))
            .unwrap()
    );
}
