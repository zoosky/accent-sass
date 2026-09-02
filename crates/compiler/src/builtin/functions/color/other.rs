//! The `sass:color` functions that operate on a color's channels in a
//! chosen space: `channel`, `adjust`, `change`, `scale`, and the color-space
//! introspection family (`space`, `to-space`, `is-legacy`, `is-in-gamut`,
//! `to-gamut`, `same`).
//!
//! Only the three legacy spaces (rgb, hsl, hwb) are implemented. Any other
//! CSS Color 4 space name is recognized so the error can say so, and the
//! unbounded spaces (lab, lch, oklab, oklch, xyz) are accepted where Dart
//! Sass treats them as a no-op (`to-gamut`, `is-in-gamut`).

use crate::{
    builtin::{builtin_imports::*, color::angle_value},
    color::{clamp_like_css, ColorSpace, GamutMapMethod},
    error::SassError,
    serializer::inspect_number,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum UpdateComponents {
    Change,
    Adjust,
    Scale,
}

/// A color space named by a `$space` (or `$method`) argument.
///
/// Dart Sass accepts every CSS Color 4 space; this implementation only
/// converts between the legacy ones. The other names are kept apart so
/// callers can pick between an "unsupported" error and Dart Sass's own
/// behavior for spaces it never needs to convert into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpaceArg {
    Legacy(ColorSpace),
    /// `lab`, `lch`, `oklab`, `oklch`, `xyz`, `xyz-d50`, `xyz-d65`: spaces
    /// with no gamut, so every color is in gamut for them.
    Unbounded {
        name: &'static str,
        polar: bool,
    },
    /// `srgb`, `srgb-linear`, `display-p3`, `display-p3-linear`, `a98-rgb`,
    /// `prophoto-rgb`, `rec2020`: bounded rectangular spaces.
    Bounded(&'static str),
}

impl SpaceArg {
    /// The name as Dart Sass prints it.
    pub(crate) fn name(self) -> &'static str {
        match self {
            SpaceArg::Legacy(space) => space.name(),
            SpaceArg::Unbounded { name, .. } | SpaceArg::Bounded(name) => name,
        }
    }

    /// Whether the space has a hue channel.
    pub(crate) fn is_polar(self) -> bool {
        match self {
            SpaceArg::Legacy(space) => space.is_polar(),
            SpaceArg::Unbounded { polar, .. } => polar,
            SpaceArg::Bounded(..) => false,
        }
    }

    /// The legacy space, or the "not supported by this implementation"
    /// error for any other space.
    pub(crate) fn legacy(self, name: &str, span: Span) -> SassResult<ColorSpace> {
        match self {
            SpaceArg::Legacy(space) => Ok(space),
            _ => Err((
                format!(
                    "${name}: Color space {space} is not supported by this implementation (rgb, hsl, and hwb are).",
                    name = name,
                    space = self.name(),
                ),
                span,
            )
                .into()),
        }
    }
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
pub(crate) fn parse_space(value: Value, name: &str, span: Span) -> SassResult<SpaceArg> {
    let text = assert_unquoted_string(value, name, span)?;

    if let Some(space) = ColorSpace::from_name(&text) {
        return Ok(SpaceArg::Legacy(space));
    }

    Ok(match text.to_ascii_lowercase().as_str() {
        "lab" => SpaceArg::Unbounded {
            name: "lab",
            polar: false,
        },
        "lch" => SpaceArg::Unbounded {
            name: "lch",
            polar: true,
        },
        "oklab" => SpaceArg::Unbounded {
            name: "oklab",
            polar: false,
        },
        "oklch" => SpaceArg::Unbounded {
            name: "oklch",
            polar: true,
        },
        "xyz" | "xyz-d65" => SpaceArg::Unbounded {
            name: "xyz",
            polar: false,
        },
        "xyz-d50" => SpaceArg::Unbounded {
            name: "xyz-d50",
            polar: false,
        },
        "srgb" => SpaceArg::Bounded("srgb"),
        "srgb-linear" => SpaceArg::Bounded("srgb-linear"),
        "display-p3" => SpaceArg::Bounded("display-p3"),
        "display-p3-linear" => SpaceArg::Bounded("display-p3-linear"),
        "a98-rgb" => SpaceArg::Bounded("a98-rgb"),
        "prophoto-rgb" => SpaceArg::Bounded("prophoto-rgb"),
        "rec2020" => SpaceArg::Bounded("rec2020"),
        _ => {
            return Err((
                format!(
                    "${name}: Unknown color space \"{space}\".",
                    name = name,
                    space = text,
                ),
                span,
            )
                .into())
        }
    })
}

/// Reads the optional `$space` argument at `position`, requiring a legacy
/// space when it is given.
fn legacy_space_arg(args: &mut ArgumentResult, position: usize) -> SassResult<Option<ColorSpace>> {
    match args.get(position, "space") {
        Some(space) if space.node != Value::Null => Ok(Some(
            parse_space(space.node, "space", space.span)?.legacy("space", space.span)?,
        )),
        _ => Ok(None),
    }
}

/// Serializes a color for an error message the way Dart Sass's
/// `color.toCssString()` does.
fn color_css_string(color: &Color, span: Span) -> SassResult<String> {
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

/// Converts `color` into the legacy space named by `$space` (keeping a
/// missing hue), or returns it unchanged when no `$space` was given. This
/// is Dart Sass's `_colorInSpace`.
fn color_in_space(color: &Color, space: Option<ColorSpace>) -> Color {
    match space {
        Some(space) => color.to_space(space, true),
        None => color.clone(),
    }
}

pub(crate) fn channel(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(3)?;
    let span = args.span();
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", span)?;
    let (channel_name, quotes) = args
        .get_err(1, "channel")?
        .assert_string_with_name("channel", span)?;
    if quotes == QuoteKind::None {
        return Err((
            format!("$channel: Expected {} to be a quoted string.", channel_name),
            span,
        )
            .into());
    }
    let space = legacy_space_arg(&mut args, 2)?;

    let color = color_in_space(&color, space);

    if channel_name == "alpha" {
        return Ok(Value::Dimension(SassNumber::new_unitless(color.alpha())));
    }

    let value = match color.native_channel(&channel_name) {
        Some(value) => value,
        None => {
            return Err((
                format!(
                    "$channel: Color {} has no channel named {}.",
                    Value::Color(Arc::new(color)).inspect(span)?,
                    channel_name
                ),
                span,
            )
                .into())
        }
    };

    let unit = match (color.space(), channel_name.as_str()) {
        (_, "hue") => Unit::Deg,
        (ColorSpace::Rgb, _) => Unit::None,
        _ => Unit::Percent,
    };

    Ok(Value::Dimension(SassNumber {
        num: value,
        unit,
        as_slash: None,
    }))
}

/// `color.space($color)`: the name of the color's space.
pub(crate) fn space(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    Ok(Value::String(
        color.space().name().to_owned(),
        QuoteKind::None,
    ))
}

/// `color.to-space($color, $space)`: converts between the legacy spaces.
/// Dart Sass never returns a missing channel from `to-space` for a legacy
/// space, so an achromatic result gets a hue of `0`.
pub(crate) fn to_space(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let span = args.span();
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", span)?;
    let space = args.get_err(1, "space")?;
    let space = parse_space(space, "space", span)?.legacy("space", span)?;

    Ok(Value::Color(Arc::new(color.to_space(space, false))))
}

/// `color.is-legacy($color)`: always true, since every color this
/// implementation can represent is in a legacy space.
pub(crate) fn is_legacy(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    args.get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    Ok(Value::True)
}

/// `color.is-in-gamut($color, $space: null)`.
pub(crate) fn is_in_gamut(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let span = args.span();
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", span)?;

    let space = match args.get(1, "space") {
        Some(space) if space.node != Value::Null => {
            match parse_space(space.node, "space", space.span)? {
                SpaceArg::Legacy(space) => Some(space),
                // A space without a gamut holds every color.
                SpaceArg::Unbounded { .. } => return Ok(Value::True),
                other => return Err(other.legacy("space", span).unwrap_err()),
            }
        }
        _ => None,
    };

    Ok(bool_value(color_in_space(&color, space).is_in_gamut()))
}

/// `color.to-gamut($color, $space: null, $method: null)`.
pub(crate) fn to_gamut(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(3)?;
    let span = args.span();
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", span)?;

    let space = match args.get(1, "space") {
        Some(space) if space.node != Value::Null => {
            Some(parse_space(space.node, "space", space.span)?)
        }
        _ => None,
    };

    let method = match args.get(2, "method") {
        Some(method) if method.node != Value::Null => method,
        _ => {
            return Err((
                "$method: color.to-gamut() requires a $method argument for forwards-compatibility with changes in the CSS spec. Suggestion:\n\n$method: local-minde",
                span,
            )
                .into())
        }
    };

    // Dart Sass validates the method before checking whether the space is
    // bounded, so an invalid method name always errors.
    let method_name = assert_unquoted_string(method.node, "method", method.span)?;
    let method = match GamutMapMethod::from_name(&method_name) {
        Some(method) => method,
        None => {
            return Err((
                format!("Unknown gamut map method \"{}\".", method_name),
                method.span,
            )
                .into())
        }
    };

    let space = match space {
        None => color.space(),
        Some(SpaceArg::Legacy(space)) => space,
        Some(SpaceArg::Unbounded { .. }) => return Ok(Value::Color(color)),
        Some(other) => return Err(other.legacy("space", span).unwrap_err()),
    };

    Ok(Value::Color(Arc::new(
        color
            .to_space(space, true)
            .to_gamut(method)
            .to_space(color.space(), false),
    )))
}

/// `color.same($color1, $color2)`.
pub(crate) fn same(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let span = args.span();
    let color1 = args
        .get_err(0, "color1")?
        .assert_color_with_name("color1", span)?;
    let color2 = args
        .get_err(1, "color2")?
        .assert_color_with_name("color2", span)?;

    Ok(bool_value(color1.same(&color2)))
}

/// Picks the legacy space to operate in when `color.adjust()` and friends
/// are called without `$space`, from the channel keywords that were
/// passed. Dart Sass looks at the keywords in argument order; grass keeps
/// named arguments sorted by name, so for a (mixed, always erroring)
/// keyword set the space named in the error can differ from Dart Sass's.
fn sniff_legacy_color_space(keywords: &[String]) -> Option<ColorSpace> {
    for key in keywords {
        match key.as_str() {
            "red" | "green" | "blue" => return Some(ColorSpace::Rgb),
            "saturation" | "lightness" => return Some(ColorSpace::Hsl),
            "whiteness" | "blackness" => return Some(ColorSpace::Hwb),
            _ => {}
        }
    }

    if keywords.iter().any(|key| key == "hue") {
        Some(ColorSpace::Hsl)
    } else {
        None
    }
}

/// Dart Sass's `_percentageOrUnitless` without the clamp: a unitless
/// number is taken as-is, a percentage is scaled to `max`, and any other
/// unit is an error.
fn percentage_or_unitless(
    number: &SassNumber,
    max: f64,
    name: &str,
    span: Span,
    visitor: &Visitor,
) -> SassResult<f64> {
    if number.unit == Unit::None {
        Ok(number.num.0)
    } else if number.unit == Unit::Percent {
        Ok(number.num.0 * max / 100.0)
    } else {
        Err((
            format!(
                "${name}: Expected {} to have unit \"%\" or no units.",
                inspect_number(number, visitor.options, span)?,
                name = name,
            ),
            span,
        )
            .into())
    }
}

/// Reads an alpha argument for `change`/`adjust`: unitless, or a
/// percentage scaled to `0..1`. Dart Sass also accepts (and deprecates)
/// other units, taking the bare value.
fn alpha_value(number: &SassNumber) -> f64 {
    if number.unit == Unit::Percent {
        number.num.0 / 100.0
    } else {
        number.num.0
    }
}

/// The per-space channel argument conversion shared by `adjust` and
/// `change` (Dart Sass's `_channelFromValue` plus the deprecation-period
/// unit handling in `_adjustChannel`/`_colorFromChannels`).
///
/// Returns the value in the channel's native units without clamping.
fn channel_value(
    space: ColorSpace,
    channel: &str,
    value: Value,
    span: Span,
    visitor: &Visitor,
) -> SassResult<f64> {
    if channel == "hue" {
        return Ok(angle_value(value, "hue", span)?.0);
    }

    let number = value.assert_number_with_name(channel, span)?;

    match space {
        ColorSpace::Rgb => percentage_or_unitless(&number, 255.0, channel, span, visitor),
        // Saturation and lightness are percentages; a number without a unit
        // is accepted (deprecated in Dart Sass) and read as a percentage.
        ColorSpace::Hsl => Ok(number.num.0),
        ColorSpace::Hwb => {
            number.assert_unit(&Unit::Percent, channel, span)?;
            Ok(number.num.0)
        }
    }
}

/// Reads a `color.scale()` factor: a percentage within `-100%..100%`,
/// returned on the `-1..1` scale.
fn scale_factor(value: Value, channel: &str, span: Span) -> SassResult<f64> {
    let number = value.assert_number_with_name(channel, span)?;
    number.assert_unit(&Unit::Percent, channel, span)?;
    number.assert_bounds(channel, -100.0, 100.0, span)?;
    Ok(number.num.0 / 100.0)
}

/// Dart Sass's `_scaleChannel`: moves `old` toward the channel's bound by
/// `factor`, never past a value that is already out of range.
fn scale_channel(old: f64, factor: f64, min: f64, max: f64) -> f64 {
    if factor == 0.0 {
        old
    } else if factor > 0.0 {
        if old >= max {
            old
        } else {
            old + (max - old) * factor
        }
    } else if old <= min {
        old
    } else {
        old + (old - min) * factor
    }
}

/// Dart Sass's `_adjustChannel` clamping for a channel with a lower and/or
/// upper clamp: a result that crosses a bound is clamped to it, unless the
/// old value was already past that bound, in which case the adjustment may
/// only move it toward the bound.
fn adjust_clamped(old: f64, result: f64, lower: Option<f64>, upper: Option<f64>) -> f64 {
    if let Some(min) = lower {
        if result < min {
            return if old < min { old.max(result) } else { min };
        }
    }
    if let Some(max) = upper {
        if result > max {
            return if old > max { old.min(result) } else { max };
        }
    }
    result
}

fn update_components(
    mut args: ArgumentResult,
    visitor: &mut Visitor,
    update: UpdateComponents,
) -> SassResult<Value> {
    let span = args.span();
    let original = args
        .get_err(0, "color")?
        .assert_color_with_name("color", span)?;

    if args.positional.len() > 1 {
        return Err((
            "Only one positional argument is allowed. All other arguments must be passed by name.",
            span,
        )
            .into());
    }

    let space_arg = legacy_space_arg(&mut args, usize::MAX)?;
    let alpha_arg = args.get(usize::MAX, "alpha");

    let keywords: Vec<String> = args
        .named
        .keys()
        .map(|key| key.as_str().to_owned())
        .collect();

    let space = match space_arg {
        Some(space) => space,
        None => sniff_legacy_color_space(&keywords).unwrap_or_else(|| original.space()),
    };

    // With an explicit `$space` the conversion keeps a missing hue, which
    // then refuses modification; without one, a powerless hue reads as 0.
    let color = match space_arg {
        Some(space) => original.to_space(space, true),
        None if keywords.is_empty() => (*original).clone(),
        None => original.to_space(space, false),
    };

    let channel_names = space.channel_names();
    let mut channel_args: [Option<Spanned<Value>>; 3] = [None, None, None];
    for key in &keywords {
        match channel_names.iter().position(|name| name == key) {
            Some(index) => channel_args[index] = args.get(usize::MAX, key.as_str()),
            None => {
                return Err((
                    format!(
                        "${key}: Color space {space} doesn't have a channel with this name.",
                        key = key,
                        space = space.name(),
                    ),
                    span,
                )
                    .into())
            }
        }
    }

    let alpha = color.alpha().0;
    let old = color.channels_or_none();

    let result = match update {
        UpdateComponents::Change => {
            let mut new = [old[0], old[1], old[2]];
            for (index, arg) in channel_args.into_iter().enumerate() {
                if let Some(arg) = arg {
                    new[index] = Some(channel_value(
                        space,
                        channel_names[index],
                        arg.node,
                        arg.span,
                        visitor,
                    )?);
                }
            }

            let alpha = match alpha_arg {
                Some(arg) => {
                    let number = arg.node.assert_number_with_name("alpha", arg.span)?;
                    if number.unit == Unit::Percent {
                        number.assert_bounds("alpha", 0.0, 100.0, arg.span)?;
                    } else {
                        number.assert_bounds("alpha", 0.0, 1.0, arg.span)?;
                    }
                    alpha_value(&number)
                }
                None => alpha,
            };

            // `hwb()` scales whiteness and blackness that sum to more than
            // 100% when the color is (re)constructed.
            if let (ColorSpace::Hwb, Some(whiteness), Some(blackness)) = (space, new[1], new[2]) {
                if whiteness + blackness > 100.0 {
                    new[1] = Some(whiteness / (whiteness + blackness) * 100.0);
                    new[2] = Some(blackness / (whiteness + blackness) * 100.0);
                }
            }

            build(space, new, alpha)
        }
        UpdateComponents::Adjust => {
            let mut new = [old[0], old[1], old[2]];
            for (index, arg) in channel_args.into_iter().enumerate() {
                let arg = match arg {
                    Some(arg) => arg,
                    None => continue,
                };
                let name = channel_names[index];
                let current = match old[index] {
                    Some(current) => current,
                    None => return Err(missing_channel_error(&color, name, arg.span)),
                };
                let adjustment = channel_value(space, name, arg.node, arg.span, visitor)?;
                let result = current + adjustment;

                new[index] = Some(match (space, name) {
                    (ColorSpace::Rgb, _) => adjust_clamped(current, result, Some(0.0), Some(255.0)),
                    (ColorSpace::Hsl, "saturation") => {
                        adjust_clamped(current, result, Some(0.0), None)
                    }
                    _ => result,
                });
            }

            let alpha = match alpha_arg {
                Some(arg) => {
                    let number = arg.node.assert_number_with_name("alpha", arg.span)?;
                    clamp_like_css(alpha + alpha_value(&number), 0.0, 1.0)
                }
                None => alpha,
            };

            build(space, new, alpha)
        }
        UpdateComponents::Scale => {
            let mut new = [old[0], old[1], old[2]];
            for (index, arg) in channel_args.into_iter().enumerate() {
                let arg = match arg {
                    Some(arg) => arg,
                    None => continue,
                };
                let name = channel_names[index];
                if name == "hue" {
                    return Err(("$hue: Channel isn't scalable.", arg.span).into());
                }
                let current = match old[index] {
                    Some(current) => current,
                    None => return Err(missing_channel_error(&color, name, arg.span)),
                };
                let factor = scale_factor(arg.node, name, arg.span)?;
                new[index] = Some(scale_channel(current, factor, 0.0, space.channel_max()));
            }

            let alpha = match alpha_arg {
                Some(arg) => {
                    let factor = scale_factor(arg.node, "alpha", arg.span)?;
                    scale_channel(alpha, factor, 0.0, 1.0)
                }
                None => alpha,
            };

            build(space, new, alpha)
        }
    };

    Ok(Value::Color(Arc::new(
        result.to_space(original.space(), false),
    )))
}

/// Builds the updated color in `space`, keeping the hue missing when no
/// channel argument replaced it.
fn build(space: ColorSpace, channels: [Option<f64>; 3], alpha: f64) -> Color {
    let value = |channel: Option<f64>| Number(channel.unwrap_or(0.0));
    Color::in_space(
        space,
        value(channels[0]),
        value(channels[1]),
        value(channels[2]),
        Number(alpha),
    )
    .with_missing_hue(channels[0].is_none())
}

pub(crate) fn scale_color(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    update_components(args, visitor, UpdateComponents::Scale)
}

pub(crate) fn change_color(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    update_components(args, visitor, UpdateComponents::Change)
}

pub(crate) fn adjust_color(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    update_components(args, visitor, UpdateComponents::Adjust)
}

pub(crate) fn ie_hex_str(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;
    Ok(Value::String(color.to_ie_hex_str(), QuoteKind::None))
}

fn bool_value(value: bool) -> Value {
    if value {
        Value::True
    } else {
        Value::False
    }
}

pub(crate) fn declare(f: &mut GlobalFunctionMap) {
    f.insert("change-color", Builtin::new(change_color));
    f.insert("adjust-color", Builtin::new(adjust_color));
    f.insert("scale-color", Builtin::new(scale_color));
    f.insert("ie-hex-str", Builtin::new(ie_hex_str));
}
