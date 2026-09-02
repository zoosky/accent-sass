//! A color is three channels in a color space, plus an alpha value.
//!
//! Colors can be constructed in Sass through names (e.g. red, blue, aqua),
//! hex codes, and the color functions: the legacy `rgb()`, `hsl()`, and
//! `hwb()`, and the CSS Color 4 `lab()`, `lch()`, `oklab()`, `oklch()`,
//! and `color()`. Dart Sass 1.79+ remembers which space a color was written
//! in (or last converted to): the space decides how the color serializes
//! (`hsl(120, 50%, 50%)` stays in hsl form, `lab(50% 10 20)` stays in lab
//! form), what `color.space()` reports, which channels `color.channel()`
//! sees, and how the color-space functions behave.
//!
//! Any channel, and the alpha, can be *missing* (`none` in CSS). A missing
//! channel reads as `0` wherever a number is needed, serializes as `none`,
//! and refuses to be modified by `color.adjust()` and friends. Conversions
//! carry a missing channel over to its analogue in the destination space
//! (see [`space`]).
//!
//! Channel values are computed with the same operation order as Dart Sass
//! so serialized channels match it bit for bit; e.g.
//! `hsla(.999999999999, 100, 100, 1)` retains its full precision.
//!
//! Color values matching named colors are implicitly converted to named colors
//! E.g. `rgba(255, 0, 0, 1)` => `red`
//!
//! Named colors retain their original casing,
//! so `rEd` should be emitted as `rEd`.

use crate::value::{fuzzy_equals, fuzzy_less_than, Number};
pub(crate) use channel::{ChannelKind, ColorChannel};
pub(crate) use gamut::{clamp_like_css, GamutMapMethod};
pub(crate) use name::NAMED_COLORS;
use space::dart_mod;
pub(crate) use space::ColorSpace;

mod channel;
mod conversions;
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

/// A color in one of the CSS Color 4 spaces (Dart Sass's `SassColor`).
#[derive(Debug, Clone)]
pub struct Color {
    /// The space the color was written in or last converted to.
    space: ColorSpace,
    /// The three channels in the space's own units. `None` is a missing
    /// channel.
    channels: [Option<f64>; 3],
    /// The alpha, within `0..1`. `None` is a missing alpha.
    alpha: Option<f64>,
    /// How the color was written, when that decides its serialization.
    pub(crate) format: ColorFormat,
}

/// How a legacy color was written in the stylesheet, when that affects the
/// serialization.
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum ColorFormat {
    /// Written with the `rgb()` or `rgba()` function, so it serializes as
    /// `rgb()` even when it could be a hex code.
    Rgb,
    /// Literal string from source text. Either a named color like `red` or a hex color
    // todo: make this is a span and lookup text from codemap
    Literal(String),
    /// Use the most appropriate format
    Infer,
}

/// `fuzzyEqualsNullable`: two missing channels are equal, a missing and a
/// present channel are not.
fn fuzzy_equals_nullable(a: Option<f64>, b: Option<f64>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => fuzzy_equals(a, b),
        _ => false,
    }
}

impl PartialEq for Color {
    /// Dart Sass's `SassColor.==`: legacy colors compare by their rgb
    /// values (channel by channel when in the same space), non-legacy
    /// colors only equal a color in the same space with the same channels.
    fn eq(&self, other: &Self) -> bool {
        if self.is_legacy() {
            if !other.is_legacy() {
                return false;
            }
            if !fuzzy_equals_nullable(self.alpha, other.alpha) {
                return false;
            }
            if self.space == other.space {
                self.channels
                    .iter()
                    .zip(other.channels.iter())
                    .all(|(a, b)| fuzzy_equals_nullable(*a, *b))
            } else {
                self.to_space(ColorSpace::Rgb, true) == other.to_space(ColorSpace::Rgb, true)
            }
        } else {
            self.space == other.space
                && self
                    .channels
                    .iter()
                    .zip(other.channels.iter())
                    .all(|(a, b)| fuzzy_equals_nullable(*a, *b))
                && fuzzy_equals_nullable(self.alpha, other.alpha)
        }
    }
}

impl Eq for Color {}

/// Dart Sass's `_normalizeHue`: wraps a hue to `0..360`, rotating it by
/// 180 degrees when `invert` is set (for a negative saturation or chroma).
fn normalize_hue(hue: Option<f64>, invert: bool) -> Option<f64> {
    hue.map(|hue| {
        dart_mod(
            dart_mod(hue, 360.0) + 360.0 + if invert { 180.0 } else { 0.0 },
            360.0,
        )
    })
}

