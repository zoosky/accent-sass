//! Parsing the CSS color syntax: the space-separated `$channels` argument
//! of `rgb()`, `hsl()`, `hwb()`, `lab()`, `lch()`, `oklab()`, `oklch()`,
//! and the `$description` of `color()`.
//!
//! This is a port of `_parseChannels`, `_parseSlashChannels`,
//! `_colorFromChannels`, and `_channelFromValue` from Dart Sass's
//! `lib/src/functions/color.dart`. The channels can be numbers, the
//! keyword `none` for a missing channel, or values that only CSS can
//! evaluate (`var()`, `calc()` with unknown terms), in which case the
//! function call is emitted as plain CSS.

use crate::{
    builtin::builtin_imports::*,
    color::{clamp_like_css, ChannelKind, ColorChannel, ColorFormat, ColorSpace},
};

use super::{
    angle_value, assert_common_list_style, assert_unquoted_string, dart_to_string, function_string,
    is_none, percentage_or_unitless, with_name,
};

/// The spaces whose special-value fallback keeps the legacy comma
/// syntax: `rgb(var(--r), 0, 0, 0.5)` rather than `rgb(var(--r) 0 0 / 0.5)`.
fn uses_comma_fallback(space: Option<ColorSpace>) -> bool {
    matches!(space, Some(ColorSpace::Rgb) | Some(ColorSpace::Hsl))
}

/// Parses the single argument of a CSS color function into a color, or
/// into the function call as a plain-CSS string when the argument holds a
/// value only CSS can evaluate (Dart Sass's `_parseChannels`).
///
/// `space` is the function's space, or `None` for `color()`, whose first
/// component names the space. `name` is the argument name for error
/// messages.
pub(crate) fn parse_channels(
    function_name: &str,
    input: Value,
    space: Option<ColorSpace>,
    name: Option<&str>,
    visitor: &Visitor,
    span: Span,
) -> SassResult<Value> {
    if input.is_var() {
        return function_string(function_name, &[input], visitor, span);
    }

    let (components, alpha_value) = match parse_slash_channels(&input, name, span)? {
        Some(parsed) => parsed,
        None => return function_string(function_name, &[input], visitor, span),
    };

    let mut space = space;
    let list = assert_common_list_style(&components, name, false, span)?;

    let channels: Vec<Value> = if list.is_empty() {
        return Err((
            with_name(name, "Color component list may not be empty.".to_owned()),
            span,
        )
            .into());
    } else if matches!(&list[0], Value::String(text, QuoteKind::None) if text.eq_ignore_ascii_case("from"))
    {
        return function_string(function_name, &[input], visitor, span);
    } else if components.is_var() {
        vec![components]
    } else {
        let channels = if space.is_none() {
            let first = list[0].clone();
            let is_var = first.is_var();
            let space_name = assert_unquoted_string(first, name.unwrap_or("list"), span)?;
            if !is_var {
                let named = match ColorSpace::from_name(&space_name) {
                    Some(space) => space,
                    None => {
                        return Err((
                            with_name(name, format!("Unknown color space \"{}\".", space_name)),
                            span,
                        )
                            .into())
                    }
                };
                if matches!(
                    named,
                    ColorSpace::Rgb
                        | ColorSpace::Hsl
                        | ColorSpace::Hwb
                        | ColorSpace::Lab
                        | ColorSpace::Lch
                        | ColorSpace::Oklab
                        | ColorSpace::Oklch
                ) {
                    return Err((
                        with_name(
                            name,
                            format!(
                                "The color() function doesn't support the color space {space}. Use the {space}() function instead.",
                                space = named.name(),
                            ),
                        ),
                        span,
                    )
                        .into());
                }
                space = Some(named);
            }
            list[1..].to_vec()
        } else {
            list
        };

        for (index, channel) in channels.iter().enumerate() {
            if !channel.is_special_function()
                && !matches!(channel, Value::Dimension(..))
                && !is_none(channel)
            {
                let channel_name = match space {
                    Some(space) if index < 3 => {
                        format!("{} channel", space.channels()[index].name)
                    }
                    _ => format!("channel {}", index + 1),
                };
                return Err((
                    with_name(
                        name,
                        format!(
                            "Expected {} to be a number, was {}.",
                            channel_name,
                            dart_to_string(channel, span)?
                        ),
                    ),
                    span,
                )
                    .into());
            }
        }

        channels
    };

    if alpha_value
        .as_ref()
        .map_or(false, Value::is_special_function)
    {
        return if channels.len() == 3 && uses_comma_fallback(space) {
            let mut args = channels;
            args.extend(alpha_value);
            function_string(function_name, &args, visitor, span)
        } else {
            function_string(function_name, &[input], visitor, span)
        };
    }

    let alpha = match &alpha_value {
        None => Some(1.0),
        Some(Value::String(text, QuoteKind::None)) if text == "none" => None,
        Some(value) => {
            let number = value
                .clone()
                .assert_number_with_name(name.unwrap_or("list"), span)?;
            Some(clamp_like_css(
                percentage_or_unitless(&number, 1.0, "alpha", span)?,
                0.0,
                1.0,
            ))
        }
    };

    let space = match space {
        Some(space) => space,
        None => return function_string(function_name, &[input], visitor, span),
    };

    if channels.iter().any(Value::is_special_function) {
        return if channels.len() == 3 && uses_comma_fallback(Some(space)) {
            let mut args = channels;
            args.extend(alpha_value);
            function_string(function_name, &args, visitor, span)
        } else {
            function_string(function_name, &[input], visitor, span)
        };
    }

    if channels.len() != 3 {
        return Err((
            with_name(
                name,
                format!(
                    "The {} color space has 3 channels but {} has {}.",
                    space.name(),
                    dart_to_string(&input, span)?,
                    channels.len()
                ),
            ),
            span,
        )
            .into());
    }

    let number = |value: &Value| match value {
        Value::Dimension(number) => Some(number.clone()),
        _ => None,
    };

    Ok(Value::Color(Arc::new(color_from_channels(
        space,
        number(&channels[0]),
        number(&channels[1]),
        number(&channels[2]),
        alpha,
        true,
        space == ColorSpace::Rgb,
        span,
    )?)))
}

