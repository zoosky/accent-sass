//! A color is a set of rgb channels, an alpha value, and the legacy color
//! space (`rgb`, `hsl`, or `hwb`) it belongs to.
//!
//! Colors can be constructed in Sass through names (e.g. red, blue, aqua)
//! or the builtin functions `rgb()`, `rgba()`, `hsl()`, `hsla()`, and
//! `hwb()`. Dart Sass 1.79+ remembers which space a color was written in
//! (or last converted to): the space decides how the color serializes
//! (`hsl(120, 50%, 50%)` stays in hsl form, `color.hwb(120 20% 30%)` prints
//! as hsl when it is not a whole-number rgb color), what `color.space()`
//! reports, which channels `color.channel()` sees, and whether a converted
//! achromatic color carries a *missing* hue.
//!
//! Every color keeps its rgb channels (unclamped, so out-of-gamut hsl
//! colors round-trip), plus the native channels of its space when that
//! space is hsl or hwb. Values are computed with the same operation order
//! as Dart Sass so serialized channels match it bit for bit; e.g.
//! `hsla(.999999999999, 100, 100, 1)` retains its full precision.
//!
//! Color values matching named colors are implicitly converted to named colors
//! E.g. `rgba(255, 0, 0, 1)` => `red`
//!
//! Named colors retain their original casing,
//! so `rEd` should be emitted as `rEd`.

use crate::value::{fuzzy_equals, fuzzy_round, Number};
pub(crate) use gamut::{clamp_like_css, GamutMapMethod};
pub(crate) use name::NAMED_COLORS;
pub(crate) use space::ColorSpace;
use space::{hsl_to_rgb, hwb_to_rgb, rgb_to_hsl, rgb_to_hwb};

mod gamut;
mod name;
mod space;

/// How the hue is interpolated between two colors in a polar space, per
/// <https://www.w3.org/TR/css-color-4/#hue-interpolation>.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HueInterpolationMethod {
    Shorter,
    Longer,
    Increasing,
    Decreasing,
}

impl HueInterpolationMethod {
    /// Parses the (case-insensitive) keyword used in `$method: hsl longer hue`.
    pub(crate) fn from_name(name: &str) -> Option<HueInterpolationMethod> {
        match name.to_ascii_lowercase().as_str() {
            "shorter" => Some(HueInterpolationMethod::Shorter),
            "longer" => Some(HueInterpolationMethod::Longer),
            "increasing" => Some(HueInterpolationMethod::Increasing),
            "decreasing" => Some(HueInterpolationMethod::Decreasing),
            _ => None,
        }
    }

    /// The name Dart Sass prints for the method in error messages.
    pub(crate) fn name(self) -> &'static str {
        match self {
            HueInterpolationMethod::Shorter => "shorter",
            HueInterpolationMethod::Longer => "longer",
            HueInterpolationMethod::Increasing => "increasing",
            HueInterpolationMethod::Decreasing => "decreasing",
        }
    }
}

// todo: only store alpha once on color
#[derive(Debug, Clone)]
pub struct Color {
    rgba: Rgb,
    /// The native channels when `space` is hsl; `None` otherwise.
    hsla: Option<Hsl>,
    /// The native channels when `space` is hwb; `None` otherwise.
    hwb: Option<Hwb>,
    alpha: Number,
    pub(crate) format: ColorFormat,
    /// The legacy space the color was written in or last converted to.
    space: ColorSpace,
    /// Whether the hue channel is missing (`none`). Only a color in a polar
    /// space can have a missing hue; it arises when an achromatic color is
    /// converted into hsl or hwb. Dart Sass serializes such a color as
    /// `hsl(none 0% 50%)` and refuses to modify the missing channel.
    missing_hue: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum ColorFormat {
    Rgb,
    Hsl,
    /// Literal string from source text. Either a named color like `red` or a hex color
    // todo: make this is a span and lookup text from codemap
    Literal(String),
    /// Use the most appropriate format
    Infer,
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        if self.alpha != other.alpha
            && !(self.alpha >= Number::one() && other.alpha >= Number::one())
        {
            return false;
        }

        self.rgba == other.rgba
    }
}

