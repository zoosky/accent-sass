//! Deprecation warnings, starting with `bogus-combinators`.
//!
//! Each frame test compares a warning's banner, message and source frame,
//! everything up to the closing `╵`. The location line below the frame is not
//! compared: dart-sass writes a stack trace there, and this compiler writes one
//! location.
//!
//! Every expected frame was taken from the dart-sass 1.104.1 binary's output.

use std::cell::RefCell;

use accent_sass::{DeprecationWarning, Logger};
use accent_sass_compiler::codemap::SpanLoc;

/// Records deprecation warnings as dart-sass would print them, and the count
/// left out as repetitive.
#[derive(Debug, Default)]
struct Recorder {
    warnings: RefCell<Vec<String>>,
    omitted: RefCell<Option<usize>>,
}

impl Logger for Recorder {
    fn debug(&self, _location: SpanLoc, _message: &str) {}

    fn warn(&self, _location: SpanLoc, _message: &str) {}

    fn deprecation(&self, warning: &DeprecationWarning) {
        assert_eq!("bogus-combinators", warning.deprecation().id());
        self.warnings
            .borrow_mut()
            .push(warning.formatted().to_owned());
    }

    fn repetitive_deprecations_omitted(&self, count: usize) {
        *self.omitted.borrow_mut() = Some(count);
    }
}

/// Compiles `input` with `options` and returns each warning up to and
/// including its frame's closing line.
fn frames(input: &str, options: accent_sass::Options<'_>) -> Vec<String> {
    let recorder = Recorder::default();
    accent_sass::from_string(input.to_owned(), &options.logger(&recorder))
        .expect("input should compile");

    recorder
        .warnings
        .take()
        .iter()
        .map(|warning| {
            let mut lines = Vec::new();
            for line in warning.lines() {
                lines.push(line);
                if line.trim_start().starts_with(['╵', '\'']) {
                    break;
                }
            }
            lines.join("\n")
        })
        .collect()
}

fn scss() -> accent_sass::Options<'static> {
    accent_sass::Options::default()
}

fn sass() -> accent_sass::Options<'static> {
    accent_sass::Options::default().input_syntax(accent_sass::InputSyntax::Sass)
}

/// A trailing combinator with a declaration points at both.
#[test]
fn trailing_combinator_with_a_declaration() {
    assert_eq!(
        vec![
            "DEPRECATION WARNING [bogus-combinators]: The selector \"a >\" is only valid for nesting and shouldn't\nhave children other than style rules. It will be omitted from the generated CSS.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators\n\n  ╷\n1 │ a > {b: c}\n  │ ^^^ invalid selector\n  │      ━━━━ this is not a style rule\n  ╵",
        ] as Vec<&str>,
        frames("a > {b: c}", scss()),
    );
}

/// Only the bogus complex selector in a list is marked.
#[test]
fn one_complex_selector_in_a_list() {
    assert_eq!(
        vec![
            "DEPRECATION WARNING [bogus-combinators]: The selector \"b >\" is only valid for nesting and shouldn't\nhave children other than style rules. It will be omitted from the generated CSS.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators\n\n  ╷\n1 │ a, b > {c: d}\n  │    ^^^ invalid selector\n  │         ━━━━ this is not a style rule\n  ╵",
        ] as Vec<&str>,
        frames("a, b > {c: d}", scss()),
    );
}

/// Each bogus complex selector gets its own warning.
#[test]
fn two_bogus_selectors_in_a_list() {
    assert_eq!(
        vec![
            "DEPRECATION WARNING [bogus-combinators]: The selector \"a >\" is only valid for nesting and shouldn't\nhave children other than style rules. It will be omitted from the generated CSS.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators\n\n  ╷\n1 │ a >, b + > c {d: e}\n  │ ^^^ invalid selector\n  │               ━━━━ this is not a style rule\n  ╵",
            "DEPRECATION WARNING [bogus-combinators]: The selector \"b + > c\" is invalid CSS. It will be omitted from the generated CSS.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators\n\n  ╷\n1 │ a >, b + > c {d: e}\n  │      ^^^^^^^\n  ╵",
        ] as Vec<&str>,
        frames("a >, b + > c {d: e}", scss()),
    );
}