impl Color {
    /// Builds a color in `space` from raw channel values in that space's
    /// units, without clamping (Dart Sass's `SassColor.forSpaceInternal`).
    ///
    /// Hues are normalized to `0..360`. A negative saturation or chroma is
    /// folded into the hue by rotating it 180 degrees, as CSS Color 4
    /// specifies. Alpha is clamped to `0..1`.
    pub(crate) fn for_space(
        space: ColorSpace,
        channel0: Option<f64>,
        channel1: Option<f64>,
        channel2: Option<f64>,
        alpha: Option<f64>,
    ) -> Color {
        debug_assert!(
            space != ColorSpace::Lms,
            "a color is never in the lms space"
        );

        let alpha = alpha.map(|alpha| clamp_like_css(alpha, 0.0, 1.0));
        let negative = |channel: Option<f64>| channel.map_or(false, |c| fuzzy_less_than(c, 0.0));

        let channels = match space {
            ColorSpace::Hsl => [
                normalize_hue(channel0, negative(channel1)),
                channel1.map(f64::abs),
                channel2,
            ],
            ColorSpace::Hwb => [normalize_hue(channel0, false), channel1, channel2],
            ColorSpace::Lch | ColorSpace::Oklch => [
                channel0,
                channel1.map(f64::abs),
                normalize_hue(channel2, negative(channel1)),
            ],
            _ => [channel0, channel1, channel2],
        };

        Color {
            space,
            channels,
            alpha,
            format: ColorFormat::Infer,
        }
    }

