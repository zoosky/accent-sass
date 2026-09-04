//! `alpha()`, `opacity()`, and the legacy `opacify()`/`transparentize()`
//! family.

use crate::{builtin::builtin_imports::*, color::clamp_like_css};

use super::{function_string, legacy_only_error};

/// Check if `s` matches the regex `^[a-zA-Z]+\s*=`
fn is_ms_filter(s: &str) -> bool {
    let mut bytes = s.bytes();

    if !bytes.next().is_some_and(|c| c.is_ascii_alphabetic()) {
        return false;
    }

    bytes
        .skip_while(u8::is_ascii_alphabetic)
        .find(|c| !matches!(c, b' ' | b'\t' | b'\n'))
        == Some(b'=')
}

/// The shared body of the global `alpha()` and `color.alpha()`, which only
/// differ in how the legacy-only error names the function.
fn alpha_inner(mut args: ArgumentResult, function: &str) -> SassResult<Value> {
    if args.len() <= 1 {
        let span = args.span();
        let color = args.get_err(0, "color")?;

        if let Value::String(s, QuoteKind::None) = &color
            && is_ms_filter(s)
        {
            return Ok(Value::String(format!("alpha({})", s), QuoteKind::None));
        }

        if let Value::Color(color) = &color
            && !color.is_legacy()
        {
            return Err(legacy_only_error(function, "color.channel()", false, span));
        }

        let color = color.assert_color_with_name("color", span)?;

        Ok(Value::Dimension(SassNumber::new_unitless(Number(
            color.alpha(),
        ))))
    } else {
        let err = args.max_args(1);
        let args = args
            .get_variadic()?
            .into_iter()
            .map(|arg| match arg.node {
                Value::String(s, QuoteKind::None) if is_ms_filter(&s) => Ok(s),
                _ => {
                    err.clone()?;
                    unreachable!()
                }
            })
            .collect::<SassResult<Vec<String>>>()?;

        Ok(Value::String(
            format!("alpha({})", args.join(", "),),
            QuoteKind::None,
        ))
    }
}

pub(crate) fn alpha(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    alpha_inner(args, "alpha")
}

/// `color.alpha()` from the `sass:color` module.
pub(crate) fn module_alpha(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    alpha_inner(args, "color.alpha")
}

/// `opacity($color)`: the alpha of any color. The global function also
/// passes a number, or a value only CSS can evaluate, through as the
/// plain-CSS filter function; `color.opacity()` only does so for a number.
fn opacity_inner(
    mut args: ArgumentResult,
    visitor: &mut Visitor,
    global: bool,
) -> SassResult<Value> {
    args.max_args(1)?;
    let span = args.span();
    match args.get_err(0, "color")? {
        Value::Color(c) => Ok(Value::Dimension(SassNumber::new_unitless(Number(
            c.alpha(),
        )))),
        value
            if matches!(value, Value::Dimension(..)) || (global && value.is_special_function()) =>
        {
            function_string("opacity", &[value], visitor, span)
        }
        v => Err((
            format!("$color: {} is not a color.", v.inspect(span)?),
            span,
        )
            .into()),
    }
}

pub(crate) fn opacity(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    opacity_inner(args, visitor, true)
}

/// `color.opacity()` from the `sass:color` module.
pub(crate) fn module_opacity(args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    opacity_inner(args, visitor, false)
}

/// The shared body of `opacify()`/`fade-in()` and
/// `transparentize()`/`fade-out()`: adds `sign * $amount` to the alpha of
/// a legacy color, clamped to `0..1`.
fn adjust_alpha(mut args: ArgumentResult, name: &str, sign: f64) -> SassResult<Value> {
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

    amount.assert_bounds_with_unit("amount", 0.0, 1.0, &Unit::None, span)?;

    Ok(Value::Color(Arc::new(color.change_alpha(clamp_like_css(
        color.alpha() + sign * amount.num.0,
        0.0,
        1.0,
    )))))
}

fn opacify(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    adjust_alpha(args, "opacify", 1.0)
}

fn fade_in(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    adjust_alpha(args, "fade-in", 1.0)
}

fn transparentize(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    adjust_alpha(args, "transparentize", -1.0)
}

fn fade_out(args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    adjust_alpha(args, "fade-out", -1.0)
}

pub(crate) fn declare(f: &mut GlobalFunctionMap) {
    f.insert("alpha", Builtin::new(alpha));
    f.insert("opacity", Builtin::new(opacity));
    f.insert("opacify", Builtin::new(opacify));
    f.insert("fade-in", Builtin::new(fade_in));
    f.insert("transparentize", Builtin::new(transparentize));
    f.insert("fade-out", Builtin::new(fade_out));
}

#[cfg(test)]
mod test {
    use super::is_ms_filter;
    #[test]
    fn test_is_ms_filter() {
        assert!(is_ms_filter("a=a"));
        assert!(is_ms_filter("a="));
        assert!(is_ms_filter("a  \t\n  =a"));
        assert!(!is_ms_filter("a  \t\n  a=a"));
        assert!(!is_ms_filter("aa"));
        assert!(!is_ms_filter("   aa"));
        assert!(!is_ms_filter("=a"));
        assert!(!is_ms_filter("1=a"));
    }
}
