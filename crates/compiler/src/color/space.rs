//! The three legacy color spaces (`rgb`, `hsl`, and `hwb`) and the
//! conversions between them.
//!
//! Every conversion here mirrors the corresponding routine in Dart Sass
//! (`lib/src/value/color/space/{rgb,srgb,hsl,hwb,utils}.dart`) operation for
//! operation, so channel values match Dart Sass bit for bit. That matters
//! because the serializer prints channels to ten decimal places and a
//! one-ulp difference in an intermediate can flip the last digit.
//!
//! Channel conventions: rgb channels are unit-scale (`0..1`) inside this
//! module, hsl saturation/lightness and hwb whiteness/blackness are percent
//! values (`0..100`), and hues are degrees.

use crate::value::fuzzy_equals;

/// One of the legacy color spaces that Sass keeps interchangeable for
/// backwards compatibility.
///
/// Dart Sass 1.79+ tracks which of these a color was written in (or last
/// converted to). The space decides how the color serializes, what
/// `color.space()` reports, and which channels `color.channel()` and
/// `color.adjust()` accept without an explicit `$space`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColorSpace {
    Rgb,
    Hsl,
    Hwb,
}

impl ColorSpace {
    /// The lowercase name Sass uses for the space, as returned by
    /// `color.space()` and accepted by `$space` arguments.
    pub(crate) fn name(self) -> &'static str {
        match self {
            ColorSpace::Rgb => "rgb",
            ColorSpace::Hsl => "hsl",
            ColorSpace::Hwb => "hwb",
        }
    }

    /// Whether the space has a hue channel.
    pub(crate) fn is_polar(self) -> bool {
        !matches!(self, ColorSpace::Rgb)
    }

    /// The names of the three channels, in order.
    pub(crate) fn channel_names(self) -> [&'static str; 3] {
        match self {
            ColorSpace::Rgb => ["red", "green", "blue"],
            ColorSpace::Hsl => ["hue", "saturation", "lightness"],
            ColorSpace::Hwb => ["hue", "whiteness", "blackness"],
        }
    }

    /// The upper bound of the linear channels, used by gamut checks and
    /// `color.scale()`. Hue is polar and has no bound.
    pub(crate) fn channel_max(self) -> f64 {
        match self {
            ColorSpace::Rgb => 255.0,
            ColorSpace::Hsl | ColorSpace::Hwb => 100.0,
        }
    }

    /// Looks up a legacy space by name, case-insensitively, the way
    /// `ColorSpace.fromName` does in Dart Sass.
    pub(crate) fn from_name(name: &str) -> Option<ColorSpace> {
        match name.to_ascii_lowercase().as_str() {
            "rgb" => Some(ColorSpace::Rgb),
            "hsl" => Some(ColorSpace::Hsl),
            "hwb" => Some(ColorSpace::Hwb),
            _ => None,
        }
    }
}

/// The hue-bearing channels of an hsl or hwb color computed from rgb.
///
/// `achromatic` reports whether Dart Sass would consider the hue
/// [powerless] for this conversion (zero saturation, or whiteness and
/// blackness summing to at least 100%), which is when a converted color gets
/// a *missing* hue.
///
/// [powerless]: https://www.w3.org/TR/css-color-4/#powerless
pub(crate) struct PolarChannels {
    pub hue: f64,
    pub channel1: f64,
    pub channel2: f64,
    pub achromatic: bool,
}

impl PolarChannels {
    /// The hue as a color reads it: `0` when the hue is powerless, since
    /// Dart Sass stores a missing hue and reports it as zero.
    pub(crate) fn hue_or_zero(&self) -> f64 {
        if self.achromatic {
            0.0
        } else {
            self.hue
        }
    }
}

/// Dart Sass's hue normalization from `SassColor.forSpaceInternal`:
/// `(hue % 360 + 360) % 360` with Dart's non-negative `%`.
pub(crate) fn normalize_hue(hue: f64) -> f64 {
    (hue.rem_euclid(360.0) + 360.0).rem_euclid(360.0)
}

/// The shared rgb-to-hue computation from Dart Sass's `SrgbColorSpace.convert`
/// for the hsl and hwb destinations. Inputs are unit-scale rgb.
fn rgb_hue(red: f64, green: f64, blue: f64, max: f64, min: f64) -> f64 {
    let delta = max - min;

    if max == min {
        0.0
    } else if max == red {
        60.0 * (green - blue) / delta + 360.0
    } else if max == green {
        60.0 * (blue - red) / delta + 120.0
    } else {
        60.0 * (red - green) / delta + 240.0
    }
}

/// Converts unit-scale rgb to hsl, following
/// <https://drafts.csswg.org/css-color-4/#rgb-to-hsl> as Dart Sass writes it.
///
/// Saturation and lightness come back as percentages. An out-of-gamut input
/// can produce a negative saturation, which Dart Sass folds into the hue by
/// rotating it 180 degrees, so this returns the same channels Dart Sass
/// stores for the converted color.
pub(crate) fn rgb_to_hsl(red: f64, green: f64, blue: f64) -> PolarChannels {
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);

    let mut hue = rgb_hue(red, green, blue, max, min);

    let lightness = (min + max) / 2.0;
    let mut saturation = if lightness == 0.0 || lightness == 1.0 {
        0.0
    } else {
        100.0 * (max - lightness) / lightness.min(1.0 - lightness)
    };

    if saturation < 0.0 {
        hue += 180.0;
        saturation = saturation.abs();
    }

    PolarChannels {
        hue: hue.rem_euclid(360.0),
        channel1: saturation,
        channel2: lightness * 100.0,
        achromatic: fuzzy_equals(saturation, 0.0),
    }
}

