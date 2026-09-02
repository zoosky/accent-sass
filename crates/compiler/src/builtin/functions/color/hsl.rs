//! `hsl()`, `hsla()`, the legacy hsl functions (`lighten()`, `saturate()`,
//! `adjust-hue()`, ...), and `grayscale()`, `complement()`, and `invert()`.

use crate::{
    builtin::builtin_imports::*,
    color::{clamp_like_css, ChannelKind, ColorChannel, ColorSpace, HueInterpolationMethod},
    value::fuzzy_equals,
};

use super::{
    angle_value, color_to_string, function_string, legacy_only_error, missing_channel_error,
    parse::{color_from_channels, parse_channels},
    percentage_or_unitless,
    rgb::legacy_channel_function,
    space_from_value,
};

/// `hsl($hue, $saturation, $lightness, $alpha: 1)` (Dart Sass's `_hsl`).
fn hsl_3_args(
    name: &'static str,
    mut args: ArgumentResult,
    visitor: &mut Visitor,
) -> SassResult<Value> {
    let span = args.span();
    let has_alpha = args.len() > 3;

    let hue = args.get_err(0, "hue")?;
    let saturation = args.get_err(1, "saturation")?;
    let lightness = args.get_err(2, "lightness")?;
    let alpha = if has_alpha {
        args.get(3, "alpha").map(|alpha| alpha.node)
    } else {
        None
    };

    if hue.is_special_function()
        || saturation.is_special_function()
        || lightness.is_special_function()
        || alpha.as_ref().map_or(false, Value::is_special_function)
    {
        let mut values = vec![hue, saturation, lightness];
        values.extend(alpha);
        return function_string(name, &values, visitor, span);
    }

    let alpha = match alpha {
        Some(alpha) => Some(clamp_like_css(
            percentage_or_unitless(
                &alpha.assert_number_with_name("alpha", span)?,
                1.0,
                "alpha",
                span,
            )?,
            0.0,
            1.0,
        )),
        None => Some(1.0),
    };

    Ok(Value::Color(Arc::new(color_from_channels(
        ColorSpace::Hsl,
        Some(hue.assert_number_with_name("hue", span)?),
        Some(saturation.assert_number_with_name("saturation", span)?),
        Some(lightness.assert_number_with_name("lightness", span)?),
        alpha,
        true,
        false,
        span,
    )?)))
}

fn inner_hsl(
    name: &'static str,
    mut args: ArgumentResult,
    visitor: &mut Visitor,
) -> SassResult<Value> {
    args.max_args(4)?;
    let span = args.span();

    match args.len() {
        0 | 1 => {
            let channels = args.get_err(0, "channels")?;
            parse_channels(
                name,
                channels,
                Some(ColorSpace::Hsl),
                Some("channels"),
                visitor,
                span,
            )
        }
        2 => {
            let hue = args.get_err(0, "hue")?;
            let saturation = args.get_err(1, "saturation")?;

            if hue.is_var() || saturation.is_var() {
                function_string(name, &[hue, saturation], visitor, span)
            } else {
                Err(("Missing argument $lightness.", span).into())
            }
        }
        _ => hsl_3_args(name, args, visitor),
    }
}

pub(crate) fn hsl(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    inner_hsl("hsl", args, visitor)
}

pub(crate) fn hsla(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    inner_hsl("hsla", args, visitor)
}

pub(crate) fn hue(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    legacy_channel_function(args, "hue", ColorSpace::Hsl, 0, Unit::Deg, false)
}

pub(crate) fn saturation(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    legacy_channel_function(args, "saturation", ColorSpace::Hsl, 1, Unit::Percent, false)
}

pub(crate) fn lightness(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    legacy_channel_function(args, "lightness", ColorSpace::Hsl, 2, Unit::Percent, false)
}

