use macros::TestLogger;

#[macro_use]
mod macros;

#[test]
fn simple_debug() {
    let input = "@debug 2";
    let logger = TestLogger::default();
    let options = accent_sass::Options::default().logger(&logger);
    let output = accent_sass::from_string(input.to_string(), &options).expect(input);
    assert_eq!(&output, "");
    assert_eq!(&[String::from("2")], logger.debug_messages().as_slice());
    assert_eq!(&[] as &[String], logger.warning_messages().as_slice());
}

#[test]
fn simple_debug_with_semicolon() {
    let input = "@debug 2;";
    let logger = TestLogger::default();
    let options = accent_sass::Options::default().logger(&logger);
    let output = accent_sass::from_string(input.to_string(), &options).expect(input);
    assert_eq!(&output, "");
    assert_eq!(&[String::from("2")], logger.debug_messages().as_slice());
    assert_eq!(&[] as &[String], logger.warning_messages().as_slice());
}

/// A string message is reported as its text, the same rule `@warn` follows.
/// Verified against dart-sass 1.104.0.
#[test]
fn debug_quoted_string_loses_its_quotes() {
    let input = "@debug \"careful\";";
    let logger = TestLogger::default();
    let options = accent_sass::Options::default().logger(&logger);
    let output = accent_sass::from_string(input.to_string(), &options).expect(input);
    assert_eq!(&output, "");
    assert_eq!(
        &[String::from("careful")],
        logger.debug_messages().as_slice()
    );
}

/// Anything else is inspected, which keeps the quotes of a nested string.
#[test]
fn debug_string_inside_a_list_keeps_its_quotes() {
    let input = "@debug (\"a\", \"b\");";
    let logger = TestLogger::default();
    let options = accent_sass::Options::default().logger(&logger);
    let output = accent_sass::from_string(input.to_string(), &options).expect(input);
    assert_eq!(&output, "");
    assert_eq!(
        &[String::from("\"a\", \"b\"")],
        logger.debug_messages().as_slice()
    );
}

#[test]
fn debug_while_quiet() {
    let input = "@debug 2;";
    let logger = TestLogger::default();
    let options = accent_sass::Options::default().logger(&logger).quiet(true);
    let output = accent_sass::from_string(input.to_string(), &options).expect(input);
    assert_eq!(&output, "");
    assert_eq!(&[] as &[String], logger.debug_messages().as_slice());
    assert_eq!(&[] as &[String], logger.warning_messages().as_slice());
}