/// Splits the alpha off a channel list (Dart Sass's `_parseSlashChannels`):
/// a slash-separated pair, a trailing `3/0.5` written as a slash number, or
/// a trailing unquoted string containing a slash. Returns `None` when the
/// trailing string has more than one slash, which only CSS can evaluate.
fn parse_slash_channels(
    input: &Value,
    name: Option<&str>,
    span: Span,
) -> SassResult<Option<(Value, Option<Value>)>> {
    let list = assert_common_list_style(input, name, true, span)?;

    if input.separator() == ListSeparator::Slash {
        if list.len() == 2 {
            return Ok(Some((list[0].clone(), Some(list[1].clone()))));
        }
        return Err((
            with_name(
                name,
                format!(
                    "Only 2 slash-separated elements allowed, but {} {} passed.",
                    list.len(),
                    if list.len() == 1 { "was" } else { "were" }
                ),
            ),
            span,
        )
            .into());
    }

    let (last, initial) = match list.split_last() {
        Some(split) => split,
        None => return Ok(Some((input.clone(), None))),
    };

    match last {
        Value::String(text, QuoteKind::None) => {
            let parts: Vec<&str> = text.split('/').collect();
            match parts.as_slice() {
                [_] => Ok(Some((input.clone(), None))),
                [channel3, alpha] => {
                    let mut components = initial.to_vec();
                    components.push(parse_number_or_string(channel3));
                    Ok(Some((
                        Value::List(components, ListSeparator::Space, Brackets::None),
                        Some(parse_number_or_string(alpha)),
                    )))
                }
                _ => Ok(None),
            }
        }
        Value::Dimension(SassNumber {
            as_slash: Some(slash),
            ..
        }) => {
            let mut components = initial.to_vec();
            components.push(Value::Dimension(slash.0.clone()));
            Ok(Some((
                Value::List(components, ListSeparator::Space, Brackets::None),
                Some(Value::Dimension(slash.1.clone())),
            )))
        }
        _ => Ok(Some((input.clone(), None))),
    }
}

