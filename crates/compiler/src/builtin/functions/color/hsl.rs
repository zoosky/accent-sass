use std::collections::{BTreeMap, BTreeSet};

use crate::{builtin::builtin_imports::*, serializer::serialize_number, value::SassNumber};

use super::{
    angle_value,
    other::{missing_channel_error, parse_space},
    rgb::{function_string, parse_channels, percentage_or_unitless},
    ParsedChannels,
};

use crate::{
    color::{ColorSpace, HueInterpolationMethod},
    value::fuzzy_equals,
};

fn hsl_3_args(
    name: &'static str,
    mut args: ArgumentResult,
    visitor: &mut Visitor,
) -> SassResult<Value> {
    let span = args.span();

    let hue = args.get_err(0, "hue")?;
    let saturation = args.get_err(1, "saturation")?;
    let lightness = args.get_err(2, "lightness")?;
    let alpha = args.default_arg(3, "alpha", Value::Dimension(SassNumber::new_unitless(1.0)));

    if [&hue, &saturation, &lightness, &alpha]
        .iter()
        .copied()
        .any(Value::is_special_function)
    {
        return Ok(Value::String(
            format!(
                "{}({})",
                name,
                Value::List(
                    if args.len() == 4 {
                        vec![hue, saturation, lightness, alpha]
                    } else {
                        vec![hue, saturation, lightness]
                    },
                    ListSeparator::Comma,
                    Brackets::None
                )
                .to_css_string(args.span(), false)?
            ),
            QuoteKind::None,
        ));
    }

    let hue = angle_value(hue, "hue", span)?;
    let saturation = saturation.assert_number_with_name("saturation", span)?;
    let lightness = lightness.assert_number_with_name("lightness", span)?;
    let alpha = percentage_or_unitless(
        &alpha.assert_number_with_name("alpha", span)?,
        1.0,
        "alpha",
        span,
        visitor,
    )?;

    Ok(Value::Color(Arc::new(Color::from_hsla_fn(
        Number(hue.rem_euclid(360.0)),
        saturation.num,
        lightness.num,
        Number(alpha),
    ))))
}

fn inner_hsl(
    name: &'static str,
    mut args: ArgumentResult,
    visitor: &mut Visitor,
) -> SassResult<Value> {
    args.max_args(4)?;
    let span = args.span();

    let len = args.len();

    if len == 1 || len == 0 {
        match parse_channels(
            name,
            &["hue", "saturation", "lightness"],
            args.get_err(0, "channels")?,
            visitor,
            args.span(),
        )? {
            ParsedChannels::String(s) => Ok(Value::String(s, QuoteKind::None)),
            ParsedChannels::List(list) => {
                let args = ArgumentResult {
                    positional: list,
                    named: BTreeMap::new(),
                    separator: ListSeparator::Comma,
                    span: args.span(),
                    touched: BTreeSet::new(),
                };

                hsl_3_args(name, args, visitor)
            }
        }
    } else if len == 2 {
        let hue = args.get_err(0, "hue")?;
        let saturation = args.get_err(1, "saturation")?;

        if hue.is_var() || saturation.is_var() {
            Ok(Value::String(
                function_string(name, &[hue, saturation], visitor, span)?,
                QuoteKind::None,
            ))
        } else {
            Err(("Missing argument $lightness.", args.span()).into())
        }
    } else {
        hsl_3_args(name, args, visitor)
    }
}

pub(crate) fn hsl(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    inner_hsl("hsl", args, visitor)
}

pub(crate) fn hsla(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    inner_hsl("hsla", args, visitor)
}

pub(crate) fn hue(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    Ok(Value::Dimension(SassNumber {
        num: color.hue(),
        unit: Unit::Deg,
        as_slash: None,
    }))
}

pub(crate) fn saturation(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    Ok(Value::Dimension(SassNumber {
        num: color.saturation(),
        unit: Unit::Percent,
        as_slash: None,
    }))
}

pub(crate) fn lightness(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    Ok(Value::Dimension(SassNumber {
        num: color.lightness(),
        unit: Unit::Percent,
        as_slash: None,
    }))
}

pub(crate) fn adjust_hue(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;
    let degrees = angle_value(args.get_err(1, "degrees")?, "degrees", args.span())?;

    Ok(Value::Color(Arc::new(color.adjust_hue(degrees))))
}

fn lighten(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    let amount = args
        .get_err(1, "amount")?
        .assert_number_with_name("amount", args.span())?;

    amount.assert_bounds("amount", 0.0, 100.0, args.span())?;

    Ok(Value::Color(Arc::new(color.lighten(amount.num))))
}

fn darken(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    let amount = args
        .get_err(1, "amount")?
        .assert_number_with_name("amount", args.span())?;

    amount.assert_bounds("amount", 0.0, 100.0, args.span())?;

    Ok(Value::Color(Arc::new(color.darken(amount.num))))
}