/// A leading combinator is legal nesting, so the rule is kept and only marked.
#[test]
fn leading_combinator() {
    assert_eq!(
        vec![
            "DEPRECATION WARNING [bogus-combinators]: The selector \"> a\" is invalid CSS.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators\n\n  ╷\n1 │ > a {b: c}\n  │ ^^^\n  ╵",
        ] as Vec<&str>,
        frames("> a {b: c}", scss()),
    );
}

/// Two combinators in a row can never match, so the rule is omitted.
#[test]
fn useless_selector() {
    assert_eq!(
        vec![
            "DEPRECATION WARNING [bogus-combinators]: The selector \"a + > b\" is invalid CSS. It will be omitted from the generated CSS.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators\n\n  ╷\n1 │ a + > b {c: d}\n  │ ^^^^^^^\n  ╵",
        ] as Vec<&str>,
        frames("a + > b {c: d}", scss()),
    );
}

/// A rule holding only a comment suggests a silent comment.
#[test]
fn comment_only_child() {
    assert_eq!(
        vec![
            "DEPRECATION WARNING [bogus-combinators]: The selector \"a >\" is only valid for nesting and shouldn't\nhave children other than style rules. It will be omitted from the generated CSS.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators\n\n  ╷\n1 │ a > {/* x */}\n  │ ^^^ invalid selector\n  │      ━━━━━━━ this is not a style rule\n  │              (try converting to a //-style comment)\n  ╵",
        ] as Vec<&str>,
        frames("a > {/* x */}", scss()),
    );
}

/// Adjacent lines are both drawn.
#[test]
fn declaration_on_the_next_line() {
    assert_eq!(
        vec![
            "DEPRECATION WARNING [bogus-combinators]: The selector \"a >\" is only valid for nesting and shouldn't\nhave children other than style rules. It will be omitted from the generated CSS.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators\n\n  ╷\n1 │ a > {\n  │ ^^^ invalid selector\n2 │   b: c;\n  │   ━━━━ this is not a style rule\n  ╵",
        ] as Vec<&str>,
        frames("a > {\n  b: c;\n}\n", scss()),
    );
}

/// A line between the two spans is written as `...`.
#[test]
fn skipped_line_in_the_indented_syntax() {
    assert_eq!(
        vec![
            "DEPRECATION WARNING [bogus-combinators]: The selector \"a >\" is only valid for nesting and shouldn't\nhave children other than style rules. It will be omitted from the generated CSS.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators\n\n    ╷\n1   │ a >\n    │ ^^^ invalid selector\n... │\n3   │   b: c\n    │   ━━━━ this is not a style rule\n    ╵",
        ] as Vec<&str>,
        frames("a >\n  // x\n  b: c\n", sass()),
    );
}

/// A declaration over two lines is drawn with corners.
#[test]
fn declaration_over_two_lines() {
    assert_eq!(
        vec![
            "DEPRECATION WARNING [bogus-combinators]: The selector \"a >\" is only valid for nesting and shouldn't\nhave children other than style rules. It will be omitted from the generated CSS.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators\n\n  ╷\n1 │   a > {b: c\n  │   ^^^ invalid selector\n  │ ┌──────^\n2 │ │     d}\n  │ └─────^ this is not a style rule\n  ╵",
        ] as Vec<&str>,
        frames("a > {b: c\n    d}", scss()),
    );
}

/// An at-rule without a body is not a style rule either.
#[test]
fn bodiless_at_rule_child() {
    assert_eq!(
        vec![
            "DEPRECATION WARNING [bogus-combinators]: The selector \"a >\" is only valid for nesting and shouldn't\nhave children other than style rules. It will be omitted from the generated CSS.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators\n\n  ╷\n1 │ a > {@foo bar;}\n  │ ^^^ invalid selector\n  │      ━━━━━━━━ this is not a style rule\n  ╵",
        ] as Vec<&str>,
        frames("a > {@foo bar;}", scss()),
    );
}