/// Parses `text` as a number literal with an optional unit, or keeps it
/// as an unquoted string (Dart Sass's `_parseNumberOrString`).
fn parse_number_or_string(text: &str) -> Value {
    let string = || Value::String(text.to_owned(), QuoteKind::None);
    let bytes = text.as_bytes();
    let mut index = 0;

    if matches!(bytes.first(), Some(b'+') | Some(b'-')) {
        index += 1;
    }

    let digits = |mut index: usize| {
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        index
    };

    let integer_end = digits(index);
    let mut has_digits = integer_end > index;
    index = integer_end;

    if index < bytes.len() && bytes[index] == b'.' {
        let fraction_end = digits(index + 1);
        if fraction_end > index + 1 {
            index = fraction_end;
            has_digits = true;
        }
    }

    if !has_digits {
        return string();
    }

    if index < bytes.len() && (bytes[index] == b'e' || bytes[index] == b'E') {
        let mut exponent = index + 1;
        if matches!(bytes.get(exponent), Some(b'+') | Some(b'-')) {
            exponent += 1;
        }
        let exponent_end = digits(exponent);
        if exponent_end > exponent {
            index = exponent_end;
        }
    }

    let value: f64 = match text[..index].parse() {
        Ok(value) => value,
        Err(..) => return string(),
    };

    let unit = &text[index..];
    let unit = if unit.is_empty() {
        Unit::None
    } else if unit == "%" {
        Unit::Percent
    } else if unit
        .bytes()
        .all(|b| b.is_ascii_alphabetic() || b == b'-' || b == b'_')
    {
        Unit::from(unit.to_owned())
    } else {
        return string();
    };

    Value::Dimension(SassNumber {
        num: Number(value),
        unit,
        as_slash: None,
    })
}

/// Builds a color in `space` from parsed channel numbers (Dart Sass's
/// `_colorFromChannels`). A `None` channel is missing. `clamp` says
/// whether channels that CSS clamps (rgb, lightness, chroma) are clamped,
/// which the CSS functions do and `color.change()` does not.
#[allow(clippy::too_many_arguments)]
pub(crate) fn color_from_channels(
    space: ColorSpace,
    channel0: Option<SassNumber>,
    channel1: Option<SassNumber>,
    channel2: Option<SassNumber>,
    alpha: Option<f64>,
    clamp: bool,
    from_rgb_function: bool,
    span: Span,
) -> SassResult<Color> {
    let channels = space.channels();

    let color = match space {
        ColorSpace::Hsl => Color::for_space(
            space,
            channel0
                .map(|hue| angle_value(Value::Dimension(hue), "hue", span))
                .transpose()?,
            channel_from_value(&channels[1], force_percent(channel1).as_ref(), clamp, span)?,
            channel_from_value(&channels[2], force_percent(channel2).as_ref(), clamp, span)?,
            alpha,
        ),
        ColorSpace::Hwb => {
            if let Some(whiteness) = &channel1 {
                whiteness.assert_unit(&Unit::Percent, "whiteness", span)?;
            }
            if let Some(blackness) = &channel2 {
                blackness.assert_unit(&Unit::Percent, "blackness", span)?;
            }
            let mut whiteness = channel1.map(|number| number.num.0);
            let mut blackness = channel2.map(|number| number.num.0);
            if let (Some(white), Some(black)) = (whiteness, blackness) {
                if white + black > 100.0 {
                    whiteness = Some(white / (white + black) * 100.0);
                    blackness = Some(black / (white + black) * 100.0);
                }
            }
            Color::for_space(
                space,
                channel0
                    .map(|hue| angle_value(Value::Dimension(hue), "hue", span))
                    .transpose()?,
                whiteness,
                blackness,
                alpha,
            )
        }
        _ => Color::for_space(
            space,
            channel_from_value(&channels[0], channel0.as_ref(), clamp, span)?,
            channel_from_value(&channels[1], channel1.as_ref(), clamp, span)?,
            channel_from_value(&channels[2], channel2.as_ref(), clamp, span)?,
            alpha,
        )
        .with_format(if from_rgb_function {
            ColorFormat::Rgb
        } else {
            ColorFormat::Infer
        }),
    };

    Ok(color)
}

/// Dart Sass's `_forcePercent`: reads a unitless hsl saturation or
/// lightness as a percentage (a deprecated but accepted form).
fn force_percent(number: Option<SassNumber>) -> Option<SassNumber> {
    number.map(|number| {
        if number.unit == Unit::Percent {
            number
        } else {
            SassNumber {
                num: number.num,
                unit: Unit::Percent,
                as_slash: None,
            }
        }
    })
}

