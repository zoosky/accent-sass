//! Conversions between the semantic contents of a CSS identifier and the
//! escaped text that writes it.
//!
//! Since dart-sass 1.106.0 an identifier the parser reads is stored with its
//! escapes resolved, so `.\61` and `.a` name the same class. Escapes come back
//! only when the name is written out, through [`to_css_identifier`]. These are
//! dart-sass's `toCssIdentifier`, `resolveCssEscapes` and `characterToEscape`.

use std::borrow::Cow;

use super::{hex_char_for, is_name, is_name_start};

/// The maximum code point an escape may produce.
///
/// See <https://drafts.csswg.org/css-syntax-3/#maximum-allowed-code-point>.
pub(crate) const MAX_ALLOWED_CHARACTER: u32 = 0x10_FFFF;

/// Returns whether `c` is a private-use code point in the Basic Multilingual
/// Plane.
fn is_private_use_bmp(c: char) -> bool {
    matches!(c as u32, 0xE000..=0xF8FF)
}

/// Returns whether `c` is a private-use code point in a supplementary plane,
/// which UTF-16 encodes with the high surrogates `0xDB80..=0xDBFF`.
fn is_private_use_supplementary(c: char) -> bool {
    matches!(c as u32, 0xF_0000..=MAX_ALLOWED_CHARACTER)
}

/// Returns the escape sequence that writes `c` in an identifier, or `c` itself
/// when it needs no escaping.
///
/// If `identifier_start` is `true`, this also escapes the characters that are
/// valid in an identifier's body but not at its start, which are the digits.
/// Control characters and digits become hexadecimal escapes ending in a space;
/// anything else becomes a backslash before the character.
pub(crate) fn character_to_escape(c: char, identifier_start: bool) -> String {
    let valid = if identifier_start {
        is_name_start(c)
    } else {
        is_name(c)
    };

    if valid {
        return c.to_string();
    }

    let value = c as u32;
    if value <= 0x1F || value == 0x7F || (identifier_start && c.is_ascii_digit()) {
        let mut buffer = String::with_capacity(4);
        buffer.push('\\');
        if value > 0xF {
            buffer.push(hex_char_for(value >> 4));
        }
        buffer.push(hex_char_for(value & 0xF));
        buffer.push(' ');
        buffer
    } else {
        format!("\\{c}")
    }
}

/// Returns whether `text` is an identifier whose written form is exactly its
/// semantic contents, so it needs no escaping.
///
/// This is a fast path. It may answer `false` for text that needs no escapes,
/// but never `true` for text that does.
fn is_simple_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    match chars.next() {
        Some(c) if is_name_start(c) && !is_private_use_bmp(c) && (c as u32) <= 0xFFFF => {}
        _ => return false,
    }

    chars.all(|c| is_name(c) && !is_private_use_bmp(c) && (c as u32) <= 0xFFFF)
}

/// Returns a minimally escaped CSS identifier whose contents are `text`.
///
/// If `body` is `true`, the result continues an identifier rather than
/// starting one: a leading digit or `-` is written as is. That is the form a
/// parent selector's suffix takes.
///
/// # Errors
///
/// Returns the message dart-sass reports when `text` cannot be written as an
/// identifier at all: it is empty, or it contains U+0000.
pub(crate) fn to_css_identifier(text: &str, body: bool) -> Result<Cow<'_, str>, &'static str> {
    if is_simple_identifier(text) {
        return Ok(Cow::Borrowed(text));
    }

    fn write_hex_escape(buffer: &mut String, c: char) {
        buffer.push('\\');
        buffer.push_str(&format!("{:x}", c as u32));
        buffer.push(' ');
    }

    let mut buffer = String::with_capacity(text.len() + 4);
    let mut chars = text.chars().peekable();

    let mut double_dash = false;
    if !body && chars.peek() == Some(&'-') {
        chars.next();
        if chars.peek().is_none() {
            return Ok(Cow::Borrowed("\\2d "));
        }

        buffer.push('-');

        if chars.peek() == Some(&'-') {
            chars.next();
            buffer.push('-');
            double_dash = true;
        }
    }

    if !double_dash {
        match chars.next() {
            None => return Err("The empty string can't be represented as a CSS identifier."),
            Some('\0') => return Err("U+0000 can't be represented as a CSS identifier."),
            Some(c) if (c as u32) > 0xFFFF => {
                if is_private_use_supplementary(c) {
                    write_hex_escape(&mut buffer, c);
                } else {
                    buffer.push(c);
                }
            }
            Some(c) if body && is_name(c) && !is_private_use_bmp(c) => buffer.push(c),
            Some(c) if !body && is_name_start(c) && !is_private_use_bmp(c) => buffer.push(c),
            Some(c)
                if (c as u32) <= 0x1F
                    || c == '\u{7F}'
                    || c.is_ascii_digit()
                    || is_private_use_bmp(c) =>
            {
                write_hex_escape(&mut buffer, c);
            }
            Some(c) => {
                buffer.push('\\');
                buffer.push(c);
            }
        }
    }

    for c in chars {
        match c {
            '\0' => return Err("The U+0000 can't be represented as a CSS identifier."),
            c if (c as u32) > 0xFFFF => {
                if is_private_use_supplementary(c) {
                    write_hex_escape(&mut buffer, c);
                } else {
                    buffer.push(c);
                }
            }
            c if is_name(c) && !is_private_use_bmp(c) => buffer.push(c),
            c if (c as u32) <= 0x1F || c == '\u{7F}' || is_private_use_bmp(c) => {
                write_hex_escape(&mut buffer, c);
            }
            c => {
                buffer.push('\\');
                buffer.push(c);
            }
        }
    }

    Ok(Cow::Owned(buffer))
}