impl Eq for Color {}

impl Color {
    pub(crate) const fn new_rgba(
        red: Number,
        green: Number,
        blue: Number,
        alpha: Number,
        format: ColorFormat,
    ) -> Color {
        Color {
            rgba: Rgb::new(red, green, blue),
            alpha,
            hsla: None,
            hwb: None,
            format,
            space: ColorSpace::Rgb,
            missing_hue: false,
        }
    }

    /// The legacy space this color is in.
    pub(crate) fn space(&self) -> ColorSpace {
        self.space
    }

    /// Whether the hue channel is missing.
    pub(crate) fn missing_hue(&self) -> bool {
        self.missing_hue
    }

    /// Marks the hue missing (or present). A color in the rgb space has no
    /// hue, so the flag is only ever set on a polar-space color.
    pub(crate) fn with_missing_hue(mut self, missing: bool) -> Color {
        self.missing_hue = missing && self.space.is_polar();
        self
    }

    /// Converts a derived color back into the space of the color an
    /// operation was applied to, dropping any missing channel. This is what
    /// Dart Sass does at the end of nearly every color function
    /// (`.toSpace(color.space, legacyMissing: false)`).
    pub(crate) fn inherit_space(self, source: &Color) -> Color {
        self.to_space(source.space, false)
    }

    /// Converts this color to `space`.
    ///
    /// A color already in `space` is returned unchanged, missing hue and
    /// all. Otherwise the channels are recomputed from rgb. When
    /// `legacy_missing` is true, an achromatic result in a polar space (or
    /// a source whose hue was already missing) gets a missing hue, as it
    /// does in Dart Sass's `SassColor.toSpace`; when false, that hue is `0`.
    pub(crate) fn to_space(&self, space: ColorSpace, legacy_missing: bool) -> Color {
        if self.space == space {
            return self.clone();
        }

        match space {
            ColorSpace::Rgb => Color::new_rgba(
                self.red(),
                self.green(),
                self.blue(),
                self.alpha(),
                ColorFormat::Infer,
            ),
            ColorSpace::Hsl | ColorSpace::Hwb => {
                let (red, green, blue) = self.unit_rgb();
                Color::from_unit_rgb_in_space(
                    space,
                    red,
                    green,
                    blue,
                    self.alpha(),
                    legacy_missing,
                    self.missing_hue,
                )
            }
        }
    }

    /// Builds a color in `space` from unit-scale rgb, the way Dart Sass's
    /// `SrgbColorSpace.convert` does. See [`Color::to_space`] for the
    /// missing-hue rules.
    fn from_unit_rgb_in_space(
        space: ColorSpace,
        red: f64,
        green: f64,
        blue: f64,
        alpha: Number,
        legacy_missing: bool,
        source_missing_hue: bool,
    ) -> Color {
        let polar = match space {
            ColorSpace::Rgb => {
                return Color::new_rgba(
                    Number(red * 255.0),
                    Number(green * 255.0),
                    Number(blue * 255.0),
                    alpha,
                    ColorFormat::Infer,
                );
            }
            ColorSpace::Hsl => rgb_to_hsl(red, green, blue),
            ColorSpace::Hwb => rgb_to_hwb(red, green, blue),
        };

        let powerless = source_missing_hue || polar.achromatic;
        let hue = Number(if powerless { 0.0 } else { polar.hue_or_zero() });
        let color = match space {
            ColorSpace::Hsl => {
                Color::from_hsla(hue, Number(polar.channel1), Number(polar.channel2), alpha)
            }
            _ => Color::from_hwb(hue, Number(polar.channel1), Number(polar.channel2), alpha),
        };

        color.with_missing_hue(legacy_missing && powerless)
    }

