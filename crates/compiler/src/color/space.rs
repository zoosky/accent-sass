//! The CSS Color 4 color spaces and the conversions between them.
//!
//! Every conversion here mirrors the corresponding routine in Dart Sass
//! (`lib/src/value/color/space/*.dart`) operation for operation, so channel
//! values match Dart Sass bit for bit. That matters because the serializer
//! prints channels to ten decimal places and a one-ulp difference in an
//! intermediate can flip the last digit.
//!
//! Conversions between two spaces go through a shared *linear* form: each
//! space knows how to make its channels linear-light ([`ColorSpace::to_linear`]),
//! a 3x3 matrix maps one linear space to another, and the destination
//! re-encodes the result ([`ColorSpace::from_linear`]). The polar and
//! perceptual spaces sit on top of a linear one: hsl and hwb on srgb, lab
//! and lch on xyz-d50, oklab and oklch on the internal lms space.
//!
//! A channel can be *missing* (`none` in CSS). Conversions carry a set of
//! "missing" flags along so that a missing channel maps to the analogous
//! missing channel in the destination: a missing hue stays missing across
//! polar spaces, a missing lightness stays missing across perceptual
//! spaces, and so on.

use std::f64::consts::PI;

use crate::value::{fuzzy_equals, fuzzy_greater_than_or_equals};

use super::{
    Color,
    channel::{ChannelKind, ColorChannel},
    conversions::*,
};

/// A CSS Color 4 color space, plus the internal `lms` space that oklab and
/// oklch are converted through. A [`Color`] is never in the `lms` space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ColorSpace {
    Rgb,
    Hsl,
    Hwb,
    Srgb,
    SrgbLinear,
    DisplayP3,
    DisplayP3Linear,
    A98Rgb,
    ProphotoRgb,
    Rec2020,
    XyzD65,
    XyzD50,
    Lab,
    Lch,
    Oklab,
    Oklch,
    Lms,
}

/// `29^3 / 3^3`, from the CIE Lab definition.
const LAB_KAPPA: f64 = 24389.0 / 27.0;

/// `6^3 / 29^3`, from the CIE Lab definition.
const LAB_EPSILON: f64 = 216.0 / 24389.0;

/// The channels of the `0..1` rgb-like spaces.
const RGB_CHANNELS: [ColorChannel; 3] = [
    ColorChannel::linear("red", 0.0, 1.0),
    ColorChannel::linear("green", 0.0, 1.0),
    ColorChannel::linear("blue", 0.0, 1.0),
];

/// The channels of the xyz spaces.
const XYZ_CHANNELS: [ColorChannel; 3] = [
    ColorChannel::linear("x", 0.0, 1.0),
    ColorChannel::linear("y", 0.0, 1.0),
    ColorChannel::linear("z", 0.0, 1.0),
];

/// A `0..255` legacy rgb channel, clamped at both ends when parsed.
const fn legacy_rgb_channel(name: &'static str) -> ColorChannel {
    ColorChannel {
        name,
        kind: ChannelKind::Linear {
            min: 0.0,
            max: 255.0,
            requires_percent: false,
            lower_clamped: true,
            upper_clamped: true,
            conventionally_percent: false,
        },
    }
}

/// A `0..100` channel that must be written as a percentage.
const fn percent_channel(name: &'static str, lower_clamped: bool) -> ColorChannel {
    ColorChannel {
        name,
        kind: ChannelKind::Linear {
            min: 0.0,
            max: 100.0,
            requires_percent: true,
            lower_clamped,
            upper_clamped: false,
            conventionally_percent: false,
        },
    }
}

/// A lightness channel clamped to `0..max`.
const fn lightness_channel(max: f64, conventionally_percent: bool) -> ColorChannel {
    ColorChannel {
        name: "lightness",
        kind: ChannelKind::Linear {
            min: 0.0,
            max,
            requires_percent: false,
            lower_clamped: true,
            upper_clamped: true,
            conventionally_percent,
        },
    }
}

/// A chroma channel clamped at zero.
const fn chroma_channel(max: f64) -> ColorChannel {
    ColorChannel {
        name: "chroma",
        kind: ChannelKind::Linear {
            min: 0.0,
            max,
            requires_percent: false,
            lower_clamped: true,
            upper_clamped: false,
            conventionally_percent: false,
        },
    }
}

/// Which of the perceptual channels are missing while a conversion is in
/// flight (Dart Sass's `missingLightness`/`missingChroma`/`missingHue`/
/// `missingA`/`missingB` parameters).
#[derive(Debug, Clone, Copy, Default)]
struct Missing {
    lightness: bool,
    chroma: bool,
    hue: bool,
    a: bool,
    b: bool,
}

impl Missing {
    /// A missing `a` and `b` together mean a missing chroma and hue, and
    /// vice versa.
    fn normalized(mut self) -> Missing {
        if self.a && self.b {
            self.chroma = true;
            self.hue = true;
        } else if self.chroma && self.hue {
            self.a = true;
            self.b = true;
        }
        self
    }

    /// Whether every perceptual channel is missing, which makes the whole
    /// converted color missing.
    fn all_perceptual(self) -> bool {
        self.lightness && self.chroma && self.hue
    }
}

/// Dart's `double.sign`: `1`, `-1`, or the number itself for zero and NaN.
pub(crate) fn sign(number: f64) -> f64 {
    if number > 0.0 {
        1.0
    } else if number < 0.0 {
        -1.0
    } else {
        number
    }
}

/// The sRGB and Display P3 transfer function, linear-light from
/// gamma-encoded.
pub(crate) fn srgb_to_linear(channel: f64) -> f64 {
    let abs = channel.abs();
    if abs <= 0.04045 {
        channel / 12.92
    } else {
        sign(channel) * ((abs + 0.055) / 1.055).powf(2.4)
    }
}

/// The sRGB and Display P3 transfer function, gamma-encoded from
/// linear-light.
pub(crate) fn srgb_from_linear(channel: f64) -> f64 {
    let abs = channel.abs();
    if abs <= 0.0031308 {
        channel * 12.92
    } else {
        sign(channel) * (1.055 * abs.powf(1.0 / 2.4) - 0.055)
    }
}