fn saturate(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    if args.len() == 1 {
        let amount = args
            .get_err(0, "amount")?
            .assert_number_with_name("amount", args.span())?;

        return Ok(Value::String(
            format!(
                "saturate({})",
                serialize_number(&amount, &Options::default(), args.span())?,
            ),
            QuoteKind::None,
        ));
    }

    let amount = args
        .get_err(1, "amount")?
        .assert_number_with_name("amount", args.span())?;

    amount.assert_bounds("amount", 0.0, 100.0, args.span())?;

    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    Ok(Value::Color(Arc::new(color.saturate(amount.num))))
}

fn desaturate(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    let amount = args
        .get_err(1, "amount")?
        .assert_number_with_name("amount", args.span())?;

    amount.assert_bounds("amount", 0.0, 100.0, args.span())?;

    Ok(Value::Color(Arc::new(color.desaturate(amount.num))))
}

pub(crate) fn grayscale(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let color = match args.get_err(0, "color")? {
        Value::Color(c) => c,
        Value::Dimension(SassNumber {
            num: n,
            unit: u,
            as_slash: _,
        }) => {
            return Ok(Value::String(
                format!("grayscale({}{})", n.inspect(), u),
                QuoteKind::None,
            ))
        }
        v => {
            return Err((
                format!("$color: {} is not a color.", v.inspect(args.span())?),
                args.span(),
            )
                .into())
        }
    };
    Ok(Value::Color(Arc::new(color.grayscale())))
}

/// Reads the `$weight` of `invert()` as a fraction (`0..1`), requiring it
/// to be within `0%` and `100%`.
fn invert_weight(weight: Spanned<Value>) -> SassResult<Number> {
    let span = weight.span;
    let weight = weight.node.assert_number_with_name("weight", span)?;
    weight.assert_bounds("weight", 0.0, 100.0, span)?;
    Ok(weight.num / Number(100.0))
}

/// `color.complement($color, $space: null)`: rotates the hue 180 degrees in
/// `$space` (hsl by default). With an explicit `$space` the conversion
/// keeps a missing hue, which cannot be modified; without one, a
/// powerless hue reads as `0`.
pub(crate) fn complement(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;
    let span = args.span();
    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", span)?;

    let space = match args.get(1, "space") {
        Some(space) if space.node != Value::Null => {
            let space = parse_space(space.node, "space", space.span)?;
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
            Some(space.legacy("space", span)?)
        }
        _ => None,
    };

    let in_space = color.to_space(space.unwrap_or(ColorSpace::Hsl), space.is_some());
    if in_space.missing_hue() {
        return Err(missing_channel_error(&in_space, "hue", span));
    }

    Ok(Value::Color(Arc::new(
        in_space.rotate_hue(180.0).to_space(color.space(), false),
    )))
}

/// `color.invert($color, $weight: 100%, $space: null)`.
///
/// Without `$space` this is the legacy inversion: flip the rgb channels,
/// mix with the original by `$weight`, and convert back to the color's
/// space keeping a missing hue. With `$space` the color is inverted in
/// that space (a missing hue cannot be inverted) and, for a partial
/// weight, interpolated with the original there.
pub(crate) fn invert(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(3)?;
    let span = args.span();
    let weight = args.get(1, "weight");
    let space = args.get(2, "space");

    let color = match args.get_err(0, "color")? {
        Value::Color(c) => c,
        Value::Dimension(SassNumber {
            num: n,
            unit: u,
            as_slash: _,
        }) => {
            // The plain-CSS `invert()` filter function only takes one
            // argument; an explicit default weight of `100%` is allowed.
            if let Some(weight) = weight {
                let is_default_weight = match &weight.node {
                    Value::Dimension(number) => {
                        number.num == Number(100.0) && number.unit == Unit::Percent
                    }
                    _ => false,
                };
                if !is_default_weight {
                    return Err((
                        "Only one argument may be passed to the plain-CSS invert() function.",
                        span,
                    )
                        .into());
                }
            }
            return Ok(Value::String(
                format!("invert({}{})", n.inspect(), u),
                QuoteKind::None,
            ));
        }
        v => {
            return Err((
                format!("$color: {} is not a color.", v.inspect(span)?),
                span,
            )
                .into())
        }
    };

    let weight = match weight {
        Some(weight) => invert_weight(weight)?,
        None => Number::one(),
    };

    let space = match space {
        Some(space) if space.node != Value::Null => {
            Some(parse_space(space.node, "space", space.span)?.legacy("space", space.span)?)
        }
        _ => None,
    };

    let space = match space {
        Some(space) => space,
        None => return Ok(Value::Color(Arc::new(color.invert(weight)))),
    };

    if fuzzy_equals(weight.0, 0.0) {
        return Ok(Value::Color(color));
    }

    let in_space = color.to_space(space, true);
    if in_space.missing_hue() {
        return Err(missing_channel_error(&in_space, "hue", span));
    }
    let inverted = in_space.invert_channels();

    Ok(Value::Color(Arc::new(if fuzzy_equals(weight.0, 1.0) {
        inverted.to_space(color.space(), false)
    } else {
        color.interpolate(
            &inverted,
            space,
            HueInterpolationMethod::Shorter,
            1.0 - weight.0,
            false,
        )
    })))
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
