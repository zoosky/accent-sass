//! Gamut checks and gamut mapping for legacy colors.
//!
//! `color.is-in-gamut()` and `color.to-gamut()` decide whether a color fits
//! its own space's bounds and, if not, pull it back inside. The `clip`
//! method clamps each channel; `local-minde` follows the [CSS Color 4 gamut
//! mapping algorithm], which searches for the most saturated in-gamut color
//! that is perceptually indistinguishable (in OKLab) from the input.
//!
//! The OKLab/OKLCH round trip and the matrices are transcribed from Dart
//! Sass (`lib/src/value/color/{conversions,space/lms,space/oklab,
//! space/oklch,gamut_map_method/local_minde}.dart`) so results match it.
//!
//! [CSS Color 4 gamut mapping algorithm]: https://www.w3.org/TR/2022/CRD-css-color-4-20221101/#css-gamut-mapping-algorithm

use std::f64::consts::PI;

use crate::value::{fuzzy_equals, fuzzy_greater_than_or_equals, fuzzy_less_than_or_equals};

use super::{Color, ColorFormat, ColorSpace, Number};

/// How `color.to-gamut()` brings an out-of-gamut color back in range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GamutMapMethod {
    /// Clamp every channel to its bounds.
    Clip,
    /// The CSS Color 4 "local MINDE" search in OKLCH.
    LocalMinde,
}

impl GamutMapMethod {
    /// Parses the (case-sensitive) method name Dart Sass accepts.
    pub(crate) fn from_name(name: &str) -> Option<GamutMapMethod> {
        match name {
            "clip" => Some(GamutMapMethod::Clip),
            "local-minde" => Some(GamutMapMethod::LocalMinde),
            _ => None,
        }
    }
}

/// The just-noticeable difference in OKLab used by local MINDE.
const JND: f64 = 0.02;

/// The chroma resolution at which the local MINDE search stops.
const EPSILON: f64 = 0.0001;

const LMS_TO_OKLAB: [f64; 9] = [
    0.210454268309314,
    0.7936177747023054,
    -0.0040720430116193,
    1.9779985324311684,
    -2.42859224204858,
    0.450593709617411,
    0.0259040424655478,
    0.7827717124575296,
    -0.8086757549230774,
];

const OKLAB_TO_LMS: [f64; 9] = [
    1.0000000000000002,
    0.3963377773761749,
    0.2158037573099136,
    0.9999999999999998,
    -0.10556134581565854,
    -0.06385417282581334,
    0.9999999999999999,
    -0.0894841775298118,
    -1.2914855480194094,
];

const LINEAR_SRGB_TO_LMS: [f64; 9] = [
    0.412221469470763,
    0.5363325372617348,
    0.0514459932675022,
    0.2119034958178252,
    0.6806995506452342,
    0.1073969535369405,
    0.08830245919005641,
    0.2817188391361215,
    0.6299787016738221,
];

const LMS_TO_LINEAR_SRGB: [f64; 9] = [
    4.076741636075958,
    -3.307711539258062,
    0.23096990318210417,
    -1.268437973285032,
    2.609757349287689,
    -0.3413193760026571,
    -0.00419607613867551,
    -0.7034186179359363,
    1.707614694074612,
];

/// Multiplies a 3x3 row-major matrix by a column vector, in the exact
/// operation order Dart Sass uses (`m0 * a + m1 * b + m2 * c`).
fn multiply(matrix: &[f64; 9], a: f64, b: f64, c: f64) -> [f64; 3] {
    [
        matrix[0] * a + matrix[1] * b + matrix[2] * c,
        matrix[3] * a + matrix[4] * b + matrix[5] * c,
        matrix[6] * a + matrix[7] * b + matrix[8] * c,
    ]
}

/// The sRGB transfer function, linear-light from gamma-encoded.
fn srgb_to_linear(channel: f64) -> f64 {
    let abs = channel.abs();
    if abs <= 0.04045 {
        channel / 12.92
    } else {
        channel.signum() * ((abs + 0.055) / 1.055).powf(2.4)
    }
}

/// The sRGB transfer function, gamma-encoded from linear-light.
fn srgb_from_linear(channel: f64) -> f64 {
    let abs = channel.abs();
    if abs <= 0.0031308 {
        channel * 12.92
    } else {
        channel.signum() * (1.055 * abs.powf(1.0 / 2.4) - 0.055)
    }
}

/// `pow(|x|, 1/3) * sign(x)`, where the sign of zero is zero as in Dart.
fn cube_root_preserving_sign(number: f64) -> f64 {
    let root = number.abs().powf(1.0 / 3.0);
    if number > 0.0 {
        root
    } else if number < 0.0 {
        -root
    } else {
        number
    }
}

