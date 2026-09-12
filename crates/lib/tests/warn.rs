use macros::TestLogger;

#[macro_use]
mod macros;

#[test]
fn warn_debug() {
    let input = "@warn 2";
    let logger = TestLogger::default();
    let options = accent_sass::Options::default().logger(&logger);
    let output = accent_sass::from_string(input.to_string(), &options).expect(input);
    assert_eq!(&output, "");
    assert_eq!(&[] as &[String], logger.debug_messages().as_slice());
    assert_eq!(&[String::from("2")], logger.warning_messages().as_slice());
}

#[test]
fn simple_warn_with_semicolon() {
    let input = "@warn 2;";
    let logger = TestLogger::default();
    let options = accent_sass::Options::default().logger(&logger);
    let output = accent_sass::from_string(input.to_string(), &options).expect(input);
    assert_eq!(&output, "");
    assert_eq!(&[] as &[String], logger.debug_messages().as_slice());
    assert_eq!(&[String::from("2")], logger.warning_messages().as_slice());
}

/// A string message is reported as its text. Verified against dart-sass
/// 1.104.0, which unwraps a top-level string in `visitWarnRule`.
#[test]
fn warn_quoted_string_loses_its_quotes() {
    let input = "@warn \"careful\";";
    let logger = TestLogger::default();
    let options = accent_sass::Options::default().logger(&logger);
    let output = accent_sass::from_string(input.to_string(), &options).expect(input);
    assert_eq!(&output, "");
    assert_eq!(
        &[String::from("careful")],
        logger.warning_messages().as_slice()
    );
}

/// Only the outermost value is unwrapped, so a string inside a list keeps its
/// quotes. dart-sass serializes the list itself.
#[test]
fn warn_string_inside_a_list_keeps_its_quotes() {
    let input = "@warn (\"a\", \"b\");";
    let logger = TestLogger::default();
    let options = accent_sass::Options::default().logger(&logger);
    let output = accent_sass::from_string(input.to_string(), &options).expect(input);
    assert_eq!(&output, "");
    assert_eq!(
        &[String::from("\"a\", \"b\"")],
        logger.warning_messages().as_slice()
    );
}

/// A value that is not a string still takes the CSS serialization path, where
/// null is empty, as dart-sass gives.
#[test]
fn warn_null_reports_an_empty_message() {
    let input = "@warn null;";
    let logger = TestLogger::default();
    let options = accent_sass::Options::default().logger(&logger);
    let output = accent_sass::from_string(input.to_string(), &options).expect(input);
    assert_eq!(&output, "");
    assert_eq!(&[String::new()], logger.warning_messages().as_slice());
}

#[test]
fn warn_while_quiet() {
    let input = "@warn 2;";
    let logger = TestLogger::default();
    let options = accent_sass::Options::default().logger(&logger).quiet(true);
    let output = accent_sass::from_string(input.to_string(), &options).expect(input);
    assert_eq!(&output, "");
    assert_eq!(&[] as &[String], logger.debug_messages().as_slice());
    assert_eq!(&[] as &[String], logger.warning_messages().as_slice());
}