    /// An opaque rgb color from a named-color table entry, keeping the
    /// name's spelling for serialization.
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8, format: String) -> Self {
        let alpha = f64::from(alpha);
        Color {
            space: ColorSpace::Rgb,
            channels: [
                Some(f64::from(red)),
                Some(f64::from(green)),
                Some(f64::from(blue)),
            ],
            alpha: Some(if alpha > 1.0 { alpha / 255.0 } else { alpha }),
            format: ColorFormat::Literal(format),
        }
    }

    /// An rgb color from unclamped channels with the given format.
    pub(crate) fn new_rgba(
        red: Number,
        green: Number,
        blue: Number,
        alpha: Number,
        format: ColorFormat,
    ) -> Color {
        Color::for_space(
            ColorSpace::Rgb,
            Some(red.0),
            Some(green.0),
            Some(blue.0),
            Some(alpha.0),
        )
        .with_format(format)
    }

    /// Sets the serialization format. Only an rgb-space color has one.
    pub(crate) fn with_format(mut self, format: ColorFormat) -> Color {
        debug_assert!(format == ColorFormat::Infer || self.space == ColorSpace::Rgb);
        self.format = format;
        self
    }

    /// The space this color is in.
    pub(crate) fn space(&self) -> ColorSpace {
        self.space
    }

    /// Whether the color is in one of the legacy spaces (rgb, hsl, hwb).
    pub(crate) fn is_legacy(&self) -> bool {
        self.space.is_legacy()
    }

    /// The three channels, with `None` for a missing channel.
    pub(crate) fn channels_or_none(&self) -> [Option<f64>; 3] {
        self.channels
    }

    /// The three channels, with a missing channel reading as `0`.
    pub(crate) fn channels(&self) -> [f64; 3] {
        [self.channel0(), self.channel1(), self.channel2()]
    }

    /// The channel at `index`, with a missing channel reading as `0`.
    pub(crate) fn channel(&self, index: usize) -> f64 {
        self.channels[index].unwrap_or(0.0)
    }

    pub(crate) fn channel0(&self) -> f64 {
        self.channel(0)
    }

    pub(crate) fn channel1(&self) -> f64 {
        self.channel(1)
    }

    pub(crate) fn channel2(&self) -> f64 {
        self.channel(2)
    }

    /// The alpha, with a missing alpha reading as `0`.
    pub fn alpha(&self) -> f64 {
        self.alpha.unwrap_or(0.0)
    }

    /// The alpha, or `None` when it is missing.
    pub(crate) fn alpha_or_none(&self) -> Option<f64> {
        self.alpha
    }

    pub(crate) fn is_alpha_missing(&self) -> bool {
        self.alpha.is_none()
    }

    /// Whether the channel at `index` is missing.
    pub(crate) fn is_channel_missing(&self, index: usize) -> bool {
        self.channels[index].is_none()
    }

    /// Whether the channel at `index` is [powerless]: present, but with no
    /// effect on the color. That is the hue of an hsl color with zero
    /// saturation, of an hwb color whose whiteness and blackness sum to at
    /// least 100%, and of an lch or oklch color with zero chroma.
    ///
    /// [powerless]: https://www.w3.org/TR/css-color-4/#powerless
    pub(crate) fn is_channel_powerless(&self, index: usize) -> bool {
        match (self.space, index) {
            (ColorSpace::Hsl, 0) => fuzzy_equals(self.channel1(), 0.0),
            (ColorSpace::Hwb, 0) => {
                crate::value::fuzzy_greater_than_or_equals(self.channel1() + self.channel2(), 100.0)
            }
            (ColorSpace::Lch | ColorSpace::Oklch, 2) => fuzzy_equals(self.channel1(), 0.0),
            _ => false,
        }
    }

    /// Whether any channel, or the alpha, is missing.
    pub(crate) fn has_missing_channel(&self) -> bool {
        self.channels.iter().any(Option::is_none) || self.alpha.is_none()
    }

    /// Converts this color to `space`.
    ///
    /// A color already in `space` is returned unchanged, missing channels
    /// and all. Otherwise the channels are converted (see
    /// [`ColorSpace::convert`]) and a missing alpha becomes `0`. When `legacy_missing` is false, a legacy
    /// result has its missing channels replaced by `0`, which is what most
    /// color functions do so that a legacy color never surprises a
    /// stylesheet with `none`.
    pub(crate) fn to_space(&self, space: ColorSpace, legacy_missing: bool) -> Color {
        if self.space == space {
            return self.clone();
        }

        // Dart Sass passes the non-null alpha here, so a missing alpha
        // reads as `0` after any conversion.
        let converted = self.space.convert(
            space,
            self.channels[0],
            self.channels[1],
            self.channels[2],
            Some(self.alpha()),
        );

        if !legacy_missing && converted.is_legacy() && converted.has_missing_channel() {
            Color::for_space(
                converted.space,
                Some(converted.channel0()),
                Some(converted.channel1()),
                Some(converted.channel2()),
                Some(converted.alpha()),
            )
        } else {
            converted
        }
    }

    /// Replaces the alpha, keeping the space and channels. Missing
    /// channels read as `0` afterwards, as in Dart Sass's `changeAlpha`.
    pub(crate) fn change_alpha(&self, alpha: f64) -> Color {
        Color::for_space(
            self.space,
            Some(self.channel0()),
            Some(self.channel1()),
            Some(self.channel2()),
            Some(alpha),
        )
    }

    /// A channel as seen through a legacy space, for the legacy `red()`,
    /// `hue()`, etc. functions. The caller checks that the color is legacy.
    pub(crate) fn legacy_channel(&self, space: ColorSpace, index: usize) -> f64 {
        self.to_space(space, true).channel(index)
    }

    /// The legacy `mix()`: blends the rgb channels of two legacy colors,
    /// with `weight_scale` (`0..1`) the share of `self`. The result is
    /// always an rgb-space color. Adapted from
    /// <https://github.com/sass/dart-sass/blob/0d0270cb12a9ac5cce73a4d0785fecb00735feee/lib/src/functions/color.dart#L718>.
    pub(crate) fn mix_legacy(&self, other: &Color, weight_scale: f64) -> Color {
        let rgb1 = self.to_space(ColorSpace::Rgb, true);
        let rgb2 = other.to_space(ColorSpace::Rgb, true);
        let normalized_weight = weight_scale * 2.0 - 1.0;
        let alpha_distance = self.alpha() - other.alpha();

        let combined_weight1 = if normalized_weight * alpha_distance == -1.0 {
            normalized_weight
        } else {
            (normalized_weight + alpha_distance) / (1.0 + normalized_weight * alpha_distance)
        };
        let weight1 = (combined_weight1 + 1.0) / 2.0;
        let weight2 = 1.0 - weight1;

        Color::for_space(
            ColorSpace::Rgb,
            Some(rgb1.channel0() * weight1 + rgb2.channel0() * weight2),
            Some(rgb1.channel1() * weight1 + rgb2.channel1() * weight2),
            Some(rgb1.channel2() * weight1 + rgb2.channel2() * weight2),
            Some(rgb1.alpha() * weight_scale + rgb2.alpha() * (1.0 - weight_scale)),
        )
    }

    /// Interpolates between two colors in `space` according to the CSS
    /// Color 4 [color interpolation] procedure, with premultiplied alpha and
    /// the given hue method for polar spaces. `weight` is the share of
    /// `self` in the result. A missing channel on one side takes the other
    /// side's value; missing on both sides stays missing.
    ///
    /// The result is converted back to `self`'s space; `legacy_missing`
    /// says whether that conversion keeps missing channels.
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

        let channels1 = [
            color1.channels[0].or(color2.channels[0]),
            color1.channels[1].or(color2.channels[1]),
            color1.channels[2].or(color2.channels[2]),
        ];
        let channels2 = [
            color2.channels[0].or(color1.channels[0]),
            color2.channels[1].or(color1.channels[1]),
            color2.channels[2].or(color1.channels[2]),
        ];

        let alpha1 = self.alpha.unwrap_or_else(|| other.alpha());
        let alpha2 = other.alpha.unwrap_or_else(|| self.alpha());
        let this_multiplier = self.alpha.unwrap_or(1.0) * weight;
        let other_multiplier = other.alpha.unwrap_or(1.0) * (1.0 - weight);
        let mixed_alpha = if self.is_alpha_missing() && other.is_alpha_missing() {
            None
        } else {
            Some(alpha1 * weight + alpha2 * (1.0 - weight))
        };

        let mixed = |index: usize| {
            channels1[index].map(|channel1| {
                (channel1 * this_multiplier + channels2[index].unwrap_or(0.0) * other_multiplier)
                    / mixed_alpha.unwrap_or(1.0)
            })
        };
        let hue = |index: usize| {
            channels1[index].map(|hue1| {
                interpolate_hues(hue1, channels2[index].unwrap_or(0.0), hue_method, weight)
            })
        };

        let result = match space {
            ColorSpace::Hsl | ColorSpace::Hwb => {
                Color::for_space(space, hue(0), mixed(1), mixed(2), mixed_alpha)
            }
            ColorSpace::Lch | ColorSpace::Oklch => {
                Color::for_space(space, mixed(0), mixed(1), hue(2), mixed_alpha)
            }
            _ => Color::for_space(space, mixed(0), mixed(1), mixed(2), mixed_alpha),
        };

        result.to_space(self.space, legacy_missing)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_saturation_rotates_the_hue_on_construction() {
        let color = Color::for_space(
            ColorSpace::Hsl,
            Some(30.0),
            Some(-50.0),
            Some(50.0),
            Some(1.0),
        );
        assert_eq!(color.channels(), [210.0, 50.0, 50.0]);
    }

    #[test]
    fn hues_wrap_on_construction() {
        let hsl = Color::for_space(
            ColorSpace::Hsl,
            Some(-30.0),
            Some(50.0),
            Some(50.0),
            Some(1.0),
        );
        assert_eq!(hsl.channel0(), 330.0);

        let oklch = Color::for_space(
            ColorSpace::Oklch,
            Some(0.5),
            Some(0.1),
            Some(400.0),
            Some(1.0),
        );
        assert_eq!(oklch.channel2(), 40.0);
    }

    #[test]
    fn legacy_colors_compare_through_rgb() {
        let rgb = Color::for_space(
            ColorSpace::Rgb,
            Some(0.0),
            Some(255.0),
            Some(0.0),
            Some(1.0),
        );
        let hsl = Color::for_space(
            ColorSpace::Hsl,
            Some(120.0),
            Some(100.0),
            Some(50.0),
            Some(1.0),
        );
        assert_eq!(rgb, hsl);

        let srgb = Color::for_space(ColorSpace::Srgb, Some(0.0), Some(1.0), Some(0.0), Some(1.0));
        assert_ne!(rgb, srgb);
    }

    #[test]
    fn missing_channels_are_dropped_from_legacy_conversions_on_request() {
        let gray = Color::for_space(
            ColorSpace::Rgb,
            Some(128.0),
            Some(128.0),
            Some(128.0),
            Some(1.0),
        );
        assert!(gray.to_space(ColorSpace::Hsl, true).is_channel_missing(0));
        assert!(!gray.to_space(ColorSpace::Hsl, false).is_channel_missing(0));
        assert!(gray.to_space(ColorSpace::Lch, false).is_channel_missing(2));
    }
}
