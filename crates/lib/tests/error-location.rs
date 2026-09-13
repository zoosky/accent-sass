//! Where an error points: the source line and caret beneath the message.
//!
//! The `error!` macro compares only the first line, so a caret one column off
//! passes it. These tests compare the first four lines of the rendered error,
//! which is the part this compiler draws the same way dart-sass does; the
//! location line below them is written differently and is not compared.
//!
//! Every expectation was taken from the dart-sass 1.104.1 binary's output.

/// Asserts the message, the frame's top bar, the source line and the caret.
fn assert_location(input: &str, expected: &str) {
    let error = accent_sass::from_string(input.to_owned(), &accent_sass::Options::default())
        .expect_err("input should not compile");
    let rendered = error.to_string();
    let head = rendered.lines().take(4).collect::<Vec<_>>().join("\n");

    assert_eq!(expected, head);
}

/// Input that ends early is reported just past its last character.
#[test]
fn end_of_input_after_a_declaration() {
    assert_location(
        "a { b: c",
        "Error: expected end of rule.\n  ╷\n1 │ a { b: c\n  │         ^",
    );
}

/// An unclosed block, reported by the parser of its contents.
#[test]
fn end_of_input_after_an_open_brace() {
    assert_location(
        "@function foo() {",
        "Error: Expected identifier.\n  ╷\n1 │ @function foo() {\n  │                  ^",
    );
}

/// An empty span moves back over whitespace to the end of the previous line.
#[test]
fn end_of_input_after_a_trailing_newline() {
    assert_location(
        "a {\n  b: c\n",
        "Error: expected end of rule.\n  ╷\n2 │   b: c\n  │       ^",
    );
}

/// Spaces without a newline do not move the span.
#[test]
fn end_of_input_after_trailing_spaces() {
    assert_location(
        "a { b: c;   ",
        "Error: expected end of rule.\n  ╷\n1 │ a { b: c;   \n  │             ^",
    );
}

/// An error at the start of a line points at the end of the one before.
#[test]
fn missing_end_of_rule_at_line_start() {
    assert_location(
        "%x {}\n{}",
        "Error: expected end of rule.\n  ╷\n1 │ %x {}\n  │      ^",
    );
}

/// A missing operand points where it should be, not at the operator.
#[test]
fn missing_operand_at_end_of_input() {
    assert_location(
        "a {color: 1 +",
        "Error: Expected expression.\n  ╷\n1 │ a {color: 1 +\n  │              ^",
    );
}

/// The same in the middle of the input.
#[test]
fn missing_operand_before_semicolon() {
    assert_location(
        "a {b: 1 + ;}",
        "Error: Expected expression.\n  ╷\n1 │ a {b: 1 + ;}\n  │           ^",
    );
}

/// And at the end of a line.
#[test]
fn missing_operand_before_newline() {
    assert_location(
        "a {b: 1 +\n}",
        "Error: Expected expression.\n  ╷\n1 │ a {b: 1 +\n  │          ^",
    );
}

/// A missing expression at the end of input.
#[test]
fn missing_list_after_in() {
    assert_location(
        "@each $i in",
        "Error: Expected expression.\n  ╷\n1 │ @each $i in\n  │            ^",
    );
}

/// An empty value points at what follows it.
#[test]
fn empty_value() {
    assert_location(
        "a {b: ;}",
        "Error: Expected expression.\n  ╷\n1 │ a {b: ;}\n  │       ^",
    );
}

/// Directly after `#{`, dart-sass highlights the `#{` itself.
#[test]
fn empty_interpolation() {
    assert_location(
        "a {b: #{}}",
        "Error: Expected expression.\n  ╷\n1 │ a {b: #{}}\n  │       ^^",
    );
}

/// The same when the input ends there.
#[test]
fn empty_interpolation_at_end_of_input() {
    assert_location(
        "a {b: #{",
        "Error: Expected expression.\n  ╷\n1 │ a {b: #{\n  │       ^^",
    );
}

/// After whitespace, it points where the expression should start.
#[test]
fn empty_interpolation_with_space() {
    assert_location(
        "a {b: #{ }}",
        "Error: Expected expression.\n  ╷\n1 │ a {b: #{ }}\n  │          ^",
    );
}