    /// Builds a color in `space` from raw channel values in that space's
    /// units, without clamping (Dart Sass's `SassColor.forSpaceInternal`).
    pub(crate) fn in_space(
        space: ColorSpace,
        channel0: Number,
        channel1: Number,
        channel2: Number,
        alpha: Number,
    ) -> Color {
        match space {
            ColorSpace::Rgb => {
                Color::new_rgba(channel0, channel1, channel2, alpha, ColorFormat::Infer)
            }
            ColorSpace::Hsl => Color::from_hsla(channel0, channel1, channel2, alpha),
            ColorSpace::Hwb => Color::from_hwb(channel0, channel1, channel2, alpha),
        }
    }

    /// The rgb channels on the `0..1` scale, recomputed from the native
    /// channels of an hsl or hwb color so a conversion out of those spaces
    /// takes the same arithmetic path as in Dart Sass.
    fn unit_rgb(&self) -> (f64, f64, f64) {
        if let Some(hsl) = &self.hsla {
            hsl_to_rgb(hsl.hue, hsl.saturation, hsl.lightness)
        } else if let Some(hwb) = &self.hwb {
            hwb_to_rgb(hwb.hue, hwb.whiteness, hwb.blackness)
        } else {
            (
                self.red().0 / 255.0,
                self.green().0 / 255.0,
                self.blue().0 / 255.0,
            )
        }
    }

    /// The hsl channels (degrees, percent, percent): native for an hsl
    /// color, otherwise converted from rgb. A missing hue reads as `0`.
    fn hsl_view(&self) -> (f64, f64, f64) {
        match &self.hsla {
            Some(hsl) => (hsl.hue, hsl.saturation, hsl.lightness),
            None => {
                let (red, green, blue) = self.unit_rgb();
                let polar = rgb_to_hsl(red, green, blue);
                (polar.hue_or_zero(), polar.channel1, polar.channel2)
            }
        }
    }

    /// The hwb channels (degrees, percent, percent): native for an hwb
    /// color, otherwise converted from rgb. A missing hue reads as `0`.
    fn hwb_view(&self) -> (f64, f64, f64) {
        match &self.hwb {
            Some(hwb) => (hwb.hue, hwb.whiteness, hwb.blackness),
            None => {
                let (red, green, blue) = self.unit_rgb();
                let polar = rgb_to_hwb(red, green, blue);
                (polar.hue_or_zero(), polar.channel1, polar.channel2)
            }
        }
    }

    /// The channels of the color's own space, in that space's units, or
    /// `None` when the space has no channel with that name. A missing hue
    /// reads as `0`, as `color.channel()` reports it.
    pub(crate) fn native_channel(&self, name: &str) -> Option<Number> {
        let index = self
            .space
            .channel_names()
            .iter()
            .position(|channel| *channel == name)?;

        let channels = match self.space {
            ColorSpace::Rgb => [self.red().0, self.green().0, self.blue().0],
            ColorSpace::Hsl => {
                let (hue, saturation, lightness) = self.hsl_view();
                [hue, saturation, lightness]
            }
            ColorSpace::Hwb => {
                let (hue, whiteness, blackness) = self.hwb_view();
                [hue, whiteness, blackness]
            }
        };

        Some(Number(channels[index]))
    }

    /// The three channels of the color's own space, with the hue as `None`
    /// when it is missing.
    pub(crate) fn channels_or_none(&self) -> [Option<f64>; 3] {
        let names = self.space.channel_names();
        let channel = |name: &str| self.native_channel(name).map(|n| n.0);
        let hue = if self.missing_hue {
            None
        } else {
            channel(names[0])
        };
        [hue, channel(names[1]), channel(names[2])]
    }

