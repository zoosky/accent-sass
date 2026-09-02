//! `rgb()`, `rgba()`, the legacy rgb channel getters, and `mix()`.

use crate::{
    builtin::builtin_imports::*,
    color::{clamp_like_css, ColorSpace, HueInterpolationMethod},
};

use super::{
    assert_common_list_style, assert_unquoted_string, color_to_string, dart_to_string,
    function_string, legacy_only_error,
    parse::{color_from_channels, parse_channels},
    percentage_or_unitless, space_from_value,
};

/// `rgb($color, $alpha)`: replaces the alpha of a legacy color (Dart
/// Sass's `_rgbTwoArg`).
fn inner_rgb_2_arg(
    name: &'static str,
    mut args: ArgumentResult,
    visitor: &mut Visitor,
) -> SassResult<Value> {
    let span = args.span();
    // rgba(var(--foo), 0.5) is valid CSS because --foo might be `123, 456, 789`
    // and functions are parsed after variable substitution.
    let first = args.get_err(0, "color")?;
    let second = args.get_err(1, "alpha")?;

    if first.is_var() || (!matches!(first, Value::Color(..)) && second.is_var()) {
        return function_string(name, &[first, second], visitor, span);
    }

    let color = first.assert_color_with_name("color", span)?;
    if !color.is_legacy() {
        return Err((
            format!(
                "${name}: Expected {color} to be in the legacy RGB, HSL, or HWB color space.\n\nRecommendation: color.change({color}, $alpha: {alpha})",
                name = name,
                color = color_to_string(&color, span)?,
                alpha = dart_to_string(&second, span)?,
            ),
            span,
        )
            .into());
    }

    let color = color.to_space(ColorSpace::Rgb, true);
    if second.is_special_function() {
        let channel =
            |index: usize| Value::Dimension(SassNumber::new_unitless(Number(color.channel(index))));
        return function_string(
            name,
            &[channel(0), channel(1), channel(2), second],
            visitor,
            span,
        );
    }

    let alpha = second.assert_number_with_name("alpha", span)?;
    Ok(Value::Color(Arc::new(color.change_alpha(clamp_like_css(
        percentage_or_unitless(&alpha, 1.0, "alpha", span)?,
        0.0,
        1.0,
    )))))
}

/// `rgb($red, $green, $blue, $alpha: 1)` (Dart Sass's `_rgb`).
fn inner_rgb_3_arg(
    name: &'static str,
    mut args: ArgumentResult,
    visitor: &mut Visitor,
) -> SassResult<Value> {
    let span = args.span();
    let has_alpha = args.len() > 3;
    let alpha = if has_alpha {
        args.get(3, "alpha").map(|alpha| alpha.node)
    } else {
        None
    };

    let red = args.get_err(0, "red")?;
    let green = args.get_err(1, "green")?;
    let blue = args.get_err(2, "blue")?;

    if red.is_special_function()
        || green.is_special_function()
        || blue.is_special_function()
        || alpha.as_ref().map_or(false, Value::is_special_function)
    {
        let mut values = vec![red, green, blue];
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
        ColorSpace::Rgb,
        Some(red.assert_number_with_name("red", span)?),
        Some(green.assert_number_with_name("green", span)?),
        Some(blue.assert_number_with_name("blue", span)?),
        alpha,
        true,
        true,
        span,
    )?)))
}

fn inner_rgb(
    name: &'static str,
    mut args: ArgumentResult,
    visitor: &mut Visitor,
) -> SassResult<Value> {
    args.max_args(4)?;

    match args.len() {
        0 | 1 => {
            let span = args.span();
            let channels = args.get_err(0, "channels")?;
            parse_channels(
                name,
                channels,
                Some(ColorSpace::Rgb),
                Some("channels"),
                visitor,
                span,
            )
        }
        2 => inner_rgb_2_arg(name, args, visitor),
        _ => inner_rgb_3_arg(name, args, visitor),
    }
}

pub(crate) fn rgb(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    inner_rgb("rgb", args, visitor)
}

pub(crate) fn rgba(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    inner_rgb("rgba", args, visitor)
}

/// The legacy channel getters (`red()`, `hue()`, `whiteness()`, ...): the
/// channel as seen through `space`, which only a legacy color has.
pub(crate) fn legacy_channel_function(
    mut args: ArgumentResult,
    name: &str,
    space: ColorSpace,
    index: usize,
    unit: Unit,
    round: bool,
) -> SassResult<Value> {
    args.max_args(1)?;
    let span = args.span();
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", span)?;

    if !color.is_legacy() {
        return Err(legacy_only_error(
            &format!("color.{}", name),
            "color.channel()",
            true,
            span,
        ));
    }

    let mut value = color.legacy_channel(space, index);
    if round {
        value = value.round();
    }

    Ok(Value::Dimension(SassNumber {
        num: Number(value),
        unit,
        as_slash: None,
    }))
}

