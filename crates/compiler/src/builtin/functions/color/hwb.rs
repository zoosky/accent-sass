use crate::builtin::builtin_imports::*;

use super::{
    angle_value,
    rgb::{parse_channels, percentage_or_unitless},
    ParsedChannels,
};

pub(crate) fn blackness(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;

    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    Ok(Value::Dimension(SassNumber {
        num: color.blackness(),
        unit: Unit::Percent,
        as_slash: None,
    }))
}

pub(crate) fn whiteness(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;

    let color = args
        .get_err(0, "color")?
        .assert_color_with_name("color", args.span())?;

    Ok(Value::Dimension(SassNumber {
        num: color.whiteness(),
        unit: Unit::Percent,
        as_slash: None,
    }))
}

fn hwb_inner(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    let span = args.span();

    let hue = angle_value(args.get_err(0, "hue")?, "hue", args.span())?;

    // Dart Sass does not range-check whiteness or blackness: a negative
    // value yields an out-of-gamut color, and a sum above 100% is scaled.
    let whiteness = args
        .get_err(1, "whiteness")?
        .assert_number_with_name("whiteness", span)?;
    whiteness.assert_unit(&Unit::Percent, "whiteness", span)?;

    let blackness = args
        .get_err(2, "blackness")?
        .assert_number_with_name("blackness", span)?;
    blackness.assert_unit(&Unit::Percent, "blackness", span)?;

    let alpha = args
        .default_arg(3, "alpha", Value::Dimension(SassNumber::new_unitless(1.0)))
        .assert_number_with_name("alpha", args.span())?;

    let alpha = percentage_or_unitless(&alpha, 1.0, "alpha", args.span(), visitor)?;

    // Whiteness and blackness that sum to more than 100% are scaled down
    // proportionally when the color is constructed, as in Dart Sass's
    // `_colorFromChannels`.
    let (mut whiteness, mut blackness) = (whiteness.num.0, blackness.num.0);
    if whiteness + blackness > 100.0 {
        let sum = whiteness + blackness;
        whiteness = whiteness / sum * 100.0;
        blackness = blackness / sum * 100.0;
    }

    Ok(Value::Color(Arc::new(Color::from_hwb(
        hue,
        Number(whiteness),
        Number(blackness),
        Number(alpha),
    ))))
}

pub(crate) fn hwb(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(4)?;

    if args.len() == 0 || args.len() == 1 {
        match parse_channels(
            "hwb",
            &["hue", "whiteness", "blackness"],
            args.get_err(0, "channels")?,
            visitor,
            args.span(),
        )? {
            ParsedChannels::String(s) => Err((
                format!("Expected numeric channels, got \"{}\".", s),
                args.span(),
            )
                .into()),
            ParsedChannels::List(list) => {
                let args = ArgumentResult {
                    positional: list,
                    named: BTreeMap::new(),
                    separator: ListSeparator::Comma,
                    span: args.span(),
                    touched: BTreeSet::new(),
                };

                hwb_inner(args, visitor)
            }
        }
    } else if args.len() == 3 || args.len() == 4 {
        hwb_inner(args, visitor)
    } else {
        args.max_args(1)?;
        unreachable!()
    }
}

/// The global `hwb($channels)` function: the plain-CSS space-separated
/// syntax only. The comma-separated form is only available as
/// `color.hwb()`.
fn hwb_global(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    hwb(args, visitor)
}

pub(crate) fn declare(f: &mut GlobalFunctionMap) {
    f.insert("hwb", Builtin::new(hwb_global));
}