    /// Whether two colors are the same color for `color.same()`.
    ///
    /// Colors in the same space compare channel by channel, with a missing
    /// hue reading as `0`; colors in different spaces compare by their rgb
    /// values. Dart Sass compares the latter in XYZ; the rgb comparison
    /// here is equivalent up to the fuzzy-equality epsilon.
    pub(crate) fn same(&self, other: &Color) -> bool {
        if !fuzzy_equals(self.alpha().0, other.alpha().0) {
            return false;
        }

        if self.space == other.space {
            let mine = self.channels_or_none();
            let theirs = other.channels_or_none();
            mine.iter()
                .zip(theirs.iter())
                .all(|(a, b)| fuzzy_equals(a.unwrap_or(0.0), b.unwrap_or(0.0)))
        } else {
            let (r1, g1, b1) = self.unit_rgb();
            let (r2, g2, b2) = other.unit_rgb();
            fuzzy_equals(r1, r2) && fuzzy_equals(g1, g2) && fuzzy_equals(b1, b2)
        }
    }
}

#[derive(Debug, Clone)]
struct Rgb {
    red: Number,
    green: Number,
    blue: Number,
}

impl PartialEq for Rgb {
    fn eq(&self, other: &Self) -> bool {
        if self.red != other.red && !(self.red >= Number(255.0) && other.red >= Number(255.0)) {
            return false;
        }
        if self.green != other.green
            && !(self.green >= Number(255.0) && other.green >= Number(255.0))
        {
            return false;
        }
        if self.blue != other.blue && !(self.blue >= Number(255.0) && other.blue >= Number(255.0)) {
            return false;
        }
        true
    }
}

impl Eq for Rgb {}

impl Rgb {
    pub const fn new(red: Number, green: Number, blue: Number) -> Self {
        Rgb { red, green, blue }
    }
}

/// The native channels of an hsl color: hue in degrees (normalized to
/// `0..360`), saturation and lightness as percentages.
#[derive(Debug, Clone)]
struct Hsl {
    hue: f64,
    saturation: f64,
    lightness: f64,
}

/// The native channels of an hwb color: hue in degrees (normalized to
/// `0..360`), whiteness and blackness as percentages. Whiteness and
/// blackness are stored as given; a sum above 100% is only scaled down when
/// computing rgb, as the CSS algorithm specifies.
#[derive(Debug, Clone)]
struct Hwb {
    hue: f64,
    whiteness: f64,
    blackness: f64,
}

