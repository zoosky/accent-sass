//! The `sass:color` functions that operate on a color's channels in a
//! chosen space: `channel`, `adjust`, `change`, `scale`, and the
//! color-space introspection family (`space`, `to-space`, `is-legacy`,
//! `is-missing`, `is-powerless`, `is-in-gamut`, `to-gamut`, `same`).

use crate::{
    builtin::builtin_imports::*,
    color::{clamp_like_css, ChannelKind, ColorChannel, ColorSpace, GamutMapMethod},
    value::{fuzzy_equals, fuzzy_round},
};

use super::{
    angle_value, assert_unquoted_string, color_in_space, color_to_string, dart_to_string, is_none,
    missing_channel_error,
    parse::{channel_from_value, color_from_channels},
    space_from_value,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum UpdateComponents {
    Change,
    Adjust,
    Scale,
}

/// Reads a `$channel` argument, which must be a quoted string (Dart Sass's
/// `_channelName`).
fn channel_name(value: Value, span: Span) -> SassResult<String> {
    let (name, quotes) = value.assert_string_with_name("channel", span)?;
    if quotes == QuoteKind::None {
        return Err((
            format!("$channel: Expected {} to be a quoted string.", name),
            span,
        )
            .into());
    }
    Ok(name)
}

/// The index of the channel called `name` in `color`'s space, with `3`
/// standing for alpha, or Dart Sass's "doesn't have a channel named"
/// error.
fn channel_index(color: &Color, name: &str, span: Span) -> SassResult<usize> {
    if name == "alpha" {
        return Ok(3);
    }
    match color.space().channel_index(name) {
        Some(index) => Ok(index),
        None => Err((
            format!(
                "$channel: Color {} doesn't have a channel named \"{}\".",
                color_to_string(color, span)?,
                name
            ),
            span,
        )
            .into()),
    }
}

/// `color.channel($color, $channel, $space: null)`: a channel of the
/// color as seen in `$space`, in the channel's conventional unit.
pub(crate) fn channel(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(3)?;
    let span = args.span();
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", span)?;
    let channel = args.get_err(1, "channel")?;
    let color = color_in_space(&color, args.get(2, "space"), true)?;
    let name = channel_name(channel, span)?;

    if name == "alpha" {
        return Ok(Value::Dimension(SassNumber::new_unitless(Number(
            color.alpha(),
        ))));
    }

    let index = match color.space().channel_index(&name) {
        Some(index) => index,
        None => {
            return Err((
                format!(
                    "$channel: Color {} has no channel named {}.",
                    color_to_string(&color, span)?,
                    name
                ),
                span,
            )
                .into())
        }
    };

    let info = color.space().channels()[index];
    let unit = info.associated_unit();
    let mut value = color.channel(index);
    if unit == Unit::Percent {
        if let ChannelKind::Linear { max, .. } = info.kind {
            value = value * 100.0 / max;
        }
    }

    Ok(Value::Dimension(SassNumber {
        num: Number(value),
        unit,
        as_slash: None,
    }))
}

/// `color.space($color)`: the name of the color's space.
pub(crate) fn space(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    Ok(Value::String(
        color.space().name().to_owned(),
        QuoteKind::None,
    ))
}

/// `color.to-space($color, $space)`. A legacy result never carries a
/// missing channel, so an achromatic legacy result gets a hue of `0`.
pub(crate) fn to_space(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let span = args.span();
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", span)?;
    let space = match args.get(1, "space") {
        Some(space) => space,
        None => return Err(("Missing argument $space.", span).into()),
    };

    Ok(Value::Color(Arc::new(color_in_space(
        &color,
        Some(space),
        false,
    )?)))
}

/// `color.is-legacy($color)`: whether the color is in rgb, hsl, or hwb.
pub(crate) fn is_legacy(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    Ok(bool_value(color.is_legacy()))
}

/// `color.is-missing($color, $channel)`: whether the channel is `none`.
pub(crate) fn is_missing(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let span = args.span();
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", span)?;
    let name = channel_name(args.get_err(1, "channel")?, span)?;

    let missing = match channel_index(&color, &name, span)? {
        3 => color.is_alpha_missing(),
        index => color.is_channel_missing(index),
    };
    Ok(bool_value(missing))
}

/// `color.is-powerless($color, $channel, $space: null)`: whether the
/// channel has no effect on the color (see
/// [`Color::is_channel_powerless`]).
pub(crate) fn is_powerless(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(3)?;
    let span = args.span();
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", span)?;
    let channel = args.get_err(1, "channel")?;
    let color = color_in_space(&color, args.get(2, "space"), true)?;
    let name = channel_name(channel, span)?;

    let powerless = match channel_index(&color, &name, span)? {
        3 => false,
        index => color.is_channel_powerless(index),
    };
    Ok(bool_value(powerless))
}

/// `color.is-in-gamut($color, $space: null)`.
pub(crate) fn is_in_gamut(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let span = args.span();
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", span)?;
    let color = color_in_space(&color, args.get(1, "space"), true)?;

    Ok(bool_value(color.is_in_gamut()))
}

/// `color.to-gamut($color, $space: null, $method: null)`: maps the color
/// into the gamut of `$space` (its own space by default) and converts it
/// back. An unbounded space holds every color, so the color is returned
/// unchanged for it.
pub(crate) fn to_gamut(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(3)?;
    let span = args.span();
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", span)?;

    let space = match args.get(1, "space") {
        Some(space) if space.node != Value::Null => {
            space_from_value(space.node, "space", space.span)?
        }
        _ => color.space(),
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

    if !space.is_bounded() {
        return Ok(Value::Color(color));
    }

    Ok(Value::Color(Arc::new(
        color
            .to_space(space, true)
            .to_gamut(method)
            .to_space(color.space(), false),
    )))
}

/// `color.same($color1, $color2)`: whether two colors are the same color,
/// comparing channel by channel in the same space and through xyz
/// otherwise, with missing channels reading as `0`.
pub(crate) fn same(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let span = args.span();
    let color1 = args
        .get_err(0, "color1")?
        .assert_color_with_name("color1", span)?;
    let color2 = args
        .get_err(1, "color2")?
        .assert_color_with_name("color2", span)?;

    let to_xyz_no_missing = |color: &Color| {
        if color.space() == ColorSpace::XyzD65 && !color.has_missing_channel() {
            return color.clone();
        }
        let [x, y, z] = color.channels();
        if color.space() == ColorSpace::XyzD65 {
            Color::for_space(
                ColorSpace::XyzD65,
                Some(x),
                Some(y),
                Some(z),
                Some(color.alpha()),
            )
        } else {
            color.space().convert(
                ColorSpace::XyzD65,
                Some(x),
                Some(y),
                Some(z),
                Some(color.alpha()),
            )
        }
    };

    let same = if color1.space() == color2.space() {
        let channels1 = color1.channels();
        let channels2 = color2.channels();
        channels1
            .iter()
            .zip(channels2.iter())
            .all(|(a, b)| fuzzy_equals(*a, *b))
            && fuzzy_equals(color1.alpha(), color2.alpha())
    } else {
        to_xyz_no_missing(&color1) == to_xyz_no_missing(&color2)
    };

    Ok(bool_value(same))
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

/// Dart Sass's `_channelForChange`: the number to build the changed color
/// from, which is the argument when one was given (or `None` for the
/// keyword `none`) and otherwise the current channel in its conventional
/// unit.
fn channel_for_change(
    arg: Option<&Spanned<Value>>,
    color: &Color,
    index: usize,
    span: Span,
) -> SassResult<Option<SassNumber>> {
    let arg = match arg {
        None => {
            return Ok(color.channels_or_none()[index].map(|value| SassNumber {
                num: Number(value),
                unit: if color.space().is_legacy() && color.space().is_polar() && index > 0 {
                    Unit::Percent
                } else {
                    Unit::None
                },
                as_slash: None,
            }))
        }
        Some(arg) => arg,
    };

    if is_none(&arg.node) {
        return Ok(None);
    }
    match &arg.node {
        Value::Dimension(number) => Ok(Some(number.clone())),
        value => Err((
            format!(
                "${}: {} is not a number or unquoted \"none\".",
                color.space().channels()[index].name,
                dart_to_string(value, span)?
            ),
            arg.span,
        )
            .into()),
    }
}

/// Dart Sass's `_changeColor`.
fn change_color_channels(
    color: &Color,
    channel_args: [Option<Spanned<Value>>; 3],
    alpha_arg: Option<Spanned<Value>>,
    span: Span,
) -> SassResult<Color> {
    let alpha = match alpha_arg {
        None => Some(color.alpha()),
        Some(arg) if is_none(&arg.node) => None,
        Some(arg) => match &arg.node {
            Value::Dimension(number) if number.unit == Unit::None => {
                number.assert_bounds("alpha", 0.0, 1.0, arg.span)?;
                Some(number.num.0)
            }
            Value::Dimension(number) if number.unit == Unit::Percent => {
                number.assert_bounds_with_unit("alpha", 0.0, 100.0, &Unit::Percent, arg.span)?;
                Some(number.num.0 / 100.0)
            }
            Value::Dimension(number) => {
                // Dart Sass deprecates other units and takes the bare value.
                number.assert_bounds("alpha", 0.0, 1.0, arg.span)?;
                Some(number.num.0)
            }
            value => {
                return Err((
                    format!(
                        "$alpha: {} is not a number or unquoted \"none\".",
                        dart_to_string(value, span)?
                    ),
                    arg.span,
                )
                    .into())
            }
        },
    };

    color_from_channels(
        color.space(),
        channel_for_change(channel_args[0].as_ref(), color, 0, span)?,
        channel_for_change(channel_args[1].as_ref(), color, 1, span)?,
        channel_for_change(channel_args[2].as_ref(), color, 2, span)?,
        alpha,
        false,
        false,
        span,
    )
}

/// Dart Sass's `_scaleChannel`: moves a channel toward its bound by a
/// percentage factor, never past a value that is already out of range.
fn scale_channel(
    color: &Color,
    channel: &ColorChannel,
    old_value: Option<f64>,
    factor_arg: Option<&Spanned<SassNumber>>,
) -> SassResult<Option<f64>> {
    let factor_arg = match factor_arg {
        Some(factor_arg) => factor_arg,
        None => return Ok(old_value),
    };

    let (min, max) = match channel.kind {
        ChannelKind::Linear { min, max, .. } => (min, max),
        ChannelKind::PolarAngle => {
            return Err((
                format!("${}: Channel isn't scalable.", channel.name),
                factor_arg.span,
            )
                .into())
        }
    };

    let old_value = match old_value {
        Some(old_value) => old_value,
        None => return Err(missing_channel_error(color, channel.name, factor_arg.span)),
    };

    let number = &factor_arg.node;
    number.assert_unit(&Unit::Percent, channel.name, factor_arg.span)?;
    number.assert_bounds_with_unit(channel.name, -100.0, 100.0, &Unit::Percent, factor_arg.span)?;
    let factor = number.num.0 / 100.0;

    Ok(Some(if factor == 0.0 {
        old_value
    } else if factor > 0.0 {
        if old_value >= max {
            old_value
        } else {
            old_value + (max - old_value) * factor
        }
    } else if old_value <= min {
        old_value
    } else {
        old_value + (old_value - min) * factor
    }))
}

/// Dart Sass's `_adjustChannel`: adds an adjustment to a channel. A result
/// that crosses a clamped bound is clamped to it, unless the old value was
/// already past that bound, in which case the adjustment may only move it
/// toward the bound.
fn adjust_channel(
    color: &Color,
    channel: &ColorChannel,
    old_value: Option<f64>,
    adjustment_arg: Option<&Spanned<SassNumber>>,
) -> SassResult<Option<f64>> {
    let adjustment_arg = match adjustment_arg {
        Some(adjustment_arg) => adjustment_arg,
        None => return Ok(old_value),
    };
    let old_value = match old_value {
        Some(old_value) => old_value,
        None => {
            return Err(missing_channel_error(
                color,
                channel.name,
                adjustment_arg.span,
            ))
        }
    };

    let span = adjustment_arg.span;
    let mut adjustment = adjustment_arg.node.clone();
    let legacy_polar = matches!(color.space(), ColorSpace::Hsl | ColorSpace::Hwb);
    if legacy_polar && channel.is_polar_angle() {
        adjustment = SassNumber::new_unitless(Number(angle_value(
            Value::Dimension(adjustment),
            "hue",
            span,
        )?));
    } else if color.space() == ColorSpace::Hsl
        && (channel.name == "saturation" || channel.name == "lightness")
    {
        // A unitless saturation or lightness is deprecated but accepted as
        // a percentage.
        adjustment = SassNumber {
            num: adjustment.num,
            unit: Unit::Percent,
            as_slash: None,
        };
    } else if channel.name == "alpha" && adjustment.unit != Unit::None {
        // A unit on an alpha adjustment is deprecated; the bare value is
        // used.
        adjustment = SassNumber::new_unitless(adjustment.num);
    }

    let result =
        old_value + channel_from_value(channel, Some(&adjustment), false, span)?.unwrap_or(0.0);

    Ok(Some(match channel.kind {
        ChannelKind::Linear {
            lower_clamped: true,
            min,
            ..
        } if result < min => {
            if old_value < min {
                old_value.max(result)
            } else {
                min
            }
        }
        ChannelKind::Linear {
            upper_clamped: true,
            max,
            ..
        } if result > max => {
            if old_value > max {
                old_value.min(result)
            } else {
                max
            }
        }
        _ => result,
    }))
}

/// Dart Sass's `_updateComponents`, the shared body of `color.change()`,
/// `color.adjust()`, and `color.scale()`.
fn update_components(mut args: ArgumentResult, update: UpdateComponents) -> SassResult<Value> {
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

    let space_keyword = args.get(usize::MAX, "space");
    let alpha_arg = args.get(usize::MAX, "alpha");

    let keywords: Vec<String> = args
        .named
        .keys()
        .map(|key| key.as_str().to_owned())
        .collect();

    let color = if space_keyword.is_none() && original.is_legacy() && !keywords.is_empty() {
        match sniff_legacy_color_space(&keywords) {
            Some(space) => original.to_space(space, false),
            None => (*original).clone(),
        }
    } else {
        match space_keyword {
            Some(space) => {
                original.to_space(space_from_value(space.node, "space", space.span)?, true)
            }
            None => (*original).clone(),
        }
    };

    let channel_infos = color.space().channels();
    let mut channel_args: [Option<Spanned<Value>>; 3] = [None, None, None];
    for key in &keywords {
        match channel_infos.iter().position(|info| info.name == key) {
            Some(index) => channel_args[index] = args.get(usize::MAX, key.as_str()),
            None => {
                return Err((
                    format!(
                        "${key}: Color space {space} doesn't have a channel with this name.",
                        key = key,
                        space = color.space().name(),
                    ),
                    span,
                )
                    .into())
            }
        }
    }

    let result = if update == UpdateComponents::Change {
        change_color_channels(&color, channel_args, alpha_arg, span)?
    } else {
        let mut channel_numbers: [Option<Spanned<SassNumber>>; 3] = [None, None, None];
        for (index, arg) in channel_args.into_iter().enumerate() {
            if let Some(arg) = arg {
                channel_numbers[index] = Some(Spanned {
                    node: arg
                        .node
                        .assert_number_with_name(channel_infos[index].name, arg.span)?,
                    span: arg.span,
                });
            }
        }
        let alpha_number = match alpha_arg {
            Some(arg) => Some(Spanned {
                node: arg.node.assert_number_with_name("alpha", arg.span)?,
                span: arg.span,
            }),
            None => None,
        };

        let old = color.channels_or_none();
        if update == UpdateComponents::Scale {
            Color::for_space(
                color.space(),
                scale_channel(
                    &color,
                    &channel_infos[0],
                    old[0],
                    channel_numbers[0].as_ref(),
                )?,
                scale_channel(
                    &color,
                    &channel_infos[1],
                    old[1],
                    channel_numbers[1].as_ref(),
                )?,
                scale_channel(
                    &color,
                    &channel_infos[2],
                    old[2],
                    channel_numbers[2].as_ref(),
                )?,
                scale_channel(
                    &color,
                    &ColorChannel::ALPHA,
                    color.alpha_or_none(),
                    alpha_number.as_ref(),
                )?,
            )
        } else {
            Color::for_space(
                color.space(),
                adjust_channel(
                    &color,
                    &channel_infos[0],
                    old[0],
                    channel_numbers[0].as_ref(),
                )?,
                adjust_channel(
                    &color,
                    &channel_infos[1],
                    old[1],
                    channel_numbers[1].as_ref(),
                )?,
                adjust_channel(
                    &color,
                    &channel_infos[2],
                    old[2],
                    channel_numbers[2].as_ref(),
                )?,
                adjust_channel(
                    &color,
                    &ColorChannel::ALPHA,
                    color.alpha_or_none(),
                    alpha_number.as_ref(),
                )?
                .map(|alpha| clamp_like_css(alpha, 0.0, 1.0)),
            )
        }
    };

    Ok(Value::Color(Arc::new(
        result.to_space(original.space(), false),
    )))
}

pub(crate) fn scale_color(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    update_components(args, UpdateComponents::Scale)
}

pub(crate) fn change_color(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    update_components(args, UpdateComponents::Change)
}

pub(crate) fn adjust_color(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    update_components(args, UpdateComponents::Adjust)
}

/// `color.ie-hex-str($color)`: the `#AARRGGBB` form of the color mapped
/// into the rgb gamut.
pub(crate) fn ie_hex_str(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?
        .to_space(ColorSpace::Rgb, true)
        .to_gamut(GamutMapMethod::LocalMinde);

    let hex = |component: f64| format!("{:02X}", fuzzy_round(component) as u8);

    Ok(Value::String(
        format!(
            "#{}{}{}{}",
            hex(color.alpha() * 255.0),
            hex(color.channel0()),
            hex(color.channel1()),
            hex(color.channel2())
        ),
        QuoteKind::None,
    ))
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