/// The CSS3 `HUE_TO_RGB` helper.
fn hue_to_rgb(m1: f64, m2: f64, mut hue: f64) -> f64 {
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

/// `pow(|x|, 1/3) * sign(x)`.
fn cube_root_preserving_sign(number: f64) -> f64 {
    number.abs().powf(1.0 / 3.0) * sign(number)
}

/// Dart's `%` on doubles, which is always non-negative for a positive
/// divisor.
pub(crate) fn dart_mod(number: f64, divisor: f64) -> f64 {
    number.rem_euclid(divisor)
}

impl ColorSpace {
    /// The lowercase name Sass uses for the space, as returned by
    /// `color.space()` and accepted by `$space` arguments.
    pub(crate) fn name(self) -> &'static str {
        match self {
            ColorSpace::Rgb => "rgb",
            ColorSpace::Hsl => "hsl",
            ColorSpace::Hwb => "hwb",
            ColorSpace::Srgb => "srgb",
            ColorSpace::SrgbLinear => "srgb-linear",
            ColorSpace::DisplayP3 => "display-p3",
            ColorSpace::DisplayP3Linear => "display-p3-linear",
            ColorSpace::A98Rgb => "a98-rgb",
            ColorSpace::ProphotoRgb => "prophoto-rgb",
            ColorSpace::Rec2020 => "rec2020",
            ColorSpace::XyzD65 => "xyz",
            ColorSpace::XyzD50 => "xyz-d50",
            ColorSpace::Lab => "lab",
            ColorSpace::Lch => "lch",
            ColorSpace::Oklab => "oklab",
            ColorSpace::Oklch => "oklch",
            ColorSpace::Lms => "lms",
        }
    }

    /// Looks up a space by name, case-insensitively, the way
    /// `ColorSpace.fromName` does in Dart Sass. `xyz` and `xyz-d65` are the
    /// same space.
    pub(crate) fn from_name(name: &str) -> Option<ColorSpace> {
        Some(match name.to_ascii_lowercase().as_str() {
            "rgb" => ColorSpace::Rgb,
            "hwb" => ColorSpace::Hwb,
            "hsl" => ColorSpace::Hsl,
            "srgb" => ColorSpace::Srgb,
            "srgb-linear" => ColorSpace::SrgbLinear,
            "display-p3" => ColorSpace::DisplayP3,
            "display-p3-linear" => ColorSpace::DisplayP3Linear,
            "a98-rgb" => ColorSpace::A98Rgb,
            "prophoto-rgb" => ColorSpace::ProphotoRgb,
            "rec2020" => ColorSpace::Rec2020,
            "xyz" | "xyz-d65" => ColorSpace::XyzD65,
            "xyz-d50" => ColorSpace::XyzD50,
            "lab" => ColorSpace::Lab,
            "lch" => ColorSpace::Lch,
            "oklab" => ColorSpace::Oklab,
            "oklch" => ColorSpace::Oklch,
            _ => return None,
        })
    }

    /// Whether the space is one of the three that Sass has always
    /// supported (rgb, hsl, hwb), which are interchangeable for the legacy
    /// functions and serialize in the legacy syntax.
    pub(crate) fn is_legacy(self) -> bool {
        matches!(self, ColorSpace::Rgb | ColorSpace::Hsl | ColorSpace::Hwb)
    }

    /// Whether the space has a hue channel.
    pub(crate) fn is_polar(self) -> bool {
        matches!(
            self,
            ColorSpace::Hsl | ColorSpace::Hwb | ColorSpace::Lch | ColorSpace::Oklch
        )
    }

    /// Whether the space has a gamut: a color outside its channel ranges
    /// cannot be displayed. The perceptual and xyz spaces are unbounded.
    pub(crate) fn is_bounded(self) -> bool {
        !matches!(
            self,
            ColorSpace::XyzD65
                | ColorSpace::XyzD50
                | ColorSpace::Lab
                | ColorSpace::Lch
                | ColorSpace::Oklab
                | ColorSpace::Oklch
                | ColorSpace::Lms
        )
    }

    /// The three channels of the space, in order.
    pub(crate) fn channels(self) -> [ColorChannel; 3] {
        match self {
            ColorSpace::Rgb => [
                legacy_rgb_channel("red"),
                legacy_rgb_channel("green"),
                legacy_rgb_channel("blue"),
            ],
            ColorSpace::Hsl => [
                ColorChannel::HUE,
                percent_channel("saturation", true),
                percent_channel("lightness", false),
            ],
            ColorSpace::Hwb => [
                ColorChannel::HUE,
                percent_channel("whiteness", false),
                percent_channel("blackness", false),
            ],
            ColorSpace::Srgb
            | ColorSpace::SrgbLinear
            | ColorSpace::DisplayP3
            | ColorSpace::DisplayP3Linear
            | ColorSpace::A98Rgb
            | ColorSpace::ProphotoRgb
            | ColorSpace::Rec2020 => RGB_CHANNELS,
            ColorSpace::XyzD65 | ColorSpace::XyzD50 => XYZ_CHANNELS,
            ColorSpace::Lab => [
                lightness_channel(100.0, false),
                ColorChannel::linear("a", -125.0, 125.0),
                ColorChannel::linear("b", -125.0, 125.0),
            ],
            ColorSpace::Lch => [
                lightness_channel(100.0, false),
                chroma_channel(150.0),
                ColorChannel::HUE,
            ],
            ColorSpace::Oklab => [
                lightness_channel(1.0, true),
                ColorChannel::linear("a", -0.4, 0.4),
                ColorChannel::linear("b", -0.4, 0.4),
            ],
            ColorSpace::Oklch => [
                lightness_channel(1.0, true),
                chroma_channel(0.4),
                ColorChannel::HUE,
            ],
            ColorSpace::Lms => [
                ColorChannel::linear("long", 0.0, 1.0),
                ColorChannel::linear("medium", 0.0, 1.0),
                ColorChannel::linear("short", 0.0, 1.0),
            ],
        }
    }

    /// The index of the channel called `name`, if the space has one. Names
    /// are case-sensitive, as in Dart Sass.
    pub(crate) fn channel_index(self, name: &str) -> Option<usize> {
        self.channels()
            .iter()
            .position(|channel| channel.name == name)
    }

    /// Converts a color's channels from this space to `dest`, following
    /// the `convert` method of the matching Dart Sass space class. Missing
    /// channels (`None`) propagate to their analogues in `dest`.
    ///
    /// The legacy result is not normalized for `legacy_missing`; see
    /// [`Color::to_space`] for that.
    pub(crate) fn convert(
        self,
        dest: ColorSpace,
        channel0: Option<f64>,
        channel1: Option<f64>,
        channel2: Option<f64>,
        alpha: Option<f64>,
    ) -> Color {
        match self {
            ColorSpace::Rgb => ColorSpace::Srgb.convert_srgb(
                dest,
                channel0.map(|red| red / 255.0),
                channel1.map(|green| green / 255.0),
                channel2.map(|blue| blue / 255.0),
                alpha,
                Missing::default(),
            ),
            ColorSpace::Hsl => {
                let scaled_hue = dart_mod(channel0.unwrap_or(0.0) / 360.0, 1.0);
                let scaled_saturation = channel1.unwrap_or(0.0) / 100.0;
                let scaled_lightness = channel2.unwrap_or(0.0) / 100.0;

                let m2 = if scaled_lightness <= 0.5 {
                    scaled_lightness * (scaled_saturation + 1.0)
                } else {
                    scaled_lightness + scaled_saturation - scaled_lightness * scaled_saturation
                };
                let m1 = scaled_lightness * 2.0 - m2;

                ColorSpace::Srgb.convert_srgb(
                    dest,
                    Some(hue_to_rgb(m1, m2, scaled_hue + 1.0 / 3.0)),
                    Some(hue_to_rgb(m1, m2, scaled_hue)),
                    Some(hue_to_rgb(m1, m2, scaled_hue - 1.0 / 3.0)),
                    alpha,
                    Missing {
                        lightness: channel2.is_none(),
                        chroma: channel1.is_none(),
                        hue: channel0.is_none(),
                        ..Missing::default()
                    },
                )
            }
            ColorSpace::Hwb => self.convert_hwb(dest, channel0, channel1, channel2, alpha),
            ColorSpace::Srgb => self.convert_srgb(
                dest,
                channel0,
                channel1,
                channel2,
                alpha,
                Missing::default(),
            ),
            ColorSpace::SrgbLinear => match dest {
                ColorSpace::Rgb | ColorSpace::Hsl | ColorSpace::Hwb | ColorSpace::Srgb => {
                    ColorSpace::Srgb.convert(
                        dest,
                        channel0.map(srgb_from_linear),
                        channel1.map(srgb_from_linear),
                        channel2.map(srgb_from_linear),
                        alpha,
                    )
                }
                _ => self.convert_linear(
                    dest,
                    channel0,
                    channel1,
                    channel2,
                    alpha,
                    Missing::default(),
                ),
            },
            ColorSpace::DisplayP3 if dest == ColorSpace::DisplayP3Linear => Color::for_space(
                dest,
                channel0.map(srgb_to_linear),
                channel1.map(srgb_to_linear),
                channel2.map(srgb_to_linear),
                alpha,
            ),
            ColorSpace::DisplayP3Linear if dest == ColorSpace::DisplayP3 => Color::for_space(
                dest,
                channel0.map(srgb_from_linear),
                channel1.map(srgb_from_linear),
                channel2.map(srgb_from_linear),
                alpha,
            ),
            ColorSpace::DisplayP3
            | ColorSpace::DisplayP3Linear
            | ColorSpace::A98Rgb
            | ColorSpace::ProphotoRgb
            | ColorSpace::Rec2020
            | ColorSpace::XyzD65 => self.convert_linear(
                dest,
                channel0,
                channel1,
                channel2,
                alpha,
                Missing::default(),
            ),
            ColorSpace::XyzD50 => self.convert_xyz_d50(
                dest,
                channel0,
                channel1,
                channel2,
                alpha,
                Missing::default(),
            ),
            ColorSpace::Lab => {
                self.convert_lab(dest, channel0, channel1, channel2, alpha, false, false)
            }
            ColorSpace::Lch => {
                let hue_radians = channel2.unwrap_or(0.0) * PI / 180.0;
                ColorSpace::Lab.convert_lab(
                    dest,
                    channel0,
                    Some(channel1.unwrap_or(0.0) * hue_radians.cos()),
                    Some(channel1.unwrap_or(0.0) * hue_radians.sin()),
                    alpha,
                    channel1.is_none(),
                    channel2.is_none(),
                )
            }
            ColorSpace::Oklab => {
                self.convert_oklab(dest, channel0, channel1, channel2, alpha, false, false)
            }
            ColorSpace::Oklch => {
                let hue_radians = channel2.unwrap_or(0.0) * PI / 180.0;
                ColorSpace::Oklab.convert_oklab(
                    dest,
                    channel0,
                    Some(channel1.unwrap_or(0.0) * hue_radians.cos()),
                    Some(channel1.unwrap_or(0.0) * hue_radians.sin()),
                    alpha,
                    channel1.is_none(),
                    channel2.is_none(),
                )
            }
            ColorSpace::Lms => self.convert_lms(
                dest,
                channel0,
                channel1,
                channel2,
                alpha,
                Missing::default(),
            ),
        }
    }

    /// `HwbColorSpace.convert`. When both whiteness and blackness are
    /// missing the color is converted as the pure hue, and only the hue's
    /// analogue survives in a polar destination.
    fn convert_hwb(
        self,
        dest: ColorSpace,
        hue: Option<f64>,
        whiteness: Option<f64>,
        blackness: Option<f64>,
        alpha: Option<f64>,
    ) -> Color {
        if whiteness.is_none() && blackness.is_none() {
            if hue.is_none() {
                return Color::for_space(dest, None, None, None, alpha);
            }
            let converted = self.convert(dest, hue, Some(0.0), Some(0.0), alpha);
            return match dest {
                ColorSpace::Hsl => Color::for_space(
                    dest,
                    Some(converted.channel0()),
                    None,
                    None,
                    Some(converted.alpha()),
                ),
                ColorSpace::Lch | ColorSpace::Oklch => Color::for_space(
                    dest,
                    None,
                    None,
                    Some(converted.channel2()),
                    Some(converted.alpha()),
                ),
                _ => converted,
            };
        }

        let scaled_hue = dart_mod(hue.unwrap_or(0.0), 360.0) / 360.0;
        let mut scaled_whiteness = whiteness.unwrap_or(0.0) / 100.0;
        let mut scaled_blackness = blackness.unwrap_or(0.0) / 100.0;

        let sum = scaled_whiteness + scaled_blackness;
        if sum > 1.0 {
            scaled_whiteness /= sum;
            scaled_blackness /= sum;
        }

        let factor = 1.0 - scaled_whiteness - scaled_blackness;
        let to_rgb = |hue: f64| hue_to_rgb(0.0, 1.0, hue) * factor + scaled_whiteness;

        ColorSpace::Srgb.convert_srgb(
            dest,
            Some(to_rgb(scaled_hue + 1.0 / 3.0)),
            Some(to_rgb(scaled_hue)),
            Some(to_rgb(scaled_hue - 1.0 / 3.0)),
            alpha,
            Missing {
                hue: hue.is_none(),
                ..Missing::default()
            },
        )
    }

    /// `SrgbColorSpace.convert`: unit-scale rgb to any space, with the
    /// hsl and hwb formulas from CSS Color 4 inlined.
    fn convert_srgb(
        self,
        dest: ColorSpace,
        red: Option<f64>,
        green: Option<f64>,
        blue: Option<f64>,
        alpha: Option<f64>,
        missing: Missing,
    ) -> Color {
        if (red.is_none() && green.is_none() && blue.is_none()) || missing.all_perceptual() {
            return Color::for_space(dest, None, None, None, alpha);
        }

        match dest {
            ColorSpace::Hsl | ColorSpace::Hwb => {
                let red = red.unwrap_or(0.0);
                let green = green.unwrap_or(0.0);
                let blue = blue.unwrap_or(0.0);
                let max = red.max(green).max(blue);
                let min = red.min(green).min(blue);
                let delta = max - min;

                let mut hue = if max == min {
                    0.0
                } else if max == red {
                    60.0 * (green - blue) / delta + 360.0
                } else if max == green {
                    60.0 * (blue - red) / delta + 120.0
                } else {
                    60.0 * (red - green) / delta + 240.0
                };

                if dest == ColorSpace::Hsl {
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

                    Color::for_space(
                        dest,
                        if missing.hue || fuzzy_equals(saturation, 0.0) {
                            None
                        } else {
                            Some(dart_mod(hue, 360.0))
                        },
                        if missing.chroma {
                            None
                        } else {
                            Some(saturation)
                        },
                        if missing.lightness {
                            None
                        } else {
                            Some(lightness * 100.0)
                        },
                        alpha,
                    )
                } else {
                    let whiteness = min * 100.0;
                    let blackness = 100.0 - max * 100.0;
                    let achromatic = missing.chroma && missing.lightness;

                    Color::for_space(
                        dest,
                        if missing.hue || fuzzy_greater_than_or_equals(whiteness + blackness, 100.0)
                        {
                            None
                        } else {
                            Some(dart_mod(hue, 360.0))
                        },
                        if achromatic { None } else { Some(whiteness) },
                        if achromatic { None } else { Some(blackness) },
                        alpha,
                    )
                }
            }
            ColorSpace::Rgb => Color::for_space(
                dest,
                red.map(|red| red * 255.0),
                green.map(|green| green * 255.0),
                blue.map(|blue| blue * 255.0),
                alpha,
            ),
            ColorSpace::SrgbLinear => Color::for_space(
                dest,
                red.map(srgb_to_linear),
                green.map(srgb_to_linear),
                blue.map(srgb_to_linear),
                alpha,
            ),
            _ => self.convert_linear(dest, red, green, blue, alpha, missing),
        }
    }

    /// `ColorSpace.convertLinear`: the general path through the linear
    /// form and a transformation matrix.
    fn convert_linear(
        self,
        dest: ColorSpace,
        red: Option<f64>,
        green: Option<f64>,
        blue: Option<f64>,
        alpha: Option<f64>,
        missing: Missing,
    ) -> Color {
        let missing = missing.normalized();

        if missing.all_perceptual() || (red.is_none() && green.is_none() && blue.is_none()) {
            return Color::for_space(dest, None, None, None, alpha);
        }

        let linear_dest = match dest {
            ColorSpace::Hsl | ColorSpace::Hwb => ColorSpace::Srgb,
            ColorSpace::Lab | ColorSpace::Lch => ColorSpace::XyzD50,
            ColorSpace::Oklab | ColorSpace::Oklch => ColorSpace::Lms,
            _ => dest,
        };

        let (transformed_red, transformed_green, transformed_blue) = if linear_dest == self {
            (red, green, blue)
        } else {
            let linear_red = self.to_linear(red.unwrap_or(0.0));
            let linear_green = self.to_linear(green.unwrap_or(0.0));
            let linear_blue = self.to_linear(blue.unwrap_or(0.0));
            let matrix = self.transformation_matrix(linear_dest);
            (
                Some(linear_dest.from_linear(
                    matrix[0] * linear_red + matrix[1] * linear_green + matrix[2] * linear_blue,
                )),
                Some(linear_dest.from_linear(
                    matrix[3] * linear_red + matrix[4] * linear_green + matrix[5] * linear_blue,
                )),
                Some(linear_dest.from_linear(
                    matrix[6] * linear_red + matrix[7] * linear_green + matrix[8] * linear_blue,
                )),
            )
        };

        match dest {
            ColorSpace::Hsl | ColorSpace::Hwb => ColorSpace::Srgb.convert_srgb(
                dest,
                transformed_red,
                transformed_green,
                transformed_blue,
                alpha,
                Missing {
                    a: false,
                    b: false,
                    ..missing
                },
            ),
            ColorSpace::Lab | ColorSpace::Lch => ColorSpace::XyzD50.convert_xyz_d50(
                dest,
                transformed_red,
                transformed_green,
                transformed_blue,
                alpha,
                missing,
            ),
            ColorSpace::Oklab | ColorSpace::Oklch => ColorSpace::Lms.convert_lms(
                dest,
                transformed_red,
                transformed_green,
                transformed_blue,
                alpha,
                missing,
            ),
            _ => Color::for_space(
                dest,
                red.and(transformed_red),
                green.and(transformed_green),
                blue.and(transformed_blue),
                alpha,
            ),
        }
    }

    /// `XyzD50ColorSpace.convert`, which is where lab and lch are computed.
    fn convert_xyz_d50(
        self,
        dest: ColorSpace,
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
        alpha: Option<f64>,
        missing: Missing,
    ) -> Color {
        let missing = missing.normalized();

        if missing.all_perceptual() || (x.is_none() && y.is_none() && z.is_none()) {
            return Color::for_space(dest, None, None, None, alpha);
        }

        match dest {
            ColorSpace::Lab | ColorSpace::Lch => {
                let f0 = component_to_lab_f(x.unwrap_or(0.0) / D50[0]);
                let f1 = component_to_lab_f(y.unwrap_or(0.0) / D50[1]);
                let f2 = component_to_lab_f(z.unwrap_or(0.0) / D50[2]);
                let lightness = if missing.lightness {
                    None
                } else {
                    Some((116.0 * f1) - 16.0)
                };
                let a = 500.0 * (f0 - f1);
                let b = 200.0 * (f1 - f2);

                if dest == ColorSpace::Lab {
                    Color::for_space(
                        dest,
                        lightness,
                        if missing.a { None } else { Some(a) },
                        if missing.b { None } else { Some(b) },
                        alpha,
                    )
                } else {
                    lab_to_lch(
                        dest,
                        lightness,
                        Some(a),
                        Some(b),
                        alpha,
                        missing.chroma,
                        missing.hue,
                    )
                }
            }
            _ => self.convert_linear(dest, x, y, z, alpha, missing),
        }
    }

    /// `LmsColorSpace.convert`, which is where oklab and oklch are computed.
    fn convert_lms(
        self,
        dest: ColorSpace,
        long: Option<f64>,
        medium: Option<f64>,
        short: Option<f64>,
        alpha: Option<f64>,
        missing: Missing,
    ) -> Color {
        let missing = missing.normalized();

        if missing.all_perceptual() || (long.is_none() && medium.is_none() && short.is_none()) {
            return Color::for_space(dest, None, None, None, alpha);
        }

        match dest {
            ColorSpace::Oklab | ColorSpace::Oklch => {
                let long_scaled = cube_root_preserving_sign(long.unwrap_or(0.0));
                let medium_scaled = cube_root_preserving_sign(medium.unwrap_or(0.0));
                let short_scaled = cube_root_preserving_sign(short.unwrap_or(0.0));

                let lightness = LMS_TO_OKLAB[0] * long_scaled
                    + LMS_TO_OKLAB[1] * medium_scaled
                    + LMS_TO_OKLAB[2] * short_scaled;
                let a = LMS_TO_OKLAB[3] * long_scaled
                    + LMS_TO_OKLAB[4] * medium_scaled
                    + LMS_TO_OKLAB[5] * short_scaled;
                let b = LMS_TO_OKLAB[6] * long_scaled
                    + LMS_TO_OKLAB[7] * medium_scaled
                    + LMS_TO_OKLAB[8] * short_scaled;
                let lightness = if missing.lightness {
                    None
                } else {
                    Some(lightness)
                };

                if dest == ColorSpace::Oklab {
                    Color::for_space(
                        dest,
                        lightness,
                        if missing.a { None } else { Some(a) },
                        if missing.b { None } else { Some(b) },
                        alpha,
                    )
                } else {
                    lab_to_lch(
                        dest,
                        lightness,
                        Some(a),
                        Some(b),
                        alpha,
                        missing.chroma,
                        missing.hue,
                    )
                }
            }
            _ => self.convert_linear(dest, long, medium, short, alpha, missing),
        }
    }

    /// `LabColorSpace.convert`.
    #[allow(clippy::too_many_arguments)]
    fn convert_lab(
        self,
        dest: ColorSpace,
        lightness: Option<f64>,
        mut a: Option<f64>,
        mut b: Option<f64>,
        alpha: Option<f64>,
        mut missing_chroma: bool,
        mut missing_hue: bool,
    ) -> Color {
        if missing_chroma && missing_hue {
            a = None;
            b = None;
        } else if a.is_none() && b.is_none() {
            missing_chroma = true;
            missing_hue = true;
        }

        match dest {
            ColorSpace::Lab => {
                let powerless_ab = match lightness {
                    Some(lightness) => fuzzy_equals(lightness, 0.0),
                    None => true,
                };
                Color::for_space(
                    dest,
                    lightness,
                    if powerless_ab { None } else { a },
                    if powerless_ab { None } else { b },
                    alpha,
                )
            }
            ColorSpace::Lch => lab_to_lch(dest, lightness, a, b, alpha, false, false),
            _ => {
                let missing_lightness = lightness.is_none();
                let lightness = lightness.unwrap_or(0.0);
                let f1 = (lightness + 16.0) / 116.0;
                let y = if lightness > LAB_KAPPA * LAB_EPSILON {
                    ((lightness + 16.0) / 116.0).powf(3.0) * 1.0
                } else {
                    lightness / LAB_KAPPA
                };

                ColorSpace::XyzD50.convert_xyz_d50(
                    dest,
                    Some(lab_f_to_x_or_z(a.unwrap_or(0.0) / 500.0 + f1) * D50[0]),
                    Some(y * D50[1]),
                    Some(lab_f_to_x_or_z(f1 - b.unwrap_or(0.0) / 200.0) * D50[2]),
                    alpha,
                    Missing {
                        lightness: missing_lightness,
                        chroma: missing_chroma,
                        hue: missing_hue,
                        a: a.is_none(),
                        b: b.is_none(),
                    },
                )
            }
        }
    }

    /// `OklabColorSpace.convert`.
    #[allow(clippy::too_many_arguments)]
    fn convert_oklab(
        self,
        dest: ColorSpace,
        lightness: Option<f64>,
        mut a: Option<f64>,
        mut b: Option<f64>,
        alpha: Option<f64>,
        mut missing_chroma: bool,
        mut missing_hue: bool,
    ) -> Color {
        if dest == ColorSpace::Oklch {
            return lab_to_lch(dest, lightness, a, b, alpha, missing_chroma, missing_hue);
        }

        if a.is_none() && b.is_none() {
            missing_chroma = true;
            missing_hue = true;
        } else if missing_chroma && missing_hue {
            a = None;
            b = None;
        }

        let missing = Missing {
            lightness: lightness.is_none(),
            chroma: missing_chroma,
            hue: missing_hue,
            a: a.is_none(),
            b: b.is_none(),
        };
        let lightness = lightness.unwrap_or(0.0);
        let a = a.unwrap_or(0.0);
        let b = b.unwrap_or(0.0);

        ColorSpace::Lms.convert_lms(
            dest,
            Some(
                (OKLAB_TO_LMS[0] * lightness + OKLAB_TO_LMS[1] * a + OKLAB_TO_LMS[2] * b).powf(3.0)
                    + 0.0,
            ),
            Some(
                (OKLAB_TO_LMS[3] * lightness + OKLAB_TO_LMS[4] * a + OKLAB_TO_LMS[5] * b).powf(3.0)
                    + 0.0,
            ),
            Some(
                (OKLAB_TO_LMS[6] * lightness + OKLAB_TO_LMS[7] * a + OKLAB_TO_LMS[8] * b).powf(3.0)
                    + 0.0,
            ),
            alpha,
            missing,
        )
    }

    /// Makes a channel linear-light. Only the rgb-like and xyz spaces
    /// take part in linear conversions.
    fn to_linear(self, channel: f64) -> f64 {
        match self {
            ColorSpace::Rgb => srgb_to_linear(channel / 255.0),
            ColorSpace::Srgb | ColorSpace::DisplayP3 => srgb_to_linear(channel),
            ColorSpace::A98Rgb => sign(channel) * channel.abs().powf(563.0 / 256.0),
            ColorSpace::ProphotoRgb => {
                let abs = channel.abs();
                if abs <= 16.0 / 512.0 {
                    channel / 16.0
                } else {
                    sign(channel) * abs.powf(1.8)
                }
            }
            ColorSpace::Rec2020 => sign(channel) * channel.abs().powf(2.40),
            ColorSpace::SrgbLinear
            | ColorSpace::DisplayP3Linear
            | ColorSpace::XyzD65
            | ColorSpace::XyzD50
            | ColorSpace::Lms => channel,
            _ => unreachable!("color space {} has no linear form", self.name()),
        }
    }

    /// Re-encodes a linear-light channel in this space.
    fn from_linear(self, channel: f64) -> f64 {
        match self {
            ColorSpace::Rgb => srgb_from_linear(channel) * 255.0,
            ColorSpace::Srgb | ColorSpace::DisplayP3 => srgb_from_linear(channel),
            ColorSpace::A98Rgb => sign(channel) * channel.abs().powf(256.0 / 563.0),
            ColorSpace::ProphotoRgb => {
                let abs = channel.abs();
                if abs >= 1.0 / 512.0 {
                    sign(channel) * abs.powf(1.0 / 1.8)
                } else {
                    16.0 * channel
                }
            }
            ColorSpace::Rec2020 => sign(channel) * channel.abs().powf(1.0 / 2.40),
            ColorSpace::SrgbLinear
            | ColorSpace::DisplayP3Linear
            | ColorSpace::XyzD65
            | ColorSpace::XyzD50
            | ColorSpace::Lms => channel,
            _ => unreachable!("color space {} has no linear form", self.name()),
        }
    }

    /// The matrix that maps this space's linear form onto `dest`'s.
    fn transformation_matrix(self, dest: ColorSpace) -> &'static [f64; 9] {
        use ColorSpace::*;

        match (self, dest) {
            (Srgb | SrgbLinear, DisplayP3 | DisplayP3Linear) => &LINEAR_SRGB_TO_LINEAR_DISPLAY_P3,
            (Srgb | SrgbLinear, A98Rgb) => &LINEAR_SRGB_TO_LINEAR_A98_RGB,
            (Srgb | SrgbLinear, ProphotoRgb) => &LINEAR_SRGB_TO_LINEAR_PROPHOTO_RGB,
            (Srgb | SrgbLinear, Rec2020) => &LINEAR_SRGB_TO_LINEAR_REC2020,
            (Srgb | SrgbLinear, XyzD65) => &LINEAR_SRGB_TO_XYZ_D65,
            (Srgb | SrgbLinear, XyzD50) => &LINEAR_SRGB_TO_XYZ_D50,
            (Srgb | SrgbLinear, Lms) => &LINEAR_SRGB_TO_LMS,

            (DisplayP3 | DisplayP3Linear, SrgbLinear | Srgb | Rgb) => {
                &LINEAR_DISPLAY_P3_TO_LINEAR_SRGB
            }
            (DisplayP3 | DisplayP3Linear, A98Rgb) => &LINEAR_DISPLAY_P3_TO_LINEAR_A98_RGB,
            (DisplayP3 | DisplayP3Linear, ProphotoRgb) => &LINEAR_DISPLAY_P3_TO_LINEAR_PROPHOTO_RGB,
            (DisplayP3 | DisplayP3Linear, Rec2020) => &LINEAR_DISPLAY_P3_TO_LINEAR_REC2020,
            (DisplayP3 | DisplayP3Linear, XyzD65) => &LINEAR_DISPLAY_P3_TO_XYZ_D65,
            (DisplayP3 | DisplayP3Linear, XyzD50) => &LINEAR_DISPLAY_P3_TO_XYZ_D50,
            (DisplayP3 | DisplayP3Linear, Lms) => &LINEAR_DISPLAY_P3_TO_LMS,

            (A98Rgb, SrgbLinear | Srgb | Rgb) => &LINEAR_A98_RGB_TO_LINEAR_SRGB,
            (A98Rgb, DisplayP3 | DisplayP3Linear) => &LINEAR_A98_RGB_TO_LINEAR_DISPLAY_P3,
            (A98Rgb, ProphotoRgb) => &LINEAR_A98_RGB_TO_LINEAR_PROPHOTO_RGB,
            (A98Rgb, Rec2020) => &LINEAR_A98_RGB_TO_LINEAR_REC2020,
            (A98Rgb, XyzD65) => &LINEAR_A98_RGB_TO_XYZ_D65,
            (A98Rgb, XyzD50) => &LINEAR_A98_RGB_TO_XYZ_D50,
            (A98Rgb, Lms) => &LINEAR_A98_RGB_TO_LMS,

            (ProphotoRgb, SrgbLinear | Srgb | Rgb) => &LINEAR_PROPHOTO_RGB_TO_LINEAR_SRGB,
            (ProphotoRgb, A98Rgb) => &LINEAR_PROPHOTO_RGB_TO_LINEAR_A98_RGB,
            (ProphotoRgb, DisplayP3 | DisplayP3Linear) => &LINEAR_PROPHOTO_RGB_TO_LINEAR_DISPLAY_P3,
            (ProphotoRgb, Rec2020) => &LINEAR_PROPHOTO_RGB_TO_LINEAR_REC2020,
            (ProphotoRgb, XyzD65) => &LINEAR_PROPHOTO_RGB_TO_XYZ_D65,
            (ProphotoRgb, XyzD50) => &LINEAR_PROPHOTO_RGB_TO_XYZ_D50,
            (ProphotoRgb, Lms) => &LINEAR_PROPHOTO_RGB_TO_LMS,

            (Rec2020, SrgbLinear | Srgb | Rgb) => &LINEAR_REC2020_TO_LINEAR_SRGB,
            (Rec2020, A98Rgb) => &LINEAR_REC2020_TO_LINEAR_A98_RGB,
            (Rec2020, DisplayP3 | DisplayP3Linear) => &LINEAR_REC2020_TO_LINEAR_DISPLAY_P3,
            (Rec2020, ProphotoRgb) => &LINEAR_REC2020_TO_LINEAR_PROPHOTO_RGB,
            (Rec2020, XyzD65) => &LINEAR_REC2020_TO_XYZ_D65,
            (Rec2020, XyzD50) => &LINEAR_REC2020_TO_XYZ_D50,
            (Rec2020, Lms) => &LINEAR_REC2020_TO_LMS,

            (XyzD65, SrgbLinear | Srgb | Rgb) => &XYZ_D65_TO_LINEAR_SRGB,
            (XyzD65, A98Rgb) => &XYZ_D65_TO_LINEAR_A98_RGB,
            (XyzD65, ProphotoRgb) => &XYZ_D65_TO_LINEAR_PROPHOTO_RGB,
            (XyzD65, DisplayP3 | DisplayP3Linear) => &XYZ_D65_TO_LINEAR_DISPLAY_P3,
            (XyzD65, Rec2020) => &XYZ_D65_TO_LINEAR_REC2020,
            (XyzD65, XyzD50) => &XYZ_D65_TO_XYZ_D50,
            (XyzD65, Lms) => &XYZ_D65_TO_LMS,

            (XyzD50, SrgbLinear | Srgb | Rgb) => &XYZ_D50_TO_LINEAR_SRGB,
            (XyzD50, A98Rgb) => &XYZ_D50_TO_LINEAR_A98_RGB,
            (XyzD50, ProphotoRgb) => &XYZ_D50_TO_LINEAR_PROPHOTO_RGB,
            (XyzD50, DisplayP3 | DisplayP3Linear) => &XYZ_D50_TO_LINEAR_DISPLAY_P3,
            (XyzD50, Rec2020) => &XYZ_D50_TO_LINEAR_REC2020,
            (XyzD50, XyzD65) => &XYZ_D50_TO_XYZ_D65,
            (XyzD50, Lms) => &XYZ_D50_TO_LMS,

            (Lms, SrgbLinear | Srgb | Rgb) => &LMS_TO_LINEAR_SRGB,
            (Lms, A98Rgb) => &LMS_TO_LINEAR_A98_RGB,
            (Lms, ProphotoRgb) => &LMS_TO_LINEAR_PROPHOTO_RGB,
            (Lms, DisplayP3 | DisplayP3Linear) => &LMS_TO_LINEAR_DISPLAY_P3,
            (Lms, Rec2020) => &LMS_TO_LINEAR_REC2020,
            (Lms, XyzD65) => &LMS_TO_XYZ_D65,
            (Lms, XyzD50) => &LMS_TO_XYZ_D50,

            _ => unreachable!(
                "color space conversion from {} to {} not implemented",
                self.name(),
                dest.name()
            ),
        }
    }
}