// RGBA color functions
impl Color {
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8, format: String) -> Self {
        Color {
            rgba: Rgb::new(red.into(), green.into(), blue.into()),
            hsla: None,
            hwb: None,
            alpha: alpha.into(),
            format: ColorFormat::Literal(format),
            space: ColorSpace::Rgb,
            missing_hue: false,
        }
    }

    /// Create a new `Color` with just RGBA values.
    /// Color representation is created automatically.
    pub fn from_rgba(
        mut red: Number,
        mut green: Number,
        mut blue: Number,
        mut alpha: Number,
    ) -> Self {
        red = red.clamp(0.0, 255.0);
        green = green.clamp(0.0, 255.0);
        blue = blue.clamp(0.0, 255.0);
        alpha = alpha.clamp(0.0, 1.0);

        Color::new_rgba(red, green, blue, alpha, ColorFormat::Infer)
    }

    pub fn from_rgba_fn(
        mut red: Number,
        mut green: Number,
        mut blue: Number,
        mut alpha: Number,
    ) -> Self {
        red = red.clamp(0.0, 255.0);
        green = green.clamp(0.0, 255.0);
        blue = blue.clamp(0.0, 255.0);
        alpha = alpha.clamp(0.0, 1.0);

        Color::new_rgba(red, green, blue, alpha, ColorFormat::Rgb)
    }

    pub fn red(&self) -> Number {
        self.rgba.red
    }

    pub fn blue(&self) -> Number {
        self.rgba.blue
    }

    pub fn green(&self) -> Number {
        self.rgba.green
    }

    /// Mix two colors together with weight
    /// Algorithm adapted from
    /// <https://github.com/sass/dart-sass/blob/0d0270cb12a9ac5cce73a4d0785fecb00735feee/lib/src/functions/color.dart#L718>
    ///
    /// This is the legacy `mix()` (no `$method`); the result is always an
    /// rgb-space color, whatever the inputs were written in.
    pub fn mix(&self, other: &Color, weight: Number) -> Self {
        let weight = weight.clamp(0.0, 100.0);
        let normalized_weight = weight * Number(2.0) - Number::one();
        let alpha_distance = self.alpha() - other.alpha();

        let combined_weight1 = if normalized_weight * alpha_distance == Number(-1.0) {
            normalized_weight
        } else {
            (normalized_weight + alpha_distance)
                / (Number::one() + normalized_weight * alpha_distance)
        };
        let weight1 = (combined_weight1 + Number::one()) / Number(2.0);
        let weight2 = Number::one() - weight1;

        Color::new_rgba(
            self.red() * weight1 + other.red() * weight2,
            self.green() * weight1 + other.green() * weight2,
            self.blue() * weight1 + other.blue() * weight2,
            self.alpha() * weight + other.alpha() * (Number::one() - weight),
            ColorFormat::Infer,
        )
    }

    /// Interpolates between two colors in `space` according to the CSS
    /// Color 4 [color interpolation] procedure, with premultiplied alpha and
    /// the given hue method for polar spaces. `weight` is the share of
    /// `self` in the result. A missing hue on one side takes the other
    /// side's hue; missing on both sides stays missing.
    ///
    /// The result is converted back to `self`'s space; `legacy_missing`
    /// says whether that conversion keeps a missing hue.
    ///
    /// [color interpolation]: https://www.w3.org/TR/css-color-4/#interpolation
    pub(crate) fn interpolate(
        &self,
        other: &Color,
        space: ColorSpace,
        hue_method: HueInterpolationMethod,
        weight: f64,
        legacy_missing: bool,
    ) -> Color {
        if fuzzy_equals(weight, 0.0) {
            return other.clone();
        }
        if fuzzy_equals(weight, 1.0) {
            return self.clone();
        }

        let color1 = self.to_space(space, true);
        let color2 = other.to_space(space, true);
        let channels1 = color1.channels_or_none();
        let channels2 = color2.channels_or_none();

        let alpha1 = self.alpha().0;
        let alpha2 = other.alpha().0;
        let this_multiplier = alpha1 * weight;
        let other_multiplier = alpha2 * (1.0 - weight);
        let mixed_alpha = alpha1 * weight + alpha2 * (1.0 - weight);

        let pair = |index: usize| match (
            channels1[index].or(channels2[index]),
            channels2[index].or(channels1[index]),
        ) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        };
        let mixed = |index: usize| {
            pair(index).map(|(a, b)| (a * this_multiplier + b * other_multiplier) / mixed_alpha)
        };

        let (channel0, channel1, channel2) = if space.is_polar() {
            (
                pair(0).map(|(hue1, hue2)| interpolate_hues(hue1, hue2, hue_method, weight)),
                mixed(1),
                mixed(2),
            )
        } else {
            (mixed(0), mixed(1), mixed(2))
        };

        let unwrap = |channel: Option<f64>| Number(channel.unwrap_or(0.0));
        Color::in_space(
            space,
            unwrap(channel0),
            unwrap(channel1),
            unwrap(channel2),
            Number(mixed_alpha),
        )
        .with_missing_hue(channel0.is_none())
        .to_space(self.space, legacy_missing)
    }
}