pub(crate) fn red(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    legacy_channel_function(args, "red", ColorSpace::Rgb, 0, Unit::None, true)
}

pub(crate) fn green(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    legacy_channel_function(args, "green", ColorSpace::Rgb, 1, Unit::None, true)
}

pub(crate) fn blue(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    legacy_channel_function(args, "blue", ColorSpace::Rgb, 2, Unit::None, true)
}

/// Parses the `$method` of `color.mix()` (Dart Sass's
/// `InterpolationMethod.fromValue`): a color space, optionally followed by
/// a hue interpolation method and the word `hue` (`hsl longer hue`), as a
/// space-separated list.
pub(crate) fn parse_interpolation_method(
    method: Spanned<Value>,
) -> SassResult<(ColorSpace, HueInterpolationMethod)> {
    let span = method.span;
    let list = assert_common_list_style(&method.node, Some("method"), false, span)?;
    let inspected = dart_to_string(&method.node, span)?;

    let mut items = list.into_iter();
    let space = match items.next() {
        Some(space) => space_from_value(space, "method", span)?,
        None => {
            return Err((
                "$method: Expected a color interpolation method, got an empty list.",
                span,
            )
                .into())
        }
    };

    let hue_method = match items.next() {
        None => return Ok((space, HueInterpolationMethod::Shorter)),
        Some(hue_method) => {
            let name = assert_unquoted_string(hue_method, "method", span)?;
            match HueInterpolationMethod::from_name(&name) {
                Some(hue_method) => hue_method,
                None => {
                    return Err((
                        format!("$method: Unknown hue interpolation method {}.", name),
                        span,
                    )
                        .into())
                }
            }
        }
    };

    match items.next() {
        None => {
            return Err((
                format!(
                    "$method: Expected unquoted string \"hue\" after {}.",
                    inspected
                ),
                span,
            )
                .into())
        }
        Some(hue) => {
            let hue_inspected = dart_to_string(&hue, span)?;
            let hue = assert_unquoted_string(hue, "method", span)?;
            if !hue.eq_ignore_ascii_case("hue") {
                return Err((
                    format!(
                        "$method: Expected unquoted string \"hue\" at the end of {}, was {}.",
                        inspected, hue_inspected
                    ),
                    span,
                )
                    .into());
            }
        }
    }

    if items.next().is_some() {
        return Err((
            format!("$method: Expected nothing after \"hue\" in {}.", inspected),
            span,
        )
            .into());
    }

    if !space.is_polar() {
        return Err((
            format!(
                "$method: Hue interpolation method \"HueInterpolationMethod.{} hue\" may not be set for rectangular color space {}.",
                hue_method.name(),
                space.name()
            ),
            span,
        )
            .into());
    }

    Ok((space, hue_method))
}

/// `color.mix($color1, $color2, $weight: 50%, $method: null)`.
///
/// Without `$method` this is the legacy rgb mix, which only accepts legacy
/// colors and always produces an rgb-space color. With `$method` the
/// colors are interpolated in the named space per CSS Color 4 and the
/// result stays in `$color1`'s space.
pub(crate) fn mix(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(4)?;
    let span = args.span();
    let color1 = args
        .get_err(0, "color1")?
        .assert_color_with_name("color1", span)?;
    let color2 = args
        .get_err(1, "color2")?
        .assert_color_with_name("color2", span)?;

    let weight = args
        .default_arg(
            2,
            "weight",
            Value::Dimension(SassNumber {
                num: Number(50.0),
                unit: Unit::Percent,
                as_slash: None,
            }),
        )
        .assert_number_with_name("weight", span)?;

    if let Some(method) = args
        .get(3, "method")
        .filter(|method| method.node != Value::Null)
    {
        let (space, hue_method) = parse_interpolation_method(method)?;
        weight.assert_bounds_with_unit("weight", 0.0, 100.0, &Unit::Percent, span)?;
        return Ok(Value::Color(Arc::new(color1.interpolate(
            &color2,
            space,
            hue_method,
            weight.num.0 / 100.0,
            false,
        ))));
    }

    for (color, name) in [(&color1, "color1"), (&color2, "color2")] {
        if !color.is_legacy() {
            return Err((
                format!(
                    "${name}: To use color.mix() with non-legacy color {color}, you must provide a $method.",
                    name = name,
                    color = color_to_string(color, span)?,
                ),
                span,
            )
                .into());
        }
    }

    weight.assert_bounds("weight", 0.0, 100.0, span)?;
    Ok(Value::Color(Arc::new(
        color1.mix_legacy(&color2, weight.num.0 / 100.0),
    )))
}

pub(crate) fn declare(f: &mut GlobalFunctionMap) {
    f.insert("rgb", Builtin::new(rgb));
    f.insert("rgba", Builtin::new(rgba));
    f.insert("red", Builtin::new(red));
    f.insert("green", Builtin::new(green));
    f.insert("blue", Builtin::new(blue));
    f.insert("mix", Builtin::new(mix));
}