/// Dart Sass's `changeHsl`: rebuilds a legacy color from its hsl view with
/// one channel replaced, then converts back to its own space.
fn change_hsl(color: &Color, index: usize, value: f64) -> Color {
    let hsl = color.to_space(ColorSpace::Hsl, true);
    let mut channels = hsl.channels();
    channels[index] = value;

    Color::for_space(
        ColorSpace::Hsl,
        Some(channels[0]),
        Some(channels[1]),
        Some(channels[2]),
        Some(color.alpha()),
    )
    .to_space(color.space(), true)
}

pub(crate) fn adjust_hue(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let span = args.span();
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", span)?;
    let degrees = angle_value(args.get_err(1, "degrees")?, "degrees", span)?;

    if !color.is_legacy() {
        return Err(legacy_only_error(
            "adjust-hue",
            "color.adjust()",
            true,
            span,
        ));
    }

    let hue = color.legacy_channel(ColorSpace::Hsl, 0);
    Ok(Value::Color(Arc::new(change_hsl(&color, 0, hue + degrees))))
}

/// The shared body of `lighten()`, `darken()`, `saturate()`, and
/// `desaturate()`: adds `sign * $amount` to the hsl channel at `index`,
/// clamped to `0..100`.
fn adjust_legacy_channel(
    mut args: ArgumentResult,
    name: &str,
    index: usize,
    sign: f64,
) -> SassResult<Value> {
    args.max_args(2)?;
    let span = args.span();
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", span)?;
    let amount = args
        .get_err(1, "amount")?
        .assert_number_with_name("amount", span)?;

    if !color.is_legacy() {
        return Err(legacy_only_error(name, "color.adjust()", true, span));
    }

    amount.assert_bounds("amount", 0.0, 100.0, span)?;
    let current = color.legacy_channel(ColorSpace::Hsl, index);
    Ok(Value::Color(Arc::new(change_hsl(
        &color,
        index,
        clamp_like_css(current + sign * amount.num.0, 0.0, 100.0),
    ))))
}

fn lighten(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    adjust_legacy_channel(args, "lighten", 2, 1.0)
}

fn darken(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    adjust_legacy_channel(args, "darken", 2, -1.0)
}

/// `saturate($color, $amount)`, or the plain-CSS `saturate($amount)` filter
/// function when called with one argument.
fn saturate(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    if args.len() <= 1 {
        let span = args.span();
        let amount = args.get_err(0, "amount")?;
        if !amount.is_special_function() {
            amount.clone().assert_number_with_name("amount", span)?;
        }
        return function_string("saturate", &[amount], visitor, span);
    }

    adjust_legacy_channel(args, "saturate", 1, 1.0)
}

fn desaturate(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    adjust_legacy_channel(args, "desaturate", 1, -1.0)
}

/// `grayscale($color)`: drops the saturation of a legacy color in hsl, or
/// the chroma of any other color in oklch, and converts back. The global
/// function also passes a number, or a value only CSS can evaluate,
/// through as the plain-CSS filter function; `color.grayscale()` only
/// does so for a number.
fn grayscale_inner(
    mut args: ArgumentResult,
    visitor: &mut Visitor,
    global: bool,
) -> SassResult<Value> {
    args.max_args(1)?;
    let span = args.span();
    let color = match args.get_err(0, "color")? {
        Value::Color(c) => c,
        value
            if matches!(value, Value::Dimension(..)) || (global && value.is_special_function()) =>
        {
            return function_string("grayscale", &[value], visitor, span);
        }
        v => {
            return Err((
                format!("$color: {} is not a color.", v.inspect(span)?),
                span,
            )
                .into())
        }
    };

    Ok(Value::Color(Arc::new(if color.is_legacy() {
        let hsl = color.to_space(ColorSpace::Hsl, true);
        let [hue, _, lightness] = hsl.channels_or_none();
        Color::for_space(
            ColorSpace::Hsl,
            hue,
            Some(0.0),
            lightness,
            Some(hsl.alpha()),
        )
        .to_space(color.space(), false)
    } else {
        let oklch = color.to_space(ColorSpace::Oklch, true);
        let [lightness, _, hue] = oklch.channels_or_none();
        Color::for_space(
            ColorSpace::Oklch,
            lightness,
            Some(0.0),
            hue,
            Some(oklch.alpha()),
        )
        .to_space(color.space(), true)
    })))
}