/// Returns a hue partway between `hue1` and `hue2` according to `method`,
/// per <https://www.w3.org/TR/css-color-4/#hue-interpolation>.
fn interpolate_hues(
    mut hue1: f64,
    mut hue2: f64,
    method: HueInterpolationMethod,
    weight: f64,
) -> f64 {
    match method {
        HueInterpolationMethod::Shorter => {
            let difference = hue2 - hue1;
            if difference > 180.0 {
                hue1 += 360.0;
            } else if difference < -180.0 {
                hue2 += 360.0;
            }
        }
        HueInterpolationMethod::Longer => {
            let difference = hue2 - hue1;
            if difference > 0.0 && difference < 180.0 {
                hue2 += 360.0;
            } else if difference > -180.0 && difference <= 0.0 {
                hue1 += 360.0;
            }
        }
        HueInterpolationMethod::Increasing => {
            if hue2 < hue1 {
                hue2 += 360.0;
            }
        }
        HueInterpolationMethod::Decreasing => {
            if hue1 < hue2 {
                hue1 += 360.0;
            }
        }
    }

    hue1 * weight + hue2 * (1.0 - weight)
}

/// HSLA color functions
impl Color {
    /// The hue in degrees, as seen through the hsl space (`color.hue()`).
    pub fn hue(&self) -> Number {
        Number(self.hsl_view().0)
    }

    /// The saturation as a percentage, as seen through the hsl space.
    pub fn saturation(&self) -> Number {
        Number(self.hsl_view().1)
    }

    /// The lightness as a percentage, as seen through the hsl space.
    pub fn lightness(&self) -> Number {
        Number(self.hsl_view().2)
    }

    /// The color as (hue, saturation%, lightness%, alpha).
    pub fn as_hsla(&self) -> (Number, Number, Number, Number) {
        let (hue, saturation, lightness) = self.hsl_view();
        (
            Number(hue),
            Number(saturation),
            Number(lightness),
            self.alpha(),
        )
    }

    pub fn adjust_hue(&self, degrees: Number) -> Self {
        let (hue, saturation, lightness, alpha) = self.as_hsla();
        Color::from_hsla(hue + degrees, saturation, lightness, alpha).inherit_space(self)
    }

    /// Adds `amount` percentage points of lightness, clamped to `0..100`.
    pub fn lighten(&self, amount: Number) -> Self {
        let (hue, saturation, lightness, alpha) = self.as_hsla();
        Color::from_hsla(
            hue,
            saturation,
            (lightness + amount).clamp(0.0, 100.0),
            alpha,
        )
        .inherit_space(self)
    }

    /// Removes `amount` percentage points of lightness, clamped to `0..100`.
    pub fn darken(&self, amount: Number) -> Self {
        let (hue, saturation, lightness, alpha) = self.as_hsla();
        Color::from_hsla(
            hue,
            saturation,
            (lightness - amount).clamp(0.0, 100.0),
            alpha,
        )
        .inherit_space(self)
    }

    /// Adds `amount` percentage points of saturation, clamped to `0..100`.
    pub fn saturate(&self, amount: Number) -> Self {
        let (hue, saturation, lightness, alpha) = self.as_hsla();
        Color::from_hsla(
            hue,
            (saturation + amount).clamp(0.0, 100.0),
            lightness,
            alpha,
        )
        .inherit_space(self)
    }

    /// Removes `amount` percentage points of saturation, clamped to `0..100`.
    pub fn desaturate(&self, amount: Number) -> Self {
        let (hue, saturation, lightness, alpha) = self.as_hsla();
        Color::from_hsla(
            hue,
            (saturation - amount).clamp(0.0, 100.0),
            lightness,
            alpha,
        )
        .inherit_space(self)
    }

    /// `hsl()` as written in a stylesheet: the CSS channel is lower-clamped,
    /// so a negative saturation is parsed as `0%`.
    pub fn from_hsla_fn(hue: Number, saturation: Number, lightness: Number, alpha: Number) -> Self {
        let mut color = Self::from_hsla(hue, Number(saturation.0.max(0.0)), lightness, alpha);
        color.format = ColorFormat::Hsl;
        color
    }