/// A nested property points at its own name and value.
#[test]
fn nested_property_child() {
    assert_eq!(
        vec![
            "DEPRECATION WARNING [bogus-combinators]: The selector \"a >\" is only valid for nesting and shouldn't\nhave children other than style rules. It will be omitted from the generated CSS.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators\n\n  ╷\n1 │ a > {font: {family: x}}\n  │ ^^^ invalid selector\n  │             ━━━━━━━━━ this is not a style rule\n  ╵",
        ] as Vec<&str>,
        frames("a > {font: {family: x}}", scss()),
    );
}

/// The declaration span includes `!important`.
#[test]
fn important_declaration() {
    assert_eq!(
        vec![
            "DEPRECATION WARNING [bogus-combinators]: The selector \"a >\" is only valid for nesting and shouldn't\nhave children other than style rules. It will be omitted from the generated CSS.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators\n\n  ╷\n1 │ a > {b: c !important}\n  │ ^^^ invalid selector\n  │      ━━━━━━━━━━━━━━━ this is not a style rule\n  ╵",
        ] as Vec<&str>,
        frames("a > {b: c !important}", scss()),
    );
}

/// A nested selector names the resolved selector and points where it was written.
#[test]
fn nested_selector() {
    assert_eq!(
        vec![
            "DEPRECATION WARNING [bogus-combinators]: The selector \"x a >\" is only valid for nesting and shouldn't\nhave children other than style rules. It will be omitted from the generated CSS.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators\n\n  ╷\n1 │ x {a > {b: c}}\n  │    ^^^ invalid selector\n  │         ━━━━ this is not a style rule\n  ╵",
        ] as Vec<&str>,
        frames("x {a > {b: c}}", scss()),
    );
}

/// A rule inside `@media` warns the same way.
#[test]
fn inside_media() {
    assert_eq!(
        vec![
            "DEPRECATION WARNING [bogus-combinators]: The selector \"a >\" is only valid for nesting and shouldn't\nhave children other than style rules. It will be omitted from the generated CSS.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators\n\n  ╷\n1 │ @media screen {a > {b: c}}\n  │                ^^^ invalid selector\n  │                     ━━━━ this is not a style rule\n  ╵",
        ] as Vec<&str>,
        frames("@media screen {a > {b: c}}", scss()),
    );
}

/// An interpolated selector points at the whole interpolation.
#[test]
fn interpolated_selector() {
    assert_eq!(
        vec![
            "DEPRECATION WARNING [bogus-combinators]: The selector \"a >\" is only valid for nesting and shouldn't\nhave children other than style rules. It will be omitted from the generated CSS.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators\n\n  ╷\n1 │ #{\"a >\"} {b: c}\n  │ ^^^^^^^^ invalid selector\n  │           ━━━━ this is not a style rule\n  ╵",
        ] as Vec<&str>,
        frames("#{\"a >\"} {b: c}", scss()),
    );
}

/// A useless selector cannot be an extender.
#[test]
fn bogus_extender() {
    assert_eq!(
        vec![
            "DEPRECATION WARNING [bogus-combinators]: The selector \".a > + x\" is invalid CSS and can't be an extender.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators\n\n  ╷\n1 │ .a > + x {@extend .b}\n  │ ^^^^^^^^ invalid selector\n  │           ━━━━━━━━━━ @extend rule\n  ╵",
        ] as Vec<&str>,
        frames(".a > + x {@extend .b}\n.b {c: d}", scss()),
    );
}

/// A trailing combinator should not be an extender, and the extended rule warns too.
#[test]
fn trailing_combinator_extender() {
    assert_eq!(
        vec![
            "DEPRECATION WARNING [bogus-combinators]: The selector \".x >\" is invalid CSS and shouldn't be an extender.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators\n\n  ╷\n1 │ .x > {@extend .y}\n  │ ^^^^ invalid selector\n  │       ━━━━━━━━━━ @extend rule\n  ╵",
            "DEPRECATION WARNING [bogus-combinators]: The selector \".x >\" is only valid for nesting and shouldn't\nhave children other than style rules. It will be omitted from the generated CSS.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators\n\n  ╷\n1 │ .x > {@extend .y}\n  │ ^^^^ invalid selector\n2 │ .y {c: d}\n  │     ━━━━ this is not a style rule\n  ╵",
        ] as Vec<&str>,
        frames(".x > {@extend .y}\n.y {c: d}", scss()),
    );
}