/// Like [`to_css_identifier`] with `body` unset, but writes `text` unchanged
/// when it cannot be represented as an identifier.
///
/// Selector names reach the serializer from the parser, which only produces
/// representable ones, so the fallback exists to keep serialization total.
pub(crate) fn css_identifier_or_text(text: &str) -> Cow<'_, str> {
    to_css_identifier(text, false).unwrap_or(Cow::Borrowed(text))
}

/// Consumes the escape sequence starting after a backslash and returns the
/// character it represents, as dart-sass's `consumeEscapedCharacter` does.
///
/// An escape of U+0000, of a surrogate or of a code point beyond
/// [`MAX_ALLOWED_CHARACTER`] produces U+FFFD REPLACEMENT CHARACTER, as does a
/// backslash at the end of the input. Returns `None` for a backslash before a
/// newline, which escapes nothing.
fn consume_escape(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<char> {
    match chars.peek().copied() {
        None => Some('\u{FFFD}'),
        Some('\n' | '\r' | '\u{C}') => None,
        Some(c) if c.is_ascii_hexdigit() => {
            let mut value = 0u32;
            for _ in 0..6 {
                match chars.peek() {
                    Some(c) if c.is_ascii_hexdigit() => {
                        value = (value << 4) + c.to_digit(16).unwrap();
                        chars.next();
                    }
                    _ => break,
                }
            }

            if matches!(chars.peek(), Some(c) if c.is_ascii_whitespace()) {
                chars.next();
            }

            Some(
                if value == 0 || (0xD800..=0xDFFF).contains(&value) || value > MAX_ALLOWED_CHARACTER
                {
                    '\u{FFFD}'
                } else {
                    char::from_u32(value).unwrap_or('\u{FFFD}')
                },
            )
        }
        Some(c) => {
            chars.next();
            Some(c)
        }
    }
}

/// Interprets `text` as a CSS identifier and returns its contents with every
/// escape resolved, as dart-sass's `resolveCssEscapes` does.
///
/// Returns `None` if `text` is not a single valid identifier. Keyword checks
/// go through this, so `@\6d edia` is a media rule and `\75 rl(` a URL.
pub(crate) fn resolve_css_escapes(text: &str) -> Option<Cow<'_, str>> {
    if is_simple_identifier(text) {
        return Some(Cow::Borrowed(text));
    }

    let mut chars = text.chars().peekable();
    let mut buffer = String::with_capacity(text.len());

    let mut double_dash = false;
    if chars.peek() == Some(&'-') {
        chars.next();
        buffer.push('-');
        if chars.peek() == Some(&'-') {
            chars.next();
            buffer.push('-');
            double_dash = true;
        }
    }

    if !double_dash {
        match chars.next()? {
            '\\' => buffer.push(consume_escape(&mut chars)?),
            c if is_name_start(c) => buffer.push(c),
            _ => return None,
        }
    }

    while let Some(c) = chars.next() {
        match c {
            '\\' => buffer.push(consume_escape(&mut chars)?),
            c if is_name(c) => buffer.push(c),
            _ => return None,
        }
    }

    Some(Cow::Owned(buffer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_css_identifier_matches_dart_sass() {
        let cases = [
            ("abc", "abc"),
            ("10", "\\31 0"),
            ("a.b", "a\\.b"),
            ("-a", "-a"),
            ("--a", "--a"),
            ("-", "\\2d "),
            ("-1", "-\\31 "),
            ("\u{E000}", "\\e000 "),
            ("a\u{7F}", "a\\7f "),
            ("\u{FFFD}", "\u{FFFD}"),
            ("\u{10FFFF}", "\\10ffff "),
            ("\u{1F600}", "\u{1F600}"),
            ("a b", "a\\ b"),
        ];
        for (text, expected) in cases {
            assert_eq!(
                to_css_identifier(text, false).unwrap(),
                expected,
                "{text:?}"
            );
        }
        assert_eq!(to_css_identifier("1a", true).unwrap(), "1a");
        assert!(to_css_identifier("", false).is_err());
        assert!(to_css_identifier("a\0", false).is_err());
    }

    #[test]
    fn resolve_css_escapes_matches_dart_sass() {
        assert_eq!(resolve_css_escapes("\\6d edia").as_deref(), Some("media"));
        assert_eq!(resolve_css_escapes("\\75 rl").as_deref(), Some("url"));
        assert_eq!(resolve_css_escapes("a\\0").as_deref(), Some("a\u{FFFD}"));
        assert_eq!(resolve_css_escapes("\\31 x").as_deref(), Some("1x"));
        assert_eq!(resolve_css_escapes("--\\61").as_deref(), Some("--a"));
        assert_eq!(resolve_css_escapes("1x"), None);
        assert_eq!(resolve_css_escapes("a b"), None);
        assert_eq!(resolve_css_escapes(""), None);
    }

    #[test]
    fn character_to_escape_matches_dart_sass() {
        assert_eq!(character_to_escape('a', true), "a");
        assert_eq!(character_to_escape('1', true), "\\31 ");
        assert_eq!(character_to_escape('1', false), "1");
        assert_eq!(character_to_escape('\u{FFFD}', true), "\u{FFFD}");
        assert_eq!(character_to_escape('\u{1F}', false), "\\1f ");
        assert_eq!(character_to_escape('.', false), "\\.");
    }
}