    /// Creates an hsl-space color from hue (degrees), saturation and
    /// lightness (percentages), and alpha.
    ///
    /// The channels are not clamped, so out-of-gamut legacy colors
    /// round-trip. Like Dart Sass's `SassColor.forSpaceInternal`, a
    /// negative saturation is folded into the hue by rotating it 180
    /// degrees, and the hue is normalized to `0..360`.
    pub fn from_hsla(hue: Number, saturation: Number, lightness: Number, alpha: Number) -> Self {
        let invert = saturation.0 < 0.0 && !fuzzy_equals(saturation.0, 0.0);
        let hue =
            (hue.0.rem_euclid(360.0) + 360.0 + if invert { 180.0 } else { 0.0 }).rem_euclid(360.0);
        let saturation = saturation.0.abs();
        let lightness = lightness.0;

        let (red, green, blue) = hsl_to_rgb(hue, saturation, lightness);

        Color {
            rgba: Rgb::new(
                Number(red * 255.0),
                Number(green * 255.0),
                Number(blue * 255.0),
            ),
            hsla: Some(Hsl {
                hue,
                saturation,
                lightness,
            }),
            hwb: None,
            alpha,
            format: ColorFormat::Infer,
            space: ColorSpace::Hsl,
            missing_hue: false,
        }
    }

    /// Inverts the color in its own space, as `color.invert()` with a
    /// `$space` does: rgb channels flip around their range, the hue rotates
    /// 180 degrees, hsl lightness flips, and hwb whiteness and blackness
    /// swap. The hue must not be missing; the caller checks that.
    pub(crate) fn invert_channels(&self) -> Self {
        let alpha = self.alpha();
        match self.space {
            ColorSpace::Rgb => Color::new_rgba(
                Number(255.0) - self.red(),
                Number(255.0) - self.green(),
                Number(255.0) - self.blue(),
                alpha,
                ColorFormat::Infer,
            ),
            ColorSpace::Hsl => {
                let (hue, saturation, lightness) = self.hsl_view();
                Color::from_hsla(
                    Number((hue + 180.0).rem_euclid(360.0)),
                    Number(saturation),
                    Number(100.0 - lightness),
                    alpha,
                )
            }
            ColorSpace::Hwb => {
                let (hue, whiteness, blackness) = self.hwb_view();
                Color::from_hwb(
                    Number((hue + 180.0).rem_euclid(360.0)),
                    Number(blackness),
                    Number(whiteness),
                    alpha,
                )
            }
        }
    }

    /// The legacy `invert()` without a `$space`: inverts the rgb channels,
    /// mixes the result with the original by `weight` (`0..1`), and
    /// converts back to the original space keeping a missing hue, so an
    /// achromatic result written in hsl or hwb comes out as `hsl(none ..)`.
    pub fn invert(&self, weight: Number) -> Self {
        let inverse = Color::new_rgba(
            Number(255.0) - self.red(),
            Number(255.0) - self.green(),
            Number(255.0) - self.blue(),
            self.alpha(),
            ColorFormat::Infer,
        );

        inverse.mix(self, weight).to_space(self.space, true)
    }

    /// Rotates the hue of a polar-space color by `degrees`. The hue must
    /// not be missing; the caller checks that.
    pub(crate) fn rotate_hue(&self, degrees: f64) -> Self {
        let alpha = self.alpha();
        match self.space {
            ColorSpace::Rgb => self.clone(),
            ColorSpace::Hsl => {
                let (hue, saturation, lightness) = self.hsl_view();
                Color::from_hsla(
                    Number(hue + degrees),
                    Number(saturation),
                    Number(lightness),
                    alpha,
                )
            }
            ColorSpace::Hwb => {
                let (hue, whiteness, blackness) = self.hwb_view();
                Color::from_hwb(
                    Number(hue + degrees),
                    Number(whiteness),
                    Number(blackness),
                    alpha,
                )
            }
        }
    }

    /// The legacy `complement()`: rotates the hsl hue by 180 degrees and
    /// converts back to the original space.
    pub fn complement(&self) -> Self {
        self.to_space(ColorSpace::Hsl, false)
            .rotate_hue(180.0)
            .inherit_space(self)
    }