/// Without Unicode, the frame is drawn in ASCII.
#[test]
fn ascii_frame() {
    assert_eq!(
        vec![
            "DEPRECATION WARNING [bogus-combinators]: The selector \"a >\" is only valid for nesting and shouldn't\nhave children other than style rules. It will be omitted from the generated CSS.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators\n\n  ,\n1 | a > {/* x */}\n  | ^^^ invalid selector\n  |      ======= this is not a style rule\n  |              (try converting to a //-style comment)\n  '",
        ] as Vec<&str>,
        frames("a > {/* x */}", scss().unicode_error_messages(false)),
    );
}

/// A rule with nothing in it is not reported.
#[test]
fn empty_rule_is_quiet() {
    assert_eq!(vec![] as Vec<&str>, frames("a > {}", scss()),);
}

/// A trailing combinator used only for nesting is legal.
#[test]
fn nesting_only_is_quiet() {
    assert_eq!(vec![] as Vec<&str>, frames("a > {b {c: d}}", scss()),);
}

/// The declaration after a nested rule moves to a copy of the rule, which is not reported.
#[test]
fn split_rule_is_quiet() {
    assert_eq!(vec![] as Vec<&str>, frames("a > {b {c: d} e: f}", scss()),);
}

/// A placeholder rule is not reported.
#[test]
fn placeholder_is_quiet() {
    assert_eq!(vec![] as Vec<&str>, frames("%p > {b: c}", scss()),);
}

/// A mixin included twice reports its warning once, as dart-sass does.
#[test]
fn same_warning_is_reported_once() {
    assert_eq!(
        1,
        frames(
            "@mixin m {a + > b {c: d}}\n@include m;\n@include m;",
            scss()
        )
        .len()
    );
}

/// Seven distinct warnings: five are reported and two are counted.
#[test]
fn repetitions_past_five_are_counted() {
    let input = (1..=7)
        .map(|i| format!("a{i} + > b {{c: d}}\n"))
        .collect::<String>();
    let recorder = Recorder::default();
    accent_sass::from_string(input, &accent_sass::Options::default().logger(&recorder)).unwrap();

    assert_eq!(5, recorder.warnings.take().len());
    assert_eq!(Some(2), recorder.omitted.take());
}

/// `verbose` reports all seven and counts nothing.
#[test]
fn verbose_reports_every_repetition() {
    let input = (1..=7)
        .map(|i| format!("a{i} + > b {{c: d}}\n"))
        .collect::<String>();
    let recorder = Recorder::default();
    accent_sass::from_string(
        input,
        &accent_sass::Options::default()
            .verbose(true)
            .logger(&recorder),
    )
    .unwrap();

    assert_eq!(7, recorder.warnings.take().len());
    assert_eq!(None, recorder.omitted.take());
}

/// `quiet` reports nothing, and counts nothing either.
#[test]
fn quiet_reports_nothing() {
    let recorder = Recorder::default();
    accent_sass::from_string(
        "a > {b: c}".to_owned(),
        &accent_sass::Options::default()
            .quiet(true)
            .logger(&recorder),
    )
    .unwrap();

    assert!(recorder.warnings.take().is_empty());
    assert_eq!(None, recorder.omitted.take());
}

/// A logger that does not handle deprecations still receives the message
/// through `warn`.
#[test]
fn default_handler_forwards_to_warn() {
    #[derive(Debug, Default)]
    struct WarnOnly(RefCell<Vec<String>>);

    impl Logger for WarnOnly {
        fn debug(&self, _location: SpanLoc, _message: &str) {}

        fn warn(&self, _location: SpanLoc, message: &str) {
            self.0.borrow_mut().push(message.to_owned());
        }
    }

    let logger = WarnOnly::default();
    accent_sass::from_string(
        "a + > b {c: d}".to_owned(),
        &accent_sass::Options::default().logger(&logger),
    )
    .unwrap();

    assert_eq!(
        vec!["The selector \"a + > b\" is invalid CSS. It will be omitted from the generated CSS.\nThis will be an error in Dart Sass 2.0.0.\n\nMore info: https://sass-lang.com/d/bogus-combinators".to_owned()],
        logger.0.take()
    );
}