/// `XyzD50ColorSpace._convertComponentToLabF`.
fn component_to_lab_f(component: f64) -> f64 {
    if component > LAB_EPSILON {
        component.powf(1.0 / 3.0) + 0.0
    } else {
        (LAB_KAPPA * component + 16.0) / 116.0
    }
}

/// `LabColorSpace._convertFToXorZ`.
fn lab_f_to_x_or_z(component: f64) -> f64 {
    let cubed = component.powf(3.0) + 0.0;
    if cubed > LAB_EPSILON {
        cubed
    } else {
        (116.0 * component - 16.0) / LAB_KAPPA
    }
}

/// Builds an lch or oklch color from lab-style channels (Dart Sass's
/// `labToLch`). The hue is missing when the chroma is (fuzzily) zero,
/// since CSS Color 4 calls it powerless then.
#[allow(clippy::too_many_arguments)]
fn lab_to_lch(
    dest: ColorSpace,
    lightness: Option<f64>,
    a: Option<f64>,
    b: Option<f64>,
    alpha: Option<f64>,
    missing_chroma: bool,
    missing_hue: bool,
) -> Color {
    let missing_chroma = missing_chroma || (a.is_none() && b.is_none());
    let missing_hue = missing_hue || (a.is_none() && b.is_none());

    let chroma = (a.unwrap_or(0.0).powf(2.0) + b.unwrap_or(0.0).powf(2.0)).sqrt();
    let hue = if missing_hue || fuzzy_equals(chroma, 0.0) {
        None
    } else {
        Some(b.unwrap_or(0.0).atan2(a.unwrap_or(0.0)) * 180.0 / PI)
    };

    Color::for_space(
        dest,
        lightness,
        if missing_chroma { None } else { Some(chroma) },
        hue.map(|hue| if hue >= 0.0 { hue } else { hue + 360.0 }),
        alpha,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn channels(color: &Color) -> [f64; 3] {
        color.channels()
    }

    #[test]
    fn hsl_round_trips_pure_green() {
        let rgb = ColorSpace::Hsl.convert(
            ColorSpace::Rgb,
            Some(120.0),
            Some(100.0),
            Some(50.0),
            Some(1.0),
        );
        assert_eq!(channels(&rgb), [0.0, 255.0, 0.0]);

        let hsl = ColorSpace::Rgb.convert(
            ColorSpace::Hsl,
            Some(0.0),
            Some(255.0),
            Some(0.0),
            Some(1.0),
        );
        assert_eq!(channels(&hsl), [120.0, 100.0, 50.0]);
    }

    #[test]
    fn gray_has_a_missing_hue_in_the_polar_spaces() {
        let gray =
            |dest| ColorSpace::Rgb.convert(dest, Some(128.0), Some(128.0), Some(128.0), Some(1.0));
        assert!(gray(ColorSpace::Hsl).is_channel_missing(0));
        assert!(gray(ColorSpace::Hwb).is_channel_missing(0));
        assert!(gray(ColorSpace::Lch).is_channel_missing(2));
        assert!(gray(ColorSpace::Oklch).is_channel_missing(2));
    }

    #[test]
    fn negative_saturation_rotates_the_hue() {
        // `color.change(#cc0f35, $red: -5)` in Dart Sass:
        // hsl(219.3103448276, 120.8333333333%, 9.4117647059%)
        let hsl = ColorSpace::Rgb.convert(
            ColorSpace::Hsl,
            Some(-5.0),
            Some(15.0),
            Some(53.0),
            Some(1.0),
        );
        let [hue, saturation, _] = channels(&hsl);
        assert!((hue - 219.3103448276).abs() < 1e-9);
        assert!((saturation - 120.8333333333).abs() < 1e-9);
    }

    #[test]
    fn lab_matches_dart_sass() {
        // color.to-space(#cc0f35, lab)
        //   => lab(44.2229117293% 67.6217073989 34.5537259027)
        let lab = ColorSpace::Rgb.convert(
            ColorSpace::Lab,
            Some(204.0),
            Some(15.0),
            Some(53.0),
            Some(1.0),
        );
        let [lightness, a, b] = channels(&lab);
        assert!((lightness - 44.2229117293).abs() < 1e-9);
        assert!((a - 67.6217073989).abs() < 1e-9);
        assert!((b - 34.5537259027).abs() < 1e-9);
    }

    #[test]
    fn oklch_matches_dart_sass() {
        // color.to-space(#cc0f35, oklch)
        //   => oklch(53.8574934869% 0.210710041 20.5019425917deg)
        let oklch = ColorSpace::Rgb.convert(
            ColorSpace::Oklch,
            Some(204.0),
            Some(15.0),
            Some(53.0),
            Some(1.0),
        );
        let [lightness, chroma, hue] = channels(&oklch);
        assert!((lightness - 0.538574934869).abs() < 1e-9);
        assert!((chroma - 0.210710041).abs() < 1e-9);
        assert!((hue - 20.5019425917).abs() < 1e-9);
    }

    #[test]
    fn display_p3_matches_dart_sass() {
        // color.to-space(#cc0f35, display-p3)
        //   => color(display-p3 0.7336902375 0.1705609368 0.2295679504)
        let p3 = ColorSpace::Rgb.convert(
            ColorSpace::DisplayP3,
            Some(204.0),
            Some(15.0),
            Some(53.0),
            Some(1.0),
        );
        let [red, green, blue] = channels(&p3);
        assert!((red - 0.7336902375).abs() < 1e-9);
        assert!((green - 0.1705609368).abs() < 1e-9);
        assert!((blue - 0.2295679504).abs() < 1e-9);
    }

    #[test]
    fn missing_channels_propagate_to_their_analogues() {
        // color.to-space(lab(none 10 20), oklch)
        //   => oklch(none 0.4252336568 13.7521936893deg)
        let oklch =
            ColorSpace::Lab.convert(ColorSpace::Oklch, None, Some(10.0), Some(20.0), Some(1.0));
        assert!(oklch.is_channel_missing(0));
        assert!(!oklch.is_channel_missing(1));
        assert!(!oklch.is_channel_missing(2));

        // color.to-space(hsl(20 50% none), lch) => lch(none 0 none)
        let lch = ColorSpace::Hsl.convert(ColorSpace::Lch, Some(20.0), Some(50.0), None, Some(1.0));
        assert!(lch.is_channel_missing(0));
        assert!(!lch.is_channel_missing(1));
        assert!(lch.is_channel_missing(2));
        assert_eq!(lch.channel1(), 0.0);
    }

    #[test]
    fn dart_sign_of_zero_is_zero() {
        assert_eq!(sign(0.0), 0.0);
        assert_eq!(sign(-3.0), -1.0);
        assert_eq!(sign(2.5), 1.0);
        assert!(sign(f64::NAN).is_nan());
    }
}
