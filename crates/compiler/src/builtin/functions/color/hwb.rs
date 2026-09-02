//! `hwb()` and the legacy hwb channel getters.

use crate::{builtin::builtin_imports::*, color::ColorSpace};

use super::{parse::parse_channels, rgb::legacy_channel_function};

pub(crate) fn blackness(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    legacy_channel_function(args, "blackness", ColorSpace::Hwb, 2, Unit::Percent, false)
}

pub(crate) fn whiteness(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    legacy_channel_function(args, "whiteness", ColorSpace::Hwb, 1, Unit::Percent, false)
}

/// `color.hwb($channels)` or `color.hwb($hue, $whiteness, $blackness,
/// $alpha: 1)`. The comma-separated form is rewritten into the
/// space-separated syntax and parsed the same way, as Dart Sass does.
pub(crate) fn hwb(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(4)?;
    let span = args.span();

    if args.len() <= 1 {
        let channels = args.get_err(0, "channels")?;
        return parse_channels(
            "hwb",
            channels,
            Some(ColorSpace::Hwb),
            Some("channels"),
            visitor,
            span,
        );
    }

    if args.len() == 2 {
        args.max_args(1)?;
    }

    let hue = args.get_err(0, "hue")?;
    let whiteness = args.get_err(1, "whiteness")?;
    let blackness = args.get_err(2, "blackness")?;
    let alpha = args.default_arg(3, "alpha", Value::Dimension(SassNumber::new_unitless(1.0)));

    let channels = Value::List(
        vec![
            Value::List(
                vec![hue, whiteness, blackness],
                ListSeparator::Space,
                Brackets::None,
            ),
            alpha,
        ],
        ListSeparator::Slash,
        Brackets::None,
    );

    parse_channels("hwb", channels, Some(ColorSpace::Hwb), None, visitor, span)
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