pub(crate) fn grayscale(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    grayscale_inner(args, visitor, true)
}

/// `color.grayscale()` from the `sass:color` module.
pub(crate) fn module_grayscale(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    grayscale_inner(args, visitor, false)
}

/// Dart Sass's `_invertChannel`: flips a channel within its range. A
/// signed channel negates, a `0..max` channel reflects, and a hue rotates
/// 180 degrees. The channel must not be missing.
fn invert_channel(
    color: &Color,
    channel: &ColorChannel,
    value: Option<f64>,
    span: Span,
) -> SassResult<f64> {
    let value = match value {
        Some(value) => value,
        None => return Err(missing_channel_error(color, channel.name, span)),
    };

    Ok(match channel.kind {
        ChannelKind::Linear { min, .. } if min < 0.0 => -value,
        ChannelKind::Linear { max, .. } => max - value,
        ChannelKind::PolarAngle => (value + 180.0).rem_euclid(360.0),
    })
}

/// `color.complement($color, $space: null)`: rotates the hue 180 degrees
/// in `$space` (hsl for a legacy color without a `$space`). With an
/// explicit `$space` the conversion keeps a missing hue, which cannot be
/// modified; without one, a powerless hue reads as `0`.
pub(crate) fn complement(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let span = args.span();
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", span)?;
    let space_arg = args.get(1, "space").map(|space| space.node);
    let has_space = space_arg
        .as_ref()
        .map_or(false, |space| *space != Value::Null);

    let space = if color.is_legacy() && !has_space {
        ColorSpace::Hsl
    } else {
        space_from_value(space_arg.unwrap_or(Value::Null), "space", span)?
    };

    if !space.is_polar() {
        return Err((
            format!(
                "$space: Color space {} doesn't have a hue channel.",
                space.name()
            ),
            span,
        )
            .into());
    }

    let in_space = color.to_space(space, has_space);
    let channels = space.channels();
    let [channel0, channel1, channel2] = in_space.channels_or_none();
    let rotate = |index: usize, value: Option<f64>| match value {
        Some(value) => Ok(Some(value + 180.0)),
        None => Err(missing_channel_error(&in_space, channels[index].name, span)),
    };

    let result = if space.is_legacy() {
        Color::for_space(
            space,
            rotate(0, channel0)?,
            channel1,
            channel2,
            in_space.alpha_or_none(),
        )
    } else {
        Color::for_space(
            space,
            channel0,
            channel1,
            rotate(2, channel2)?,
            in_space.alpha_or_none(),
        )
    };

    Ok(Value::Color(Arc::new(
        result.to_space(color.space(), false),
    )))
}

