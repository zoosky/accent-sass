use std::io::Write;

use codemap::{CodeMap, Span};

use crate::{
    Options,
    ast::Mixin,
    ast::{CssStmt, MediaQuery, Style, SupportsRule},
    color::{Color, ColorFormat, ColorSpace, NAMED_COLORS},
    common::{BinaryOp, Brackets, ListSeparator, QuoteKind},
    error::SassResult,
    selector::{
        Combinator, ComplexSelector, ComplexSelectorComponent, CompoundSelector, Namespace, Pseudo,
        SelectorList, SimpleSelector,
    },
    unit::Unit,
    utils::hex_char_for,
    value::{
        ArgList, CalculationArg, CalculationName, Number, SassCalculation, SassFunction, SassMap,
        SassNumber, Value, fuzzy_equals, fuzzy_greater_than_or_equals, fuzzy_less_than,
        fuzzy_less_than_or_equals,
    },
};

pub(crate) fn serialize_selector_list(
    list: &SelectorList,
    options: &Options,
    span: Span,
) -> String {
    let map = CodeMap::new();
    let mut serializer = Serializer::new(options, &map, false, span);

    serializer.write_selector_list(list);

    serializer.finish_for_expr()
}

pub(crate) fn serialize_calculation_arg(
    arg: &CalculationArg,
    options: &Options,
    span: Span,
) -> SassResult<String> {
    let map = CodeMap::new();
    let mut serializer = Serializer::new(options, &map, false, span);

    serializer.write_calculation_arg(arg)?;

    Ok(serializer.finish_for_expr())
}

pub(crate) fn serialize_value(val: &Value, options: &Options, span: Span) -> SassResult<String> {
    let map = CodeMap::new();
    let mut serializer = Serializer::new(options, &map, false, span);

    serializer.visit_value(val, span)?;

    Ok(serializer.finish_for_expr())
}

pub(crate) fn inspect_value(val: &Value, options: &Options, span: Span) -> SassResult<String> {
    let map = CodeMap::new();
    let mut serializer = Serializer::new(options, &map, true, span);

    serializer.visit_value(val, span)?;

    Ok(serializer.finish_for_expr())
}

pub(crate) fn inspect_float(number: f64, options: &Options, span: Span) -> String {
    let map = CodeMap::new();
    let mut serializer = Serializer::new(options, &map, true, span);

    serializer.write_float(number);

    serializer.finish_for_expr()
}

pub(crate) fn inspect_map(map: &SassMap, options: &Options, span: Span) -> SassResult<String> {
    let code_map = CodeMap::new();
    let mut serializer = Serializer::new(options, &code_map, true, span);

    serializer.visit_map(map, span)?;

    Ok(serializer.finish_for_expr())
}

pub(crate) fn inspect_function_ref(
    func: &SassFunction,
    options: &Options,
    span: Span,
) -> SassResult<String> {
    let code_map = CodeMap::new();
    let mut serializer = Serializer::new(options, &code_map, true, span);

    serializer.visit_function_ref(func, span)?;

    Ok(serializer.finish_for_expr())
}

/// Serializes a first-class mixin the way `meta.inspect` does.
pub(crate) fn inspect_mixin_ref(
    mixin: &Mixin,
    options: &Options,
    span: Span,
) -> SassResult<String> {
    let code_map = CodeMap::new();
    let mut serializer = Serializer::new(options, &code_map, true, span);

    serializer.visit_mixin_ref(mixin, span)?;

    Ok(serializer.finish_for_expr())
}

pub(crate) fn inspect_number(
    number: &SassNumber,
    options: &Options,
    span: Span,
) -> SassResult<String> {
    let map = CodeMap::new();
    let mut serializer = Serializer::new(options, &map, true, span);

    serializer.visit_number(number)?;

    Ok(serializer.finish_for_expr())
}

pub(crate) struct Serializer<'a> {
    indentation: usize,
    options: &'a Options<'a>,
    inspect: bool,
    indent_width: usize,
    // todo: use this field
    _quote: bool,
    buffer: Vec<u8>,
    map: &'a CodeMap,
    // todo: use this field
    _span: Span,
    /// Set while serializing a comment that trails a statement on the same
    /// output line, so no indentation is written before it.
    inline_comment: bool,
}

impl<'a> Serializer<'a> {
    pub fn new(options: &'a Options<'a>, map: &'a CodeMap, inspect: bool, span: Span) -> Self {
        Self {
            inspect,
            _quote: true,
            indentation: 0,
            indent_width: 2,
            options,
            buffer: Vec::new(),
            map,
            _span: span,
            inline_comment: false,
        }
    }

    fn omit_spaces_around_complex_component(&self, component: &ComplexSelectorComponent) -> bool {
        self.options.is_compressed()
            && matches!(component, ComplexSelectorComponent::Combinator(..))
    }

    fn write_pseudo_selector(&mut self, pseudo: &Pseudo) {
        if let Some(sel) = &pseudo.selector {
            if pseudo.name == "not" && sel.is_invisible() {
                return;
            }
        }

        self.buffer.push(b':');

        if !pseudo.is_syntactic_class {
            self.buffer.push(b':');
        }

        self.buffer.extend_from_slice(pseudo.name.as_bytes());

        if pseudo.argument.is_none() && pseudo.selector.is_none() {
            return;
        }

        self.buffer.push(b'(');
        if let Some(arg) = &pseudo.argument {
            self.buffer.extend_from_slice(arg.as_bytes());
            if pseudo.selector.is_some() {
                self.buffer.push(b' ');
            }
        }

        if let Some(sel) = &pseudo.selector {
            self.write_selector_list(sel);
        }

        self.buffer.push(b')');
    }

    fn write_namespace(&mut self, namespace: &Namespace) {
        match namespace {
            Namespace::Empty => self.buffer.push(b'|'),
            Namespace::Asterisk => self.buffer.extend_from_slice(b"*|"),
            Namespace::Other(namespace) => {
                self.buffer.extend_from_slice(namespace.as_bytes());
                self.buffer.push(b'|');
            }
            Namespace::None => {}
        }
    }

    fn write_simple_selector(&mut self, simple: &SimpleSelector) {
        match simple {
            SimpleSelector::Id(name) => {
                self.buffer.push(b'#');
                self.buffer.extend_from_slice(name.as_bytes());
            }
            SimpleSelector::Class(name) => {
                self.buffer.push(b'.');
                self.buffer.extend_from_slice(name.as_bytes());
            }
            SimpleSelector::Placeholder(name) => {
                self.buffer.push(b'%');
                self.buffer.extend_from_slice(name.as_bytes());
            }
            SimpleSelector::Universal(namespace) => {
                self.write_namespace(namespace);
                self.buffer.push(b'*');
            }
            SimpleSelector::Pseudo(pseudo) => self.write_pseudo_selector(pseudo),
            SimpleSelector::Type(name) => {
                self.write_namespace(&name.namespace);
                self.buffer.extend_from_slice(name.ident.as_bytes());
            }
            SimpleSelector::Attribute(attr) => write!(&mut self.buffer, "{}", attr).unwrap(),
            SimpleSelector::Parent(..) => unreachable!("It should not be possible to format `&`."),
        }
    }