/// Converts unit-scale rgb to hwb. Whiteness and blackness come back as
/// percentages.
pub(crate) fn rgb_to_hwb(red: f64, green: f64, blue: f64) -> PolarChannels {
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);

    let hue = rgb_hue(red, green, blue, max, min);
    let whiteness = min * 100.0;
    let blackness = 100.0 - max * 100.0;

    PolarChannels {
        hue: hue.rem_euclid(360.0),
        channel1: whiteness,
        channel2: blackness,
        achromatic: whiteness + blackness > 100.0 || fuzzy_equals(whiteness + blackness, 100.0),
    }
}

/// The CSS3 `HUE_TO_RGB` helper, as in Dart Sass's `utils.dart`.
pub(crate) fn hue_to_rgb(m1: f64, m2: f64, mut hue: f64) -> f64 {
    if hue < 0.0 {
        hue += 1.0;
    }
    if hue > 1.0 {
        hue -= 1.0;
    }

    if hue < 1.0 / 6.0 {
        m1 + (m2 - m1) * hue * 6.0
    } else if hue < 1.0 / 2.0 {
        m2
    } else if hue < 2.0 / 3.0 {
        m1 + (m2 - m1) * (2.0 / 3.0 - hue) * 6.0
    } else {
        m1
    }
}

/// Converts hsl (degrees, percent, percent) to unit-scale rgb, following
/// <https://www.w3.org/TR/css3-color/#hsl-color> with Dart Sass's exact
/// operation order (plain multiply-add sequences, no fused operations).
pub(crate) fn hsl_to_rgb(hue: f64, saturation: f64, lightness: f64) -> (f64, f64, f64) {
    let scaled_hue = (hue / 360.0).rem_euclid(1.0);
    let scaled_saturation = saturation / 100.0;
    let scaled_lightness = lightness / 100.0;

    let m2 = if scaled_lightness <= 0.5 {
        scaled_lightness * (scaled_saturation + 1.0)
    } else {
        scaled_lightness + scaled_saturation - scaled_lightness * scaled_saturation
    };
    let m1 = scaled_lightness * 2.0 - m2;

    (
        hue_to_rgb(m1, m2, scaled_hue + 1.0 / 3.0),
        hue_to_rgb(m1, m2, scaled_hue),
        hue_to_rgb(m1, m2, scaled_hue - 1.0 / 3.0),
    )
}

/// Converts hwb (degrees, percent, percent) to unit-scale rgb, following
/// <https://www.w3.org/TR/css-color-4/#hwb-to-rgb>. Whiteness and blackness
/// that sum to more than 100% are scaled down proportionally, as the spec
/// requires.
pub(crate) fn hwb_to_rgb(hue: f64, whiteness: f64, blackness: f64) -> (f64, f64, f64) {
    let scaled_hue = hue.rem_euclid(360.0) / 360.0;
    let mut scaled_whiteness = whiteness / 100.0;
    let mut scaled_blackness = blackness / 100.0;

    let sum = scaled_whiteness + scaled_blackness;
    if sum > 1.0 {
        scaled_whiteness /= sum;
        scaled_blackness /= sum;
    }

    let factor = 1.0 - scaled_whiteness - scaled_blackness;
    let to_rgb = |hue: f64| hue_to_rgb(0.0, 1.0, hue) * factor + scaled_whiteness;

    (
        to_rgb(scaled_hue + 1.0 / 3.0),
        to_rgb(scaled_hue),
        to_rgb(scaled_hue - 1.0 / 3.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsl_round_trips_pure_green() {
        let (r, g, b) = hsl_to_rgb(120.0, 100.0, 50.0);
        assert_eq!((r, g, b), (0.0, 1.0, 0.0));
        let hsl = rgb_to_hsl(r, g, b);
        assert_eq!(hsl.hue, 120.0);
        assert_eq!(hsl.channel1, 100.0);
        assert_eq!(hsl.channel2, 50.0);
        assert!(!hsl.achromatic);
    }

    #[test]
    fn gray_is_achromatic_in_both_polar_spaces() {
        let hsl = rgb_to_hsl(0.5, 0.5, 0.5);
        assert!(hsl.achromatic);
        assert_eq!(hsl.hue, 0.0);

        let hwb = rgb_to_hwb(0.5, 0.5, 0.5);
        assert!(hwb.achromatic);
        assert_eq!(hwb.channel1, 50.0);
        assert_eq!(hwb.channel2, 50.0);
    }

    #[test]
    fn negative_saturation_rotates_the_hue() {
        // `color.change(#cc0f35, $red: -5)` in Dart Sass:
        // hsl(219.3103448276, 120.8333333333%, 9.4117647059%)
        let hsl = rgb_to_hsl(-5.0 / 255.0, 15.0 / 255.0, 53.0 / 255.0);
        assert!((hsl.hue - 219.3103448276).abs() < 1e-9);
        assert!((hsl.channel1 - 120.8333333333).abs() < 1e-9);
    }

    #[test]
    fn normalize_hue_wraps_negative_values() {
        assert_eq!(normalize_hue(-30.0), 330.0);
        assert_eq!(normalize_hue(480.0), 120.0);
        assert_eq!(normalize_hue(360.0), 0.0);
    }
}
