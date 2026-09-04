//! The color functions: the global CSS color functions (`rgb()`, `hsl()`,
//! `hwb()`, `lab()`, `lch()`, `oklab()`, `oklch()`, `color()`), the legacy
//! Sass color functions, and the `sass:color` module.
//!
//! The helpers in this module are shared by the submodules and mirror the
//! private helpers at the top of Dart Sass's `lib/src/functions/color.dart`.

use codemap::Span;

use crate::{
    builtin::builtin_imports::*,
    color::ColorSpace,
    error::SassError,
    serializer::inspect_number,
    value::{Value, conversion_factor},
};

use super::GlobalFunctionMap;

pub mod hsl;
pub mod hwb;
pub mod opacity;
pub mod other;
pub mod parse;
pub mod rgb;

/// Prefixes `message` with `$name: ` when the error is attributed to an
/// argument, as Dart Sass's `SassScriptException` does.
pub(crate) fn with_name(name: Option<&str>, message: String) -> String {
    match name {
        Some(name) => format!("${}: {}", name, message),
        None => message,
    }
}

/// Reads an angle argument in degrees (Dart Sass's `_angleValue`). A
/// number with an angle unit is converted; any other number is taken as
/// degrees, which Dart Sass deprecates but still accepts.
pub(crate) fn angle_value(value: Value, name: &str, span: Span) -> SassResult<f64> {
    let angle = value.assert_number_with_name(name, span)?;

    if angle.has_compatible_units(&Unit::Deg) {
        let factor = conversion_factor(&angle.unit, &Unit::Deg).unwrap();
        return Ok(angle.num.0 * factor);
    }

    Ok(angle.num.0)
}

/// Dart Sass's `Value.toString()`: the inspected form, with a
/// multi-element list wrapped in parentheses the way `SassList.toString()`
/// writes it.
pub(crate) fn dart_to_string(value: &Value, span: Span) -> SassResult<String> {
    let inspected = value.inspect(span)?;
    let wrap = match value {
        Value::List(items, separator, Brackets::None) => {
            !(items.is_empty() || (items.len() == 1 && *separator == ListSeparator::Comma))
        }
        Value::ArgList(args) => !(args.is_empty() || args.len() == 1),
        _ => false,
    };

    Ok(if wrap {
        format!("({})", inspected)
    } else {
        inspected
    })
}

/// Dart Sass's `Value.assertCommonListStyle`: the elements of a
/// space-separated (or, when `allow_slash` is set, slash-separated)
/// unbracketed list. A single value counts as a one-element list.
pub(crate) fn assert_common_list_style(
    value: &Value,
    name: Option<&str>,
    allow_slash: bool,
    span: Span,
) -> SassResult<Vec<Value>> {
    let separator = value.separator();
    let invalid_separator =
        separator == ListSeparator::Comma || (!allow_slash && separator == ListSeparator::Slash);
    let has_brackets = matches!(value, Value::List(_, _, Brackets::Bracketed));

    if !invalid_separator && !has_brackets {
        return Ok(value.clone().as_list());
    }

    let mut message = String::from("Expected");
    if has_brackets {
        message.push_str(" an unbracketed");
    }
    if invalid_separator {
        message.push_str(if has_brackets { "," } else { " a" });
        message.push_str(" space-");
        if allow_slash {
            message.push_str(" or slash-");
        }
        message.push_str("separated");
    }
    message.push_str(" list, was ");
    message.push_str(&dart_to_string(value, span)?);

    Err((with_name(name, message), span).into())
}

/// Asserts that `value` is an unquoted string, the way Dart Sass's
/// `assertString(name)..assertUnquoted(name)` does, and returns its text.
pub(crate) fn assert_unquoted_string(value: Value, name: &str, span: Span) -> SassResult<String> {
    let (text, quotes) = value.assert_string_with_name(name, span)?;
    if quotes != QuoteKind::None {
        return Err((
            format!(
                "${name}: Expected \"{text}\" to be an unquoted string.",
                name = name,
                text = text,
            ),
            span,
        )
            .into());
    }
    Ok(text)
}

/// Parses a color space name (case-insensitively) from an unquoted string.
pub(crate) fn space_from_value(value: Value, name: &str, span: Span) -> SassResult<ColorSpace> {
    let text = assert_unquoted_string(value, name, span)?;
    match ColorSpace::from_name(&text) {
        Some(space) => Ok(space),
        None => Err((
            format!(
                "${name}: Unknown color space \"{space}\".",
                name = name,
                space = text,
            ),
            span,
        )
            .into()),
    }
}