    fn write_compound_selector(&mut self, compound: &CompoundSelector) {
        let mut did_write = false;
        for simple in &compound.components {
            if did_write {
                self.write_simple_selector(simple);
            } else {
                let len = self.buffer.len();
                self.write_simple_selector(simple);
                if self.buffer.len() != len {
                    did_write = true;
                }
            }
        }

        // If we emit an empty compound, it's because all of the components got
        // optimized out because they match all selectors, so we just emit the
        // universal selector.
        if !did_write {
            self.buffer.push(b'*');
        }
    }

    fn write_complex_selector_component(&mut self, component: &ComplexSelectorComponent) {
        match component {
            ComplexSelectorComponent::Combinator(Combinator::NextSibling) => self.buffer.push(b'+'),
            ComplexSelectorComponent::Combinator(Combinator::Child) => self.buffer.push(b'>'),
            ComplexSelectorComponent::Combinator(Combinator::FollowingSibling) => {
                self.buffer.push(b'~')
            }
            ComplexSelectorComponent::Compound(compound) => self.write_compound_selector(compound),
        }
    }

    fn write_complex_selector(&mut self, complex: &ComplexSelector) {
        let mut last_component = None;

        for component in &complex.components {
            if let Some(c) = last_component {
                if !self.omit_spaces_around_complex_component(c)
                    && !self.omit_spaces_around_complex_component(component)
                {
                    self.buffer.push(b' ');
                }
            }
            self.write_complex_selector_component(component);
            last_component = Some(component);
        }
    }

    fn write_selector_list(&mut self, list: &SelectorList) {
        let complexes = list.components.iter().filter(|c| !c.is_invisible());

        let mut first = true;

        for complex in complexes {
            if first {
                first = false;
            } else {
                self.buffer.push(b',');
                if complex.line_break {
                    self.write_newline();
                    // Continuation lines of a selector list are indented to
                    // the current level, matching Dart Sass.
                    self.write_indentation();
                } else {
                    self.write_optional_space();
                }
            }
            self.write_complex_selector(complex);
        }
    }

    fn write_newline(&mut self) {
        if !self.options.is_compressed() {
            self.buffer.push(b'\n');
        }
    }

    fn write_comma_separator(&mut self) {
        self.buffer.push(b',');
        self.write_optional_space();
    }

    fn write_calculation_name(&mut self, name: CalculationName) {
        self.buffer.extend_from_slice(name.as_str().as_bytes());
    }

    fn visit_calculation(&mut self, calculation: &SassCalculation) -> SassResult<()> {
        self.write_calculation_name(calculation.name);
        self.buffer.push(b'(');

        if let Some((last, slice)) = calculation.args.split_last() {
            for arg in slice {
                self.write_calculation_arg(arg)?;
                self.write_comma_separator();
            }

            self.write_calculation_arg(last)?;
        }

        self.buffer.push(b')');

        Ok(())
    }

    fn write_calculation_arg(&mut self, arg: &CalculationArg) -> SassResult<()> {
        match arg {
            // A number inside a calculation is written inline rather than as a
            // nested `calc()`: `calc(infinity * 1px)`, not
            // `calc(calc(infinity * 1px))`.
            CalculationArg::Number(num) => {
                let has_plain_css_form = num.num.0.is_finite() && !num.unit.is_complex();

                if num.as_slash.is_none() && !has_plain_css_form {
                    self.write_calculation_number_body(num)?;
                } else {
                    self.visit_number(num)?;
                }
            }
            CalculationArg::Space(args) => {
                for (idx, arg) in args.iter().enumerate() {
                    if idx > 0 {
                        self.buffer.push(b' ');
                    }

                    self.write_calculation_arg(arg)?;
                }
            }
            CalculationArg::Calculation(calc) => {
                self.visit_calculation(calc)?;
            }
            CalculationArg::String(s) | CalculationArg::Interpolation(s) => {
                self.buffer.extend_from_slice(s.as_bytes());
            }
            CalculationArg::Operation { lhs, op, rhs } => {
                let paren_left = match &**lhs {
                    CalculationArg::Operation { op: op2, .. } => op2.precedence() < op.precedence(),
                    _ => false,
                };

                if paren_left {
                    self.buffer.push(b'(');
                }

                self.write_calculation_arg(lhs)?;

                if paren_left {
                    self.buffer.push(b')');
                }

                let operator_whitespace =
                    !self.options.is_compressed() || matches!(op, BinaryOp::Plus | BinaryOp::Minus);

                if operator_whitespace {
                    self.buffer.push(b' ');
                }

                // todo: avoid allocation with `write_binary_operator` method
                self.buffer.extend_from_slice(op.to_string().as_bytes());

                if operator_whitespace {
                    self.buffer.push(b' ');
                }

                let paren_right = match &**rhs {
                    CalculationArg::Operation { op: op2, .. } => {
                        CalculationArg::parenthesize_calculation_rhs(*op, *op2)
                    }
                    // A number written with unit factors behaves like a
                    // multiplication for precedence: `calc(a / (infinity * 1px))`.
                    CalculationArg::Number(num)
                        if num.as_slash.is_none() && Self::number_has_calculation_factors(num) =>
                    {
                        CalculationArg::parenthesize_calculation_rhs(*op, BinaryOp::Mul)
                    }
                    _ => false,
                };

                if paren_right {
                    self.buffer.push(b'(');
                }

                self.write_calculation_arg(rhs)?;

                if paren_right {
                    self.buffer.push(b')');
                }
            }
        }

        Ok(())
    }

    /// Dart Sass's `_asInt`: the channel as a whole number, or `None`. The
    /// check is exact outside inspect mode, so a channel with float noise
    /// (`204.99999999999994`) makes the whole color print as percentages;
    /// in inspect mode the check is fuzzy.
    fn as_int(&self, channel: f64) -> Option<f64> {
        let rounded = channel.round();
        let is_int = if self.inspect {
            fuzzy_equals(channel, rounded)
        } else {
            channel == rounded
        };
        if is_int { Some(rounded) } else { None }
    }

    /// Writes the three channels of an rgb-space color as comma-separated
    /// integers, or writes nothing and returns false when one of them is
    /// not a whole number.
    fn try_integer_rgb_channels(&mut self, rgb: &Color) -> bool {
        let red = match self.as_int(rgb.channel0()) {
            Some(red) => red,
            None => return false,
        };
        let green = match self.as_int(rgb.channel1()) {
            Some(green) => green,
            None => return false,
        };
        let blue = match self.as_int(rgb.channel2()) {
            Some(blue) => blue,
            None => return false,
        };

        self.write_float(red);
        self.write_comma_separator();
        self.write_float(green);
        self.write_comma_separator();
        self.write_float(blue);
        true
    }