/// `color.invert($color, $weight: 100%, $space: null)`.
///
/// Without `$space` this is the legacy inversion: flip the rgb channels,
/// mix with the original by `$weight`, and convert back to the color's
/// space. With `$space` the color is inverted in that space (a missing
/// channel cannot be inverted) and, for a partial weight, interpolated
/// with the original there.
fn invert_inner(
    mut args: ArgumentResult,
    visitor: &mut Visitor,
    global: bool,
) -> SassResult<Value> {
    args.max_args(3)?;
    let span = args.span();
    let color = args.get_err(0, "color")?;
    let weight = args
        .default_arg(
            1,
            "weight",
            Value::Dimension(SassNumber {
                num: Number(100.0),
                unit: Unit::Percent,
                as_slash: None,
            }),
        )
        .assert_number_with_name("weight", span)?;
    let space = args.get(2, "space").map(|space| space.node);

    if matches!(color, Value::Dimension(..)) || (global && color.is_special_function()) {
        // The plain-CSS `invert()` filter function only takes one
        // argument; an explicit default weight of `100%` is allowed.
        if weight.num.0 != 100.0 || weight.unit != Unit::Percent {
            return Err((
                "Only one argument may be passed to the plain-CSS invert() function.",
                span,
            )
                .into());
        }
        return function_string("invert", &[color], visitor, span);
    }

    let color = color.assert_color_with_name("color", span)?;

    let space = match space {
        Some(space) if space != Value::Null => space_from_value(space, "space", span)?,
        _ => {
            if !color.is_legacy() {
                return Err((
                    format!(
                        "$color: To use color.invert() with non-legacy color {}, you must provide a $space.",
                        color_to_string(&color, span)?
                    ),
                    span,
                )
                    .into());
            }

            let rgb = color.to_space(ColorSpace::Rgb, true);
            let channels = ColorSpace::Rgb.channels();
            let [red, green, blue] = rgb.channels_or_none();
            let inverse = Color::for_space(
                ColorSpace::Rgb,
                Some(invert_channel(&rgb, &channels[0], red, span)?),
                Some(invert_channel(&rgb, &channels[1], green, span)?),
                Some(invert_channel(&rgb, &channels[2], blue, span)?),
                color.alpha_or_none(),
            );

            weight.assert_bounds("weight", 0.0, 100.0, span)?;
            return Ok(Value::Color(Arc::new(
                inverse
                    .mix_legacy(&color, weight.num.0 / 100.0)
                    .to_space(color.space(), true),
            )));
        }
    };

    weight.assert_bounds_with_unit("weight", 0.0, 100.0, &Unit::Percent, span)?;
    let weight = weight.num.0 / 100.0;
    if fuzzy_equals(weight, 0.0) {
        return Ok(Value::Color(color));
    }

    let in_space = color.to_space(space, true);
    let channels = space.channels();
    let [channel0, channel1, channel2] = in_space.channels_or_none();
    let flip = |index: usize, value: Option<f64>| {
        invert_channel(&in_space, &channels[index], value, span).map(Some)
    };

    let inverted = match space {
        ColorSpace::Hwb => Color::for_space(
            space,
            flip(0, channel0)?,
            channel2,
            channel1,
            Some(in_space.alpha()),
        ),
        ColorSpace::Hsl | ColorSpace::Lch | ColorSpace::Oklch => Color::for_space(
            space,
            flip(0, channel0)?,
            channel1,
            flip(2, channel2)?,
            Some(in_space.alpha()),
        ),
        _ => Color::for_space(
            space,
            flip(0, channel0)?,
            flip(1, channel1)?,
            flip(2, channel2)?,
            Some(in_space.alpha()),
        ),
    };

    Ok(Value::Color(Arc::new(if fuzzy_equals(weight, 1.0) {
        inverted.to_space(color.space(), false)
    } else {
        color.interpolate(
            &inverted,
            space,
            HueInterpolationMethod::Shorter,
            1.0 - weight,
            false,
        )
    })))
}

pub(crate) fn invert(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    invert_inner(args, visitor, true)
}

/// `color.invert()` from the `sass:color` module, which only passes a
/// number through as the plain-CSS filter function.
pub(crate) fn module_invert(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    invert_inner(args, visitor, false)
}

pub(crate) fn declare(f: &mut GlobalFunctionMap) {
    f.insert("hsl", Builtin::new(hsl));
    f.insert("hsla", Builtin::new(hsla));
    f.insert("hue", Builtin::new(hue));
    f.insert("saturation", Builtin::new(saturation));
    f.insert("adjust-hue", Builtin::new(adjust_hue));
    f.insert("lightness", Builtin::new(lightness));
    f.insert("lighten", Builtin::new(lighten));
    f.insert("darken", Builtin::new(darken));
    f.insert("saturate", Builtin::new(saturate));
    f.insert("desaturate", Builtin::new(desaturate));
    f.insert("grayscale", Builtin::new(grayscale));
    f.insert("complement", Builtin::new(complement));
    f.insert("invert", Builtin::new(invert));
}