/// Dart Sass's `_colorInSpace`: converts `color` into the space named by
/// the `$space` argument, or returns it unchanged when that argument is
/// null or absent.
pub(crate) fn color_in_space(
    color: &Color,
    space: Option<Spanned<Value>>,
    legacy_missing: bool,
) -> SassResult<Color> {
    match space {
        Some(space) if space.node != Value::Null => Ok(color.to_space(
            space_from_value(space.node, "space", space.span)?,
            legacy_missing,
        )),
        _ => Ok(color.clone()),
    }
}

/// Whether `value` is the unquoted string `none` (case-insensitively),
/// which marks a missing channel.
pub(crate) fn is_none(value: &Value) -> bool {
    matches!(value, Value::String(text, QuoteKind::None) if text.eq_ignore_ascii_case("none"))
}

/// Serializes a plain-CSS function call whose arguments could not be
/// evaluated (`rgb(var(--x), 0, 0)`), as Dart Sass's `_functionString`.
/// The arguments are written with `toCssString()`, which is never
/// compressed, so the output is the same in every style.
pub(crate) fn function_string(
    name: &str,
    args: &[Value],
    _visitor: &Visitor,
    span: Span,
) -> SassResult<Value> {
    let args = args
        .iter()
        .map(|arg| arg.to_css_string(span, false))
        .collect::<SassResult<Vec<_>>>()?
        .join(", ");

    Ok(Value::String(
        format!("{}({})", name, args),
        QuoteKind::None,
    ))
}

/// Dart Sass's `_percentageOrUnitless`: a unitless number is taken as-is,
/// a percentage is scaled to `max`, and any other unit is an error.
pub(crate) fn percentage_or_unitless(
    number: &SassNumber,
    max: f64,
    name: &str,
    span: Span,
) -> SassResult<f64> {
    if number.unit == Unit::None {
        Ok(number.num.0)
    } else if number.unit == Unit::Percent {
        Ok(max * number.num.0 / 100.0)
    } else {
        Err((
            format!(
                "${name}: Expected {number} to have unit \"%\" or no units.",
                name = name,
                number = inspect_number(number, &Options::default(), span)?,
            ),
            span,
        )
            .into())
    }
}

/// Serializes a color the way `$color` prints in a Dart Sass error message
/// (`Value.toString()`, the inspected form).
pub(crate) fn color_to_string(color: &Color, span: Span) -> SassResult<String> {
    Value::Color(Arc::new(color.clone())).inspect(span)
}

/// Serializes a color the way Dart Sass's `color.toCssString()` does.
pub(crate) fn color_css_string(color: &Color, span: Span) -> SassResult<String> {
    Value::Color(Arc::new(color.clone())).to_css_string(span, false)
}

/// The error Dart Sass raises when a function would modify a missing
/// channel. `channel` is the argument name the error is attributed to.
pub(crate) fn missing_channel_error(color: &Color, channel: &str, span: Span) -> Box<SassError> {
    let color = match color_css_string(color, span) {
        Ok(color) => color,
        Err(e) => return e,
    };
    (
        format!(
            "${channel}: Because the CSS working group is still deciding on the best behavior, Sass doesn't currently support modifying missing channels (color: {color}).",
            channel = channel,
            color = color,
        ),
        span,
    )
        .into()
}

/// The error for a legacy-only function applied to a non-legacy color:
/// `function() is only supported for legacy colors. Please use suggestion
/// instead with an explicit $space argument.` (`with_space` adds the last
/// clause).
pub(crate) fn legacy_only_error(
    function: &str,
    suggestion: &str,
    with_space: bool,
    span: Span,
) -> Box<SassError> {
    (
        format!(
            "{function}() is only supported for legacy colors. Please use {suggestion} instead{argument}.",
            function = function,
            suggestion = suggestion,
            argument = if with_space {
                " with an explicit $space argument"
            } else {
                ""
            },
        ),
        span,
    )
        .into()
}

pub(crate) fn declare(f: &mut GlobalFunctionMap) {
    hsl::declare(f);
    hwb::declare(f);
    opacity::declare(f);
    other::declare(f);
    parse::declare(f);
    rgb::declare(f);
}