/// `clampLikeCss`: NaN clamps to the lower bound.
pub(crate) fn clamp_like_css(number: f64, lower: f64, upper: f64) -> f64 {
    if number.is_nan() {
        lower
    } else {
        number.clamp(lower, upper)
    }
}

/// Converts unit-scale rgb to OKLab `[lightness, a, b]`.
fn rgb_to_oklab(red: f64, green: f64, blue: f64) -> [f64; 3] {
    let lms = multiply(
        &LINEAR_SRGB_TO_LMS,
        srgb_to_linear(red),
        srgb_to_linear(green),
        srgb_to_linear(blue),
    );

    multiply(
        &LMS_TO_OKLAB,
        cube_root_preserving_sign(lms[0]),
        cube_root_preserving_sign(lms[1]),
        cube_root_preserving_sign(lms[2]),
    )
}

/// An OKLCH color. The hue is `None` when the chroma is (fuzzily) zero,
/// which is when CSS Color 4 calls it powerless.
struct Oklch {
    lightness: f64,
    chroma: f64,
    hue: Option<f64>,
}

/// Converts unit-scale rgb to OKLCH (Dart Sass's `labToLch`).
fn rgb_to_oklch(red: f64, green: f64, blue: f64) -> Oklch {
    let [lightness, a, b] = rgb_to_oklab(red, green, blue);

    let chroma = (a.powi(2) + b.powi(2)).sqrt();
    let hue = if fuzzy_equals(chroma, 0.0) {
        None
    } else {
        let hue = b.atan2(a) * 180.0 / PI;
        Some(if hue >= 0.0 { hue } else { hue + 360.0 })
    };

    Oklch {
        lightness,
        chroma,
        hue,
    }
}

/// Converts OKLCH to unit-scale rgb through OKLab and LMS.
fn oklch_to_rgb(lightness: f64, chroma: f64, hue: Option<f64>) -> (f64, f64, f64) {
    let hue_radians = hue.unwrap_or(0.0) * PI / 180.0;
    let a = chroma * hue_radians.cos();
    let b = chroma * hue_radians.sin();

    let lms = multiply(&OKLAB_TO_LMS, lightness, a, b);
    let linear = multiply(
        &LMS_TO_LINEAR_SRGB,
        lms[0].powf(3.0) + 0.0,
        lms[1].powf(3.0) + 0.0,
        lms[2].powf(3.0) + 0.0,
    );

    (
        srgb_from_linear(linear[0]),
        srgb_from_linear(linear[1]),
        srgb_from_linear(linear[2]),
    )
}

/// The OKLab color difference, `sqrt(dL^2 + da^2 + db^2)`.
fn delta_e_ok(color1: &Color, color2: &Color) -> f64 {
    let (r1, g1, b1) = color1.unit_rgb();
    let (r2, g2, b2) = color2.unit_rgb();
    let lab1 = rgb_to_oklab(r1, g1, b1);
    let lab2 = rgb_to_oklab(r2, g2, b2);

    ((lab1[0] - lab2[0]).powi(2) + (lab1[1] - lab2[1]).powi(2) + (lab1[2] - lab2[2]).powi(2)).sqrt()
}

impl Color {
    /// Whether every bounded channel of the color's own space is within its
    /// bounds, with fuzzy comparison at the edges.
    ///
    /// The check is per space: rgb channels must be within `0..255`, hsl
    /// saturation and lightness within `0%..100%`, and hwb whiteness and
    /// blackness within `0%..100%`. Hue is unbounded.
    pub(crate) fn is_in_gamut(&self) -> bool {
        let in_range = |value: f64, max: f64| {
            fuzzy_less_than_or_equals(value, max) && fuzzy_greater_than_or_equals(value, 0.0)
        };

        let (channel1, channel2) = match self.space {
            ColorSpace::Rgb => {
                return in_range(self.red().0, 255.0)
                    && in_range(self.green().0, 255.0)
                    && in_range(self.blue().0, 255.0);
            }
            ColorSpace::Hsl => {
                let (_, saturation, lightness) = self.hsl_view();
                (saturation, lightness)
            }
            ColorSpace::Hwb => {
                let (_, whiteness, blackness) = self.hwb_view();
                (whiteness, blackness)
            }
        };

        in_range(channel1, 100.0) && in_range(channel2, 100.0)
    }