    /// Writes a legacy color in the `rgb()`/`rgba()` syntax. Non-integral
    /// channels are written as percentages of 255, as Dart Sass does.
    fn write_rgb(&mut self, color: &Color) {
        let opaque = fuzzy_equals(color.alpha(), 1.0);
        let rgb = color.to_space(ColorSpace::Rgb, true);

        if opaque {
            self.buffer.extend_from_slice(b"rgb(");
        } else {
            self.buffer.extend_from_slice(b"rgba(");
        }

        if !self.try_integer_rgb_channels(&rgb) {
            self.write_channel(Some(rgb.channel0() * 100.0 / 255.0), Some(Unit::Percent));
            self.write_comma_separator();
            self.write_channel(Some(rgb.channel1() * 100.0 / 255.0), Some(Unit::Percent));
            self.write_comma_separator();
            self.write_channel(Some(rgb.channel2() * 100.0 / 255.0), Some(Unit::Percent));
        }

        if !opaque {
            self.write_comma_separator();
            self.write_float(color.alpha());
        }

        self.buffer.push(b')');
    }

    /// Writes one color channel the way Dart Sass's `_writeChannel` does:
    /// `none` for a missing channel, a plain number with `unit` for a
    /// finite one, and a `calc(..)` for a non-finite one
    /// (`calc(NaN * 1%)`).
    fn write_channel(&mut self, channel: Option<f64>, unit: Option<Unit>) {
        let value = match channel {
            Some(value) => value,
            None => {
                self.buffer.extend_from_slice(b"none");
                return;
            }
        };

        if value.is_finite() {
            self.write_float(value);
            if let Some(unit) = unit {
                let _ = write!(&mut self.buffer, "{}", unit);
            }
        } else {
            let number = SassNumber {
                num: Number(value),
                unit: unit.unwrap_or(Unit::None),
                as_slash: None,
            };
            // Writing a bare number never fails; the `Result` only exists
            // because the buffer implements `io::Write`.
            let _ = self.write_number_as_calculation(&number);
        }
    }

    /// Writes a legacy color in the `hsl()`/`hsla()` syntax, converting
    /// to hsl first. A missing hue reads as `0`.
    fn write_hsl(&mut self, color: &Color) {
        let opaque = fuzzy_equals(color.alpha(), 1.0);
        let hsl = color.to_space(ColorSpace::Hsl, true);

        if opaque {
            self.buffer.extend_from_slice(b"hsl(");
        } else {
            self.buffer.extend_from_slice(b"hsla(");
        }

        self.write_channel(Some(hsl.channel0()), None);
        self.write_comma_separator();
        self.write_channel(Some(hsl.channel1()), Some(Unit::Percent));
        self.write_comma_separator();
        self.write_channel(Some(hsl.channel2()), Some(Unit::Percent));

        if !opaque {
            self.write_comma_separator();
            self.write_float(color.alpha());
        }

        self.buffer.push(b')');
    }

    /// Writes an hwb color in the modern `hwb(H W% B% / A)` syntax. Dart
    /// Sass only uses this in inspect mode (`@debug`, `inspect()`).
    fn write_hwb(&mut self, color: &Color) {
        let hwb = color.to_space(ColorSpace::Hwb, true);

        self.buffer.extend_from_slice(b"hwb(");
        self.write_float(hwb.channel0());
        self.buffer.push(b' ');
        self.write_float(hwb.channel1());
        self.buffer.push(b'%');
        self.buffer.push(b' ');
        self.write_float(hwb.channel2());
        self.buffer.push(b'%');

        if !fuzzy_equals(color.alpha(), 1.0) {
            self.buffer.extend_from_slice(b" / ");
            self.write_float(color.alpha());
        }

        self.buffer.push(b')');
    }

    /// Writes ` / A` when the color is not opaque (`/A` when compressed),
    /// with `none` for a missing alpha.
    fn write_slash_alpha(&mut self, color: &Color) {
        if fuzzy_equals(color.alpha(), 1.0) {
            return;
        }
        self.write_optional_space();
        self.buffer.push(b'/');
        self.write_optional_space();
        self.write_channel(color.alpha_or_none(), None);
    }

    /// Writes a color in the `color(<space> c0 c1 c2 / A)` syntax, which
    /// is how every space without a dedicated CSS function serializes.
    fn write_color_function(&mut self, color: &Color) {
        self.buffer.extend_from_slice(b"color(");
        self.buffer
            .extend_from_slice(color.space().name().as_bytes());
        for channel in color.channels_or_none() {
            self.buffer.push(b' ');
            self.write_channel(channel, None);
        }
        self.write_slash_alpha(color);
        self.buffer.push(b')');
    }

    fn write_hex_component(&mut self, channel: u32) {
        debug_assert!(channel < 256);

        self.buffer.push(hex_char_for(channel >> 4) as u8);
        self.buffer.push(hex_char_for(channel & 0xF) as u8);
    }

    fn is_symmetrical_hex(channel: u32) -> bool {
        channel & 0xF == channel >> 4
    }

    fn can_use_short_hex(red: u32, green: u32, blue: u32) -> bool {
        Self::is_symmetrical_hex(red)
            && Self::is_symmetrical_hex(green)
            && Self::is_symmetrical_hex(blue)
    }

    /// Whether every channel of an rgb-space color is a whole number
    /// within `0..=255`, so the color can be written as a hex code (Dart
    /// Sass's `_canUseHex`).
    fn can_use_hex(rgb: &Color) -> bool {
        rgb.channels().iter().all(|&channel| {
            fuzzy_equals(channel, channel.round())
                && fuzzy_greater_than_or_equals(channel, 0.0)
                && fuzzy_less_than(channel, 256.0)
        })
    }

    /// The rounded channels of an rgb-space color that can be written as
    /// hex.
    fn hex_channels(rgb: &Color) -> (u32, u32, u32) {
        let [red, green, blue] = rgb.channels();
        (
            red.round() as u32,
            green.round() as u32,
            blue.round() as u32,
        )
    }