/// Converts a channel argument to the channel's native units (Dart Sass's
/// `_channelFromValue`): a percentage scales to the channel's maximum, a
/// hue is coerced to degrees and wrapped, and clamped channels are clamped
/// when `clamp` is set.
pub(crate) fn channel_from_value(
    channel: &ColorChannel,
    value: Option<&SassNumber>,
    clamp: bool,
    span: Span,
) -> SassResult<Option<f64>> {
    let value = match value {
        Some(value) => value,
        None => return Ok(None),
    };

    Ok(Some(match channel.kind {
        ChannelKind::Linear {
            requires_percent: true,
            ..
        } if value.unit != Unit::Percent => {
            return Err(value
                .assert_unit(&Unit::Percent, channel.name, span)
                .unwrap_err());
        }
        ChannelKind::Linear {
            min,
            max,
            lower_clamped,
            upper_clamped,
            ..
        } => {
            let number = percentage_or_unitless(value, max, channel.name, span)?;
            if !clamp || (!lower_clamped && !upper_clamped) {
                number
            } else {
                clamp_like_css(
                    number,
                    if lower_clamped {
                        min
                    } else {
                        f64::NEG_INFINITY
                    },
                    if upper_clamped { max } else { f64::INFINITY },
                )
            }
        }
        ChannelKind::PolarAngle => coerce_to_degrees(value, channel.name, span)?.rem_euclid(360.0),
    }))
}

/// Dart Sass's `coerceValueToUnit('deg')`: a unitless number is taken as
/// degrees, an angle is converted, and any other unit is an error.
fn coerce_to_degrees(value: &SassNumber, name: &str, span: Span) -> SassResult<f64> {
    if value.unit == Unit::None {
        return Ok(value.num.0);
    }
    if value.has_compatible_units(&Unit::Deg) {
        return angle_value(Value::Dimension(value.clone()), name, span);
    }
    Err((
        format!(
            "${name}: Expected {value} to have an angle unit (deg, grad, rad, turn).",
            name = name,
            value = crate::serializer::inspect_number(value, &Options::default(), span)?,
        ),
        span,
    )
        .into())
}

/// The global `lab($channels)`, `lch($channels)`, `oklab($channels)`, and
/// `oklch($channels)` functions.
fn channels_function(
    function_name: &'static str,
    space: ColorSpace,
) -> impl Fn(ArgumentResult, &mut Visitor) -> SassResult<Value> {
    move |mut args: ArgumentResult, visitor: &mut Visitor| {
        args.max_args(1)?;
        let span = args.span();
        let channels = args.get_err(0, "channels")?;
        parse_channels(
            function_name,
            channels,
            Some(space),
            Some("channels"),
            visitor,
            span,
        )
    }
}

pub(crate) fn lab(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    channels_function("lab", ColorSpace::Lab)(args, visitor)
}

pub(crate) fn lch(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    channels_function("lch", ColorSpace::Lch)(args, visitor)
}

pub(crate) fn oklab(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    channels_function("oklab", ColorSpace::Oklab)(args, visitor)
}

pub(crate) fn oklch(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    channels_function("oklch", ColorSpace::Oklch)(args, visitor)
}

/// The global `color($description)` function: `color(srgb 1 0 0 / 0.5)`.
pub(crate) fn color(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let span = args.span();
    let description = args.get_err(0, "description")?;
    parse_channels(
        "color",
        description,
        None,
        Some("description"),
        visitor,
        span,
    )
}

pub(crate) fn declare(f: &mut GlobalFunctionMap) {
    f.insert("lab", Builtin::new(lab));
    f.insert("lch", Builtin::new(lch));
    f.insert("oklab", Builtin::new(oklab));
    f.insert("oklch", Builtin::new(oklch));
    f.insert("color", Builtin::new(color));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_number_literals_with_units() {
        match parse_number_or_string("0.5") {
            Value::Dimension(number) => {
                assert_eq!(number.num.0, 0.5);
                assert_eq!(number.unit, Unit::None);
            }
            other => panic!("expected a number, got {:?}", other),
        }
        match parse_number_or_string("50%") {
            Value::Dimension(number) => {
                assert_eq!(number.num.0, 50.0);
                assert_eq!(number.unit, Unit::Percent);
            }
            other => panic!("expected a number, got {:?}", other),
        }
        match parse_number_or_string("-1.5e2deg") {
            Value::Dimension(number) => {
                assert_eq!(number.num.0, -150.0);
                assert_eq!(number.unit, Unit::Deg);
            }
            other => panic!("expected a number, got {:?}", other),
        }
    }

    #[test]
    fn keeps_non_numbers_as_strings() {
        assert!(matches!(
            parse_number_or_string("var(--x)"),
            Value::String(text, QuoteKind::None) if text == "var(--x)"
        ));
        assert!(matches!(
            parse_number_or_string("none"),
            Value::String(text, QuoteKind::None) if text == "none"
        ));
        assert!(matches!(parse_number_or_string("1.2.3"), Value::String(..)));
    }
}