    /// The `clip` gamut map: clamps every bounded channel of the color's own
    /// space. A missing hue stays missing.
    pub(crate) fn clip_to_gamut(&self) -> Color {
        let alpha = self.alpha();
        match self.space {
            ColorSpace::Rgb => Color::new_rgba(
                Number(clamp_like_css(self.red().0, 0.0, 255.0)),
                Number(clamp_like_css(self.green().0, 0.0, 255.0)),
                Number(clamp_like_css(self.blue().0, 0.0, 255.0)),
                alpha,
                ColorFormat::Infer,
            ),
            ColorSpace::Hsl => {
                let (hue, saturation, lightness) = self.hsl_view();
                Color::from_hsla(
                    Number(hue),
                    Number(clamp_like_css(saturation, 0.0, 100.0)),
                    Number(clamp_like_css(lightness, 0.0, 100.0)),
                    alpha,
                )
                .with_missing_hue(self.missing_hue)
            }
            ColorSpace::Hwb => {
                let (hue, whiteness, blackness) = self.hwb_view();
                Color::from_hwb(
                    Number(hue),
                    Number(clamp_like_css(whiteness, 0.0, 100.0)),
                    Number(clamp_like_css(blackness, 0.0, 100.0)),
                    alpha,
                )
                .with_missing_hue(self.missing_hue)
            }
        }
    }

    /// Returns the color unchanged when it is in gamut, or maps it into
    /// gamut with `method`.
    pub(crate) fn to_gamut(&self, method: GamutMapMethod) -> Color {
        if self.is_in_gamut() {
            return self.clone();
        }

        match method {
            GamutMapMethod::Clip => self.clip_to_gamut(),
            GamutMapMethod::LocalMinde => self.to_gamut_local_minde(),
        }
    }

    /// Builds a color in `space` from an OKLCH triple, the way
    /// `ColorSpace.oklch.convert(space, ..)` does. Converting into a polar
    /// space marks the hue missing when the result is achromatic.
    fn from_oklch_in_space(
        space: ColorSpace,
        lightness: f64,
        chroma: f64,
        hue: Option<f64>,
        alpha: Number,
    ) -> Color {
        let (red, green, blue) = oklch_to_rgb(lightness, chroma, hue);
        Color::from_unit_rgb_in_space(space, red, green, blue, alpha, true, false)
    }

    /// The CSS Color 4 local MINDE gamut map, as Dart Sass implements it:
    /// a binary search on OKLCH chroma for the most saturated color whose
    /// clipped version is within the just-noticeable difference.
    fn to_gamut_local_minde(&self) -> Color {
        let (red, green, blue) = self.unit_rgb();
        let origin = rgb_to_oklch(red, green, blue);
        let alpha = self.alpha();

        if fuzzy_greater_than_or_equals(origin.lightness, 1.0) {
            return Color::new_rgba(
                Number(255.0),
                Number(255.0),
                Number(255.0),
                alpha,
                ColorFormat::Infer,
            )
            .to_space(self.space, true);
        } else if fuzzy_less_than_or_equals(origin.lightness, 0.0) {
            return Color::new_rgba(
                Number(0.0),
                Number(0.0),
                Number(0.0),
                alpha,
                ColorFormat::Infer,
            )
            .to_space(self.space, true);
        }

        let mut clipped = self.clip_to_gamut();
        if delta_e_ok(&clipped, self) < JND {
            return clipped;
        }

        let mut min = 0.0;
        let mut max = origin.chroma;
        let mut min_in_gamut = true;

        while max - min > EPSILON {
            let chroma = (min + max) / 2.0;
            let current =
                Color::from_oklch_in_space(self.space, origin.lightness, chroma, origin.hue, alpha);

            // Per https://github.com/w3c/csswg-drafts/issues/10226#issuecomment-2065534713
            // the search falls through here once `min_in_gamut` is false
            // without re-checking `current.is_in_gamut()`.
            if min_in_gamut && current.is_in_gamut() {
                min = chroma;
                continue;
            }

            clipped = current.clip_to_gamut();
            let difference = delta_e_ok(&clipped, &current);
            if difference < JND {
                if JND - difference < EPSILON {
                    return clipped;
                }
                min_in_gamut = false;
                min = chroma;
            } else {
                max = chroma;
            }
        }

        clipped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oklch_round_trips_a_saturated_red() {
        // Dart Sass: color.to-space(#cc0f35, oklch)
        //   => oklch(53.8574934869% 0.210710041 20.5019425917deg)
        let oklch = rgb_to_oklch(204.0 / 255.0, 15.0 / 255.0, 53.0 / 255.0);
        assert!((oklch.lightness - 0.538_574_934_869).abs() < 1e-9);
        assert!((oklch.chroma - 0.210_710_041).abs() < 1e-9);
        assert!((oklch.hue.unwrap() - 20.501_942_591_7).abs() < 1e-9);

        let (red, green, blue) = oklch_to_rgb(oklch.lightness, oklch.chroma, oklch.hue);
        assert!((red * 255.0 - 204.0).abs() < 1e-9);
        assert!((green * 255.0 - 15.0).abs() < 1e-9);
        assert!((blue * 255.0 - 53.0).abs() < 1e-9);
    }

    #[test]
    fn gray_has_no_oklch_hue() {
        assert!(rgb_to_oklch(0.5, 0.5, 0.5).hue.is_none());
    }
}