    /// The color's name, when an opaque rgb-space color matches one.
    fn color_name(rgb: &Color) -> Option<&'static str> {
        if !Self::can_use_hex(rgb) {
            return None;
        }
        let (red, green, blue) = Self::hex_channels(rgb);
        NAMED_COLORS
            .get_by_rgba([red as u8, green as u8, blue as u8])
            .copied()
    }

    /// Writes an opaque rgb-space color as whichever of its name, short
    /// hex, or long hex is shortest, or writes nothing and returns false
    /// when it cannot be written as hex.
    fn try_hex_or_named_rgb(&mut self, rgb: &Color) -> bool {
        if !Self::can_use_hex(rgb) {
            return false;
        }

        let (red, green, blue) = Self::hex_channels(rgb);
        let short_hex = Self::can_use_short_hex(red, green, blue);
        let max_name_length = if short_hex { 4 } else { 7 };

        if let Some(name) = Self::color_name(rgb).filter(|name| name.len() <= max_name_length) {
            self.buffer.extend_from_slice(name.as_bytes());
        } else if short_hex {
            self.buffer.push(b'#');
            self.buffer.push(hex_char_for(red & 0xF) as u8);
            self.buffer.push(hex_char_for(green & 0xF) as u8);
            self.buffer.push(hex_char_for(blue & 0xF) as u8);
        } else {
            self.buffer.push(b'#');
            self.write_hex_component(red);
            self.write_hex_component(green);
            self.write_hex_component(blue);
        }
        true
    }

    /// Writes a legacy color with no missing channels, choosing the
    /// representation the way Dart Sass's `_writeLegacyColor` does:
    ///
    /// 1. an out-of-gamut color can only be represented exactly in hsl;
    /// 2. compressed output takes the shortest of hex, name, rgb, or hsl;
    /// 3. an hsl-space color always keeps hsl form;
    /// 4. a color written as `rgb()`, a hex code, or a name keeps that form;
    /// 5. otherwise an opaque whole-number color becomes a name or hex code,
    ///    and anything else is `rgb()`, or `hsl()` for an hwb-space color.
    fn write_legacy_color(&mut self, color: &Color) {
        let opaque = fuzzy_equals(color.alpha(), 1.0);

        if !color.is_in_gamut() && !self.inspect {
            self.write_hsl(color);
            return;
        }

        if self.options.is_compressed() {
            let rgb = color.to_space(ColorSpace::Rgb, true);
            if opaque && self.try_hex_or_named_rgb(&rgb) {
                return;
            }

            // Emit whichever of rgb and hsl is shorter, computing hsl from
            // the rgb channels as Dart Sass does. The two extra characters
            // account for the `%` signs on saturation and lightness.
            let start = self.buffer.len();
            self.write_rgb(&rgb);
            let rgb_string = self.buffer.split_off(start);
            self.write_hsl(&rgb.to_space(ColorSpace::Hsl, true));
            let hsl_string = self.buffer.split_off(start);
            if rgb_string.len() <= hsl_string.len() + 2 {
                self.buffer.extend_from_slice(&rgb_string);
            } else {
                self.buffer.extend_from_slice(&hsl_string);
            }
            return;
        }

        if color.space() == ColorSpace::Hsl {
            self.write_hsl(color);
            return;
        } else if self.inspect && color.space() == ColorSpace::Hwb {
            self.write_hwb(color);
            return;
        }

        match &color.format {
            ColorFormat::Rgb => {
                self.write_rgb(color);
                return;
            }
            ColorFormat::Literal(text) => {
                self.buffer.extend_from_slice(text.as_bytes());
                return;
            }
            ColorFormat::Infer => {}
        }

        // Always emit generated transparent colors in rgba format. This works
        // around an IE bug. See sass/sass#1782.
        if opaque {
            let rgb = color.to_space(ColorSpace::Rgb, true);
            if let Some(name) = Self::color_name(&rgb) {
                self.buffer.extend_from_slice(name.as_bytes());
                return;
            }

            if Self::can_use_hex(&rgb) {
                let (red, green, blue) = Self::hex_channels(&rgb);
                self.buffer.push(b'#');
                self.write_hex_component(red);
                self.write_hex_component(green);
                self.write_hex_component(blue);
                return;
            }
        }

        // An hwb color that can't be written as hex is written as hsl
        // rather than rgb, since that more clearly captures the author's
        // intent.
        if color.space() == ColorSpace::Hwb {
            self.write_hsl(color);
        } else {
            self.write_rgb(color);
        }
    }

    /// Writes a color the way Dart Sass's `visitColor` does. A legacy color
    /// with no missing channels takes the legacy syntax; anything else uses
    /// the CSS Color 4 syntax of its space, with `none` for missing
    /// channels. A lab-family color whose lightness is out of range (or
    /// whose chroma is negative) has no direct CSS form, so it is written
    /// as a `color-mix()` from its xyz value, or with a relative `from`
    /// prefix when it also has missing channels.
    pub fn visit_color(&mut self, color: &Color) {
        let space = color.space();
        let compressed = self.options.is_compressed();
        let [channel0, channel1, channel2] = color.channels_or_none();
        let missing0 = channel0.is_none();
        let missing1 = channel1.is_none();
        let missing2 = channel2.is_none();
        let fuzzy_in_range = |number: f64, min: f64, max: f64| {
            fuzzy_greater_than_or_equals(number, min) && fuzzy_less_than_or_equals(number, max)
        };

        match space {
            ColorSpace::Rgb | ColorSpace::Hsl | ColorSpace::Hwb if !color.has_missing_channel() => {
                self.write_legacy_color(color);
            }
            ColorSpace::Rgb => {
                self.buffer.extend_from_slice(b"rgb(");
                self.write_channel(channel0, None);
                self.buffer.push(b' ');
                self.write_channel(channel1, None);
                self.buffer.push(b' ');
                self.write_channel(channel2, None);
                self.write_slash_alpha(color);
                self.buffer.push(b')');
            }
            ColorSpace::Hsl | ColorSpace::Hwb => {
                self.buffer.extend_from_slice(space.name().as_bytes());
                self.buffer.push(b'(');
                self.write_channel(channel0, if compressed { None } else { Some(Unit::Deg) });
                self.buffer.push(b' ');
                self.write_channel(channel1, Some(Unit::Percent));
                self.buffer.push(b' ');
                self.write_channel(channel2, Some(Unit::Percent));
                self.write_slash_alpha(color);
                self.buffer.push(b')');
            }
            ColorSpace::Lab | ColorSpace::Lch | ColorSpace::Oklab | ColorSpace::Oklch => {
                let polar = space.is_polar();
                let lightness_max = match space {
                    ColorSpace::Lab | ColorSpace::Lch => 100.0,
                    _ => 1.0,
                };
                let lightness_out_of_range = !fuzzy_in_range(color.channel0(), 0.0, lightness_max);
                let negative_chroma = polar && fuzzy_less_than(color.channel1(), 0.0);

                if !self.inspect
                    && ((lightness_out_of_range && !missing1 && !missing2)
                        || (negative_chroma && !missing0 && !missing1))
                {
                    self.buffer.extend_from_slice(b"color-mix(in ");
                    self.buffer.extend_from_slice(space.name().as_bytes());
                    self.write_comma_separator();
                    self.write_color_function(&color.to_space(ColorSpace::XyzD65, true));
                    self.write_optional_space();
                    self.buffer.extend_from_slice(b"100%");
                    self.write_comma_separator();
                    self.buffer
                        .extend_from_slice(if compressed { b"red" } else { b"black" });
                    self.buffer.push(b')');
                    return;
                }

                self.buffer.extend_from_slice(space.name().as_bytes());
                self.buffer.push(b'(');

                // Dart Sass checks the lightness against `0..100` here for
                // every lab-family space, oklab and oklch included.
                if !self.inspect
                    && (!fuzzy_in_range(color.channel0(), 0.0, 100.0) || negative_chroma)
                {
                    self.buffer.extend_from_slice(b"from ");
                    self.buffer
                        .extend_from_slice(if compressed { b"red" } else { b"black" });
                    self.buffer.push(b' ');
                }

                if !compressed && !missing0 {
                    self.write_float(color.channel0() * 100.0 / lightness_max);
                    self.buffer.push(b'%');
                } else {
                    self.write_channel(channel0, None);
                }
                self.buffer.push(b' ');
                self.write_channel(channel1, None);
                self.buffer.push(b' ');
                self.write_channel(
                    channel2,
                    if polar && !compressed {
                        Some(Unit::Deg)
                    } else {
                        None
                    },
                );
                self.write_slash_alpha(color);
                self.buffer.push(b')');
            }
            _ => self.write_color_function(color),
        }
    }

    fn write_media_query(&mut self, query: &MediaQuery) {
        if let Some(modifier) = &query.modifier {
            self.buffer.extend_from_slice(modifier.as_bytes());
            self.buffer.push(b' ');
        }

        if let Some(media_type) = &query.media_type {
            self.buffer.extend_from_slice(media_type.as_bytes());

            if !query.conditions.is_empty() {
                self.buffer.extend_from_slice(b" and ");
            }
        }

        if query.conditions.len() == 1 && query.conditions.first().unwrap().starts_with("(not ") {
            self.buffer.extend_from_slice(b"not ");
            let condition = query.conditions.first().unwrap();
            self.buffer
                .extend_from_slice(condition["(not ".len()..condition.len() - 1].as_bytes());
        } else {
            let operator = if query.conjunction { " and " } else { " or " };
            self.buffer
                .extend_from_slice(query.conditions.join(operator).as_bytes());
        }
    }

    pub fn visit_number(&mut self, number: &SassNumber) -> SassResult<()> {
        if let Some(as_slash) = &number.as_slash {
            self.visit_number(&as_slash.0)?;
            self.buffer.push(b'/');
            self.visit_number(&as_slash.1)?;
            return Ok(());
        }

        // A number that has no plain-CSS representation -- non-finite, or
        // carrying complex units -- is written as a `calc()` expression, which
        // is what Dart Sass emits and, unlike a bare `NaN` or `1px/em`, is
        // valid CSS.
        if !number.num.0.is_finite() || number.unit.is_complex() {
            return self.write_number_as_calculation(number);
        }

        self.write_float(number.num.0);
        write!(&mut self.buffer, "{}", number.unit)?;

        Ok(())
    }

    /// Writes a number as a `calc()` expression, mirroring Dart Sass's
    /// `_writeCalculationValue`: a non-finite value becomes the `infinity`,
    /// `-infinity`, or `NaN` keyword with every unit written as a factor
    /// (`calc(infinity * 1px)`), while a finite value keeps its first
    /// numerator unit attached (`calc(1px / 1em)`).
    fn write_number_as_calculation(&mut self, number: &SassNumber) -> SassResult<()> {
        self.buffer.extend_from_slice(b"calc(");
        self.write_calculation_number_body(number)?;
        self.buffer.push(b')');

        Ok(())
    }

    /// Whether [`Serializer::write_calculation_number_body`] writes more than a
    /// single term for this number, which decides whether it needs parentheses
    /// in an enclosing calculation operation.
    fn number_has_calculation_factors(number: &SassNumber) -> bool {
        if !number.num.0.is_finite() {
            return number.unit != Unit::None;
        }

        number.unit.is_complex()
    }

    /// Writes the body of a number's `calc()` form -- everything between the
    /// parentheses -- so that a number appearing inside a larger calculation is
    /// inlined (`calc(1% + infinity * 1px)`) instead of nested.
    fn write_calculation_number_body(&mut self, number: &SassNumber) -> SassResult<()> {
        let (numer, denom) = number.unit.clone().numer_and_denom();
        let value = number.num.0;
        let mut factors: &[Unit] = &numer;

        if value.is_finite() {
            self.write_float(value);
            if let Some((first, rest)) = numer.split_first() {
                write!(&mut self.buffer, "{}", first)?;
                factors = rest;
            }
        } else if value.is_nan() {
            self.buffer.extend_from_slice(b"NaN");
        } else if value.is_sign_negative() {
            self.buffer.extend_from_slice(b"-infinity");
        } else {
            self.buffer.extend_from_slice(b"infinity");
        }

        for unit in factors {
            self.write_optional_space();
            self.buffer.push(b'*');
            self.write_optional_space();
            write!(&mut self.buffer, "1{}", unit)?;
        }

        for unit in &denom {
            self.write_optional_space();
            self.buffer.push(b'/');
            self.write_optional_space();
            write!(&mut self.buffer, "1{}", unit)?;
        }

        Ok(())
    }

    /// Writes a number the way Dart Sass's `_writeNumber` does: a whole
    /// number as an integer, a short decimal as-is, and a longer one
    /// rounded to ten decimal places by decimal digit. Compressed output
    /// drops the zero before the decimal point (`.5`), except on a short
    /// negative number, which Dart Sass leaves alone (`-0.5`).
    fn write_float(&mut self, float: f64) {
        if float.is_infinite() && float.is_sign_negative() {
            self.buffer.extend_from_slice(b"-Infinity");
            return;
        } else if float.is_infinite() {
            self.buffer.extend_from_slice(b"Infinity");
            return;
        } else if float.is_nan() {
            self.buffer.extend_from_slice(b"NaN");
            return;
        }

        let rounded = float.round();
        let is_int = if self.inspect {
            fuzzy_equals(float, rounded)
        } else {
            float == rounded
        };
        if is_int {
            // Dart rounds to a 64-bit integer and prints its exact digits;
            // beyond that range the shortest representation is all that is
            // left.
            if rounded == 0.0 {
                self.buffer.push(b'0');
            } else if rounded.abs() < 9.0e18 {
                let _ = write!(&mut self.buffer, "{}", rounded as i64);
            } else {
                let _ = write!(&mut self.buffer, "{}", rounded);
            }
            return;
        }

        // Rust's `Display` for `f64` is the shortest round-trip
        // representation without an exponent, which is what Dart's
        // `toString()` gives after `_removeExponent`.
        let mut text = float.to_string();

        if self.inspect {
            self.buffer.extend_from_slice(text.as_bytes());
            return;
        }

        // `SassNumber.precision + 2`
        if text.len() < 12 {
            if self.options.is_compressed() && text.starts_with('0') {
                text.remove(0);
            }
            self.buffer.extend_from_slice(text.as_bytes());
            return;
        }

        self.write_rounded(&text);
    }

    /// Dart Sass's `_writeRounded`: rounds a decimal string to ten
    /// fractional digits by looking at the eleventh, carrying into the
    /// integer part when needed, and drops trailing zeros.
    fn write_rounded(&mut self, text: &str) {
        const PRECISION: usize = 10;

        if let Some(integer) = text.strip_suffix(".0") {
            self.buffer.extend_from_slice(integer.as_bytes());
            return;
        }

        let bytes = text.as_bytes();
        // One extra leading slot to carry into.
        let mut digits = vec![0u8; bytes.len() + 1];
        let mut digits_index = 1;

        let mut text_index = 0;
        let negative = bytes[0] == b'-';
        if negative {
            text_index += 1;
        }
        loop {
            if text_index == bytes.len() {
                self.buffer.extend_from_slice(bytes);
                return;
            }

            let byte = bytes[text_index];
            text_index += 1;
            if byte == b'.' {
                break;
            }
            digits[digits_index] = byte - b'0';
            digits_index += 1;
        }
        let first_fractional_digit = digits_index;

        let index_after_precision = text_index + PRECISION;
        if index_after_precision >= bytes.len() {
            self.buffer.extend_from_slice(bytes);
            return;
        }

        while text_index < index_after_precision {
            digits[digits_index] = bytes[text_index] - b'0';
            digits_index += 1;
            text_index += 1;
        }

        if bytes[text_index] - b'0' >= 5 {
            loop {
                digits[digits_index - 1] += 1;
                if digits[digits_index - 1] != 10 {
                    break;
                }
                digits_index -= 1;
            }
        }

        // Pad the integer part back out if the carry consumed it, then
        // drop trailing zeros from the fraction.
        while digits_index < first_fractional_digit {
            digits[digits_index] = 0;
            digits_index += 1;
        }
        while digits_index > first_fractional_digit && digits[digits_index - 1] == 0 {
            digits_index -= 1;
        }

        if digits_index == 2 && digits[0] == 0 && digits[1] == 0 {
            self.buffer.push(b'0');
            return;
        }

        if negative {
            self.buffer.push(b'-');
        }

        let mut written_index = 0;
        if digits[0] == 0 {
            written_index += 1;
            if self.options.is_compressed() && digits[1] == 0 {
                written_index += 1;
            }
        }
        while written_index < first_fractional_digit {
            self.buffer.push(b'0' + digits[written_index]);
            written_index += 1;
        }

        if digits_index > first_fractional_digit {
            self.buffer.push(b'.');
            while written_index < digits_index {
                self.buffer.push(b'0' + digits[written_index]);
                written_index += 1;
            }
        }
    }

    pub fn visit_group(
        &mut self,
        stmt: CssStmt,
        prev_was_group_end: bool,
        prev_requires_semicolon: bool,
    ) -> SassResult<()> {
        if prev_requires_semicolon {
            self.buffer.push(b';');
        }

        if !self.buffer.is_empty() {
            self.write_optional_newline();
        }

        if prev_was_group_end && !self.buffer.is_empty() {
            self.write_optional_newline();
        }

        self.visit_stmt(stmt)?;

        Ok(())
    }

    fn finish_for_expr(self) -> String {
        // SAFETY: todo
        unsafe { String::from_utf8_unchecked(self.buffer) }
    }

    pub fn finish(mut self, prev_requires_semicolon: bool) -> String {
        let is_not_ascii = self.buffer.iter().any(|&c| !c.is_ascii());

        if prev_requires_semicolon {
            self.buffer.push(b';');
        }

        if !self.buffer.is_empty() {
            self.write_optional_newline();
        }

        // SAFETY: todo
        let mut as_string = unsafe { String::from_utf8_unchecked(self.buffer) };

        if is_not_ascii && self.options.is_compressed() && self.options.allows_charset {
            as_string.insert(0, '\u{FEFF}');
        } else if is_not_ascii && self.options.allows_charset {
            as_string.insert_str(0, "@charset \"UTF-8\";\n");
        }

        as_string
    }

    fn write_indentation(&mut self) {
        if self.options.is_compressed() {
            return;
        }

        self.buffer.reserve(self.indentation);
        for _ in 0..self.indentation {
            self.buffer.push(b' ');
        }
    }

    fn write_list_separator(&mut self, sep: ListSeparator) {
        match (sep, self.options.is_compressed()) {
            (ListSeparator::Space | ListSeparator::Undecided, _) => self.buffer.push(b' '),
            (ListSeparator::Comma, true) => self.buffer.push(b','),
            (ListSeparator::Comma, false) => self.buffer.extend_from_slice(b", "),
            (ListSeparator::Slash, true) => self.buffer.push(b'/'),
            (ListSeparator::Slash, false) => self.buffer.extend_from_slice(b" / "),
        }
    }

    fn elem_needs_parens(sep: ListSeparator, elem: &Value) -> bool {
        match elem {
            Value::List(elems, sep2, brackets) => {
                if elems.len() < 2 {
                    return false;
                }

                if *brackets == Brackets::Bracketed {
                    return false;
                }

                match sep {
                    ListSeparator::Comma => *sep2 == ListSeparator::Comma,
                    ListSeparator::Slash => {
                        *sep2 == ListSeparator::Comma || *sep2 == ListSeparator::Slash
                    }
                    _ => *sep2 != ListSeparator::Undecided,
                }
            }
            _ => false,
        }
    }

    fn visit_list(
        &mut self,
        list_elems: &[Value],
        sep: ListSeparator,
        brackets: Brackets,
        span: Span,
    ) -> SassResult<()> {
        if brackets == Brackets::Bracketed {
            self.buffer.push(b'[');
        } else if list_elems.is_empty() {
            if !self.inspect {
                return Err(("() isn't a valid CSS value.", span).into());
            }

            self.buffer.extend_from_slice(b"()");
            return Ok(());
        }

        let is_singleton = self.inspect
            && list_elems.len() == 1
            && (sep == ListSeparator::Comma || sep == ListSeparator::Slash);

        if is_singleton && brackets != Brackets::Bracketed {
            self.buffer.push(b'(');
        }

        let (mut x, mut y);
        let elems: &mut dyn Iterator<Item = &Value> = if self.inspect {
            x = list_elems.iter();
            &mut x
        } else {
            y = list_elems.iter().filter(|elem| !elem.is_blank());
            &mut y
        };

        let mut elems = elems.peekable();

        while let Some(elem) = elems.next() {
            if self.inspect {
                let needs_parens = Self::elem_needs_parens(sep, elem);
                if needs_parens {
                    self.buffer.push(b'(');
                }

                self.visit_value(elem, span)?;

                if needs_parens {
                    self.buffer.push(b')');
                }
            } else {
                self.visit_value(elem, span)?;
            }

            if elems.peek().is_some() {
                self.write_list_separator(sep);
            }
        }

        if is_singleton {
            match sep {
                ListSeparator::Comma => self.buffer.push(b','),
                ListSeparator::Slash => self.buffer.push(b'/'),
                _ => unreachable!(),
            }

            if brackets != Brackets::Bracketed {
                self.buffer.push(b')');
            }
        }

        if brackets == Brackets::Bracketed {
            self.buffer.push(b']');
        }

        Ok(())
    }

    fn write_map_element(&mut self, value: &Value, span: Span) -> SassResult<()> {
        // An argument list is a comma-separated list too, so it needs the same
        // parentheses to keep the map unambiguous: `(positional: (1, 2))`, not
        // `(positional: 1, 2)`.
        let needs_parens = matches!(
            value,
            Value::List(_, ListSeparator::Comma, Brackets::None) | Value::ArgList(..)
        );

        if needs_parens {
            self.buffer.push(b'(');
        }

        self.visit_value(value, span)?;

        if needs_parens {
            self.buffer.push(b')');
        }

        Ok(())
    }

    fn visit_map(&mut self, map: &SassMap, span: Span) -> SassResult<()> {
        if !self.inspect {
            return Err((
                format!(
                    "{} isn't a valid CSS value.",
                    inspect_map(map, self.options, span)?
                ),
                span,
            )
                .into());
        }

        self.buffer.push(b'(');

        let mut elems = map.iter().peekable();

        while let Some((k, v)) = elems.next() {
            self.write_map_element(&k.node, k.span)?;
            self.buffer.extend_from_slice(b": ");
            self.write_map_element(v, k.span)?;
            if elems.peek().is_some() {
                self.buffer.extend_from_slice(b", ");
            }
        }

        self.buffer.push(b')');

        Ok(())
    }

    fn visit_unquoted_string(&mut self, string: &str) {
        let mut after_newline = false;
        self.buffer.reserve(string.len());

        for c in string.bytes() {
            match c {
                b'\n' => {
                    self.buffer.push(b' ');
                    after_newline = true;
                }
                b' ' => {
                    if !after_newline {
                        self.buffer.push(b' ');
                    }
                }
                _ => {
                    self.buffer.push(c);
                    after_newline = false;
                }
            }
        }
    }

    fn visit_quoted_string(&mut self, force_double_quote: bool, string: &str) {
        let mut has_single_quote = false;
        let mut has_double_quote = false;

        let mut buffer = Vec::new();

        if force_double_quote {
            buffer.push(b'"');
        }
        let mut iter = string.as_bytes().iter().copied().peekable();
        while let Some(c) = iter.next() {
            match c {
                b'\'' => {
                    if force_double_quote {
                        buffer.push(b'\'');
                    } else if has_double_quote {
                        self.visit_quoted_string(true, string);
                        return;
                    } else {
                        has_single_quote = true;
                        buffer.push(b'\'');
                    }
                }
                b'"' => {
                    if force_double_quote {
                        buffer.push(b'\\');
                        buffer.push(b'"');
                    } else if has_single_quote {
                        self.visit_quoted_string(true, string);
                        return;
                    } else {
                        has_double_quote = true;
                        buffer.push(b'"');
                    }
                }
                b'\x00'..=b'\x08' | b'\x0A'..=b'\x1F' => {
                    buffer.push(b'\\');
                    if c as u32 > 0xF {
                        buffer.push(hex_char_for(c as u32 >> 4) as u8);
                    }
                    buffer.push(hex_char_for(c as u32 & 0xF) as u8);

                    let next = match iter.peek() {
                        Some(v) => *v,
                        None => break,
                    };

                    if next.is_ascii_hexdigit() || next == b' ' || next == b'\t' {
                        buffer.push(b' ');
                    }
                }
                b'\\' => {
                    buffer.push(b'\\');
                    buffer.push(b'\\');
                }
                _ => buffer.push(c),
            }
        }

        if force_double_quote {
            buffer.push(b'"');
            self.buffer.extend_from_slice(&buffer);
        } else {
            let quote = if has_double_quote { b'\'' } else { b'"' };
            self.buffer.push(quote);
            self.buffer.extend_from_slice(&buffer);
            self.buffer.push(quote);
        }
    }

    fn visit_function_ref(&mut self, func: &SassFunction, span: Span) -> SassResult<()> {
        if !self.inspect {
            return Err((
                format!(
                    "{} isn't a valid CSS value.",
                    inspect_function_ref(func, self.options, span)?
                ),
                span,
            )
                .into());
        }

        self.buffer.extend_from_slice(b"get-function(");
        self.visit_quoted_string(false, func.name().as_str());
        self.buffer.push(b')');

        Ok(())
    }

    /// Writes a first-class mixin. Like a function reference, it has no plain
    /// CSS form, so it is only ever written under `inspect`.
    fn visit_mixin_ref(&mut self, mixin: &Mixin, span: Span) -> SassResult<()> {
        if !self.inspect {
            return Err((
                format!(
                    "{} isn't a valid CSS value.",
                    inspect_mixin_ref(mixin, self.options, span)?
                ),
                span,
            )
                .into());
        }

        self.buffer.extend_from_slice(b"get-mixin(");
        self.visit_quoted_string(false, mixin.name().as_str());
        self.buffer.push(b')');

        Ok(())
    }

    fn visit_arglist(&mut self, arglist: &ArgList, span: Span) -> SassResult<()> {
        self.visit_list(&arglist.elems, ListSeparator::Comma, Brackets::None, span)
    }

    fn visit_value(&mut self, value: &Value, span: Span) -> SassResult<()> {
        match value {
            Value::Dimension(num) => self.visit_number(num)?,
            Value::Color(color) => self.visit_color(color),
            Value::Calculation(calc) => self.visit_calculation(calc)?,
            Value::List(elems, sep, brackets) => self.visit_list(elems, *sep, *brackets, span)?,
            Value::True => self.buffer.extend_from_slice(b"true"),
            Value::False => self.buffer.extend_from_slice(b"false"),
            Value::Null => {
                if self.inspect {
                    self.buffer.extend_from_slice(b"null")
                }
            }
            Value::Map(map) => self.visit_map(map, span)?,
            Value::FunctionRef(func) => self.visit_function_ref(func, span)?,
            Value::MixinRef(mixin) => self.visit_mixin_ref(mixin.inner(), span)?,
            Value::String(s, QuoteKind::Quoted) => self.visit_quoted_string(false, s),
            Value::String(s, QuoteKind::None) => self.visit_unquoted_string(s),
            Value::ArgList(arglist) => self.visit_arglist(arglist, span)?,
        }

        Ok(())
    }

    fn write_style(&mut self, style: Style) -> SassResult<()> {
        if !self.options.is_compressed() {
            self.write_indentation();
        }

        self.buffer
            .extend_from_slice(style.property.resolve_ref().as_bytes());
        self.buffer.push(b':');

        // todo: _writeFoldedValue and _writeReindentedValue
        if !style.declared_as_custom_property && !self.options.is_compressed() {
            self.buffer.push(b' ');
        }

        self.visit_value(&style.value.node, style.value.span)?;

        Ok(())
    }

    fn write_import(&mut self, import: &str, modifiers: Option<String>) -> SassResult<()> {
        self.write_indentation();
        self.buffer.extend_from_slice(b"@import ");
        write!(&mut self.buffer, "{}", import)?;

        if let Some(modifiers) = modifiers {
            self.buffer.push(b' ');
            self.buffer.extend_from_slice(modifiers.as_bytes());
        }

        Ok(())
    }

    fn write_comment(&mut self, comment: &str, span: Span) -> SassResult<()> {
        if self.options.is_compressed() && !comment.starts_with("/*!") {
            return Ok(());
        }

        if !self.inline_comment {
            self.write_indentation();
        }
        let col = self.map.look_up_pos(span.low()).position.column;
        let mut lines = comment.lines();

        if let Some(line) = lines.next() {
            self.buffer.extend_from_slice(line.trim_start().as_bytes());
        }

        let lines = lines
            .map(|line| {
                let diff = (line.len() - line.trim_start().len()).saturating_sub(col);
                format!("{}{}", " ".repeat(diff), line.trim_start())
            })
            .collect::<Vec<String>>()
            .join("\n");

        if !lines.is_empty() {
            write!(&mut self.buffer, "\n{}", lines)?;
        }

        Ok(())
    }

    pub fn requires_semicolon(stmt: &CssStmt) -> bool {
        match stmt {
            CssStmt::Style(_) | CssStmt::Import(_, _) => true,
            CssStmt::UnknownAtRule(rule, _) => !rule.has_body,
            _ => false,
        }
    }

    /// The source line a statement ends on, for the statements that carry a
    /// span. Used to keep a trailing comment on the same output line as the
    /// declaration it follows, as Dart Sass does.
    fn stmt_end_line(&self, stmt: &CssStmt) -> Option<usize> {
        match stmt {
            CssStmt::Style(style) => {
                Some(self.map.look_up_pos(style.value.span.high()).position.line)
            }
            CssStmt::Comment(_, span) => Some(self.map.look_up_pos(span.high()).position.line),
            _ => None,
        }
    }

    /// Whether `stmt` is a comment that starts on `prev_end_line`, and should
    /// therefore be written on the same output line as the previous statement.
    fn is_trailing_comment(&self, stmt: &CssStmt, prev_end_line: Option<usize>) -> bool {
        if self.options.is_compressed() {
            return false;
        }
        match (stmt, prev_end_line) {
            (CssStmt::Comment(_, span), Some(prev_line)) => {
                self.map.look_up_pos(span.low()).position.line == prev_line
            }
            _ => false,
        }
    }

    fn write_children(&mut self, children: Vec<CssStmt>) -> SassResult<()> {
        if self.options.is_compressed() {
            self.buffer.push(b'{');
        } else {
            self.buffer.extend_from_slice(b" {\n");
        }

        self.indentation += self.indent_width;

        let len = children.len();
        let mut prev_end_line: Option<usize> = None;

        for (idx, child) in children.into_iter().enumerate() {
            let is_last = idx + 1 == len;
            let needs_semicolon = Self::requires_semicolon(&child);
            let end_line = self.stmt_end_line(&child);

            if self.is_trailing_comment(&child, prev_end_line) {
                // Rewind the newline written after the previous statement so
                // the comment lands on the same line, separated by a space.
                if self.buffer.last() == Some(&b'\n') {
                    self.buffer.pop();
                }
                self.buffer.push(b' ');
                self.inline_comment = true;
            }

            let did_write = self.visit_stmt(child)?;
            self.inline_comment = false;

            if !did_write {
                continue;
            }

            prev_end_line = end_line;

            if needs_semicolon && !(is_last && self.options.is_compressed()) {
                self.buffer.push(b';');
            }

            self.write_optional_newline();
        }

        self.indentation -= self.indent_width;

        if self.options.is_compressed() {
            self.buffer.push(b'}');
        } else {
            self.write_indentation();
            self.buffer.extend_from_slice(b"}");
        }

        Ok(())
    }

    fn write_optional_space(&mut self) {
        if !self.options.is_compressed() {
            self.buffer.push(b' ');
        }
    }

    fn write_optional_newline(&mut self) {
        if !self.options.is_compressed() {
            self.buffer.push(b'\n');
        }
    }

    fn write_supports_rule(&mut self, supports_rule: SupportsRule) -> SassResult<()> {
        self.write_indentation();
        self.buffer.extend_from_slice(b"@supports");

        if !supports_rule.params.is_empty() {
            self.buffer.push(b' ');
            self.buffer
                .extend_from_slice(supports_rule.params.as_bytes());
        }

        self.write_children(supports_rule.body)?;

        Ok(())
    }

    /// Returns whether or not text was written
    fn visit_stmt(&mut self, stmt: CssStmt) -> SassResult<bool> {
        if stmt.is_invisible() {
            return Ok(false);
        }

        match stmt {
            CssStmt::RuleSet { selector, body, .. } => {
                self.write_indentation();
                self.write_selector_list(&selector.as_selector_list());

                self.write_children(body)?;
            }
            CssStmt::Media(media_rule, ..) => {
                self.write_indentation();
                self.buffer.extend_from_slice(b"@media ");

                if let Some((last, rest)) = media_rule.query.split_last() {
                    for query in rest {
                        self.write_media_query(query);

                        self.buffer.push(b',');

                        self.write_optional_space();
                    }

                    self.write_media_query(last);
                }

                self.write_children(media_rule.body)?;
            }
            CssStmt::UnknownAtRule(unknown_at_rule, ..) => {
                self.write_indentation();
                self.buffer.push(b'@');
                self.buffer
                    .extend_from_slice(unknown_at_rule.name.as_bytes());

                if !unknown_at_rule.params.is_empty() {
                    write!(&mut self.buffer, " {}", unknown_at_rule.params)?;
                }

                if !unknown_at_rule.has_body {
                    debug_assert!(unknown_at_rule.body.is_empty());
                    return Ok(true);
                } else if unknown_at_rule.body.iter().all(CssStmt::is_invisible) {
                    self.buffer.extend_from_slice(b" {}");
                    return Ok(true);
                }

                self.write_children(unknown_at_rule.body)?;
            }
            CssStmt::Style(style) => self.write_style(style)?,
            CssStmt::Comment(comment, span) => self.write_comment(&comment, span)?,
            CssStmt::KeyframesRuleSet(keyframes_rule_set) => {
                self.write_indentation();
                // todo: i bet we can do something like write_with_separator to avoid extra allocation
                let selector = keyframes_rule_set
                    .selector
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");

                self.buffer.extend_from_slice(selector.as_bytes());

                self.write_children(keyframes_rule_set.body)?;
            }
            CssStmt::Import(import, modifier) => self.write_import(&import, modifier)?,
            CssStmt::Supports(supports_rule, _) => self.write_supports_rule(supports_rule)?,
        }

        Ok(true)
    }
}