    /// `grayscale()`: drops the saturation in hsl and converts back to the
    /// original space. An hsl color keeps its hue (Dart Sass does not
    /// convert a color already in hsl), including a missing one.
    pub fn grayscale(&self) -> Self {
        let hsl = self.to_space(ColorSpace::Hsl, true);
        let (hue, _, lightness) = hsl.hsl_view();
        Color::from_hsla(Number(hue), Number(0.0), Number(lightness), hsl.alpha())
            .with_missing_hue(hsl.missing_hue)
            .inherit_space(self)
    }
}

/// Opacity color functions
impl Color {
    pub fn alpha(&self) -> Number {
        if self.alpha > Number::one() {
            self.alpha / Number(255.0)
        } else {
            self.alpha
        }
    }

    /// Change `alpha` to value given, keeping the color's space and channels.
    /// The source text of a literal no longer describes the color, so the
    /// result serializes from its channels.
    pub fn with_alpha(&self, alpha: Number) -> Self {
        let mut color = self.clone();
        color.alpha = alpha.clamp(0.0, 1.0);
        color.format = ColorFormat::Infer;
        color
    }

    /// Makes a color more opaque.
    /// Takes a color and a number between 0 and 1,
    /// and returns a color with the opacity increased by that amount.
    ///
    /// Unlike `color.adjust($alpha: ..)`, the legacy alpha functions
    /// rebuild the color and so drop a missing hue, as in Dart Sass.
    pub fn fade_in(&self, amount: Number) -> Self {
        self.with_alpha(self.alpha() + amount)
            .with_missing_hue(false)
    }

    /// Makes a color more transparent.
    /// Takes a color and a number between 0 and 1,
    /// and returns a color with the opacity decreased by that amount.
    pub fn fade_out(&self, amount: Number) -> Self {
        self.with_alpha(self.alpha() - amount)
            .with_missing_hue(false)
    }
}

/// Other color functions
impl Color {
    pub fn to_ie_hex_str(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            fuzzy_round(self.alpha().0 * 255.0) as u8,
            fuzzy_round(self.red().0) as u8,
            fuzzy_round(self.green().0) as u8,
            fuzzy_round(self.blue().0) as u8
        )
    }
}

/// HWB color functions
impl Color {
    /// Creates an hwb-space color from hue (degrees), whiteness and
    /// blackness (percentages), and alpha.
    ///
    /// Whiteness and blackness are stored as given. When they sum to more
    /// than 100% the rgb channels are computed from the proportionally
    /// scaled values, as the CSS algorithm specifies; `color.hwb()` and
    /// `color.change()` scale the stored channels themselves, while
    /// `color.adjust()` and `color.scale()` leave them out of gamut.
    pub fn from_hwb(hue: Number, whiteness: Number, blackness: Number, alpha: Number) -> Color {
        let hue = space::normalize_hue(hue.0);
        let whiteness = whiteness.0;
        let blackness = blackness.0;
        let alpha = alpha.clamp(0.0, 1.0);

        let (red, green, blue) = hwb_to_rgb(hue, whiteness, blackness);

        Color {
            rgba: Rgb::new(
                Number(red * 255.0),
                Number(green * 255.0),
                Number(blue * 255.0),
            ),
            hsla: None,
            hwb: Some(Hwb {
                hue,
                whiteness,
                blackness,
            }),
            alpha,
            format: ColorFormat::Infer,
            space: ColorSpace::Hwb,
            missing_hue: false,
        }
    }

    /// The whiteness as a percentage, as seen through the hwb space.
    pub fn whiteness(&self) -> Number {
        Number(self.hwb_view().1)
    }

    /// The blackness as a percentage, as seen through the hwb space.
    pub fn blackness(&self) -> Number {
        Number(self.hwb_view().2)
    }
}
