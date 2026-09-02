//! Gamut checks and gamut mapping.
//!
//! `color.is-in-gamut()` and `color.to-gamut()` decide whether a color fits
//! its space's bounds and, if not, pull it back inside. The `clip` method
//! clamps each channel; `local-minde` follows the [CSS Color 4 gamut
//! mapping algorithm], which searches for the most saturated in-gamut color
//! that is perceptually indistinguishable (in OKLab) from the input.
//!
//! Both are transcribed from Dart Sass
//! (`lib/src/value/color/gamut_map_method/{clip,local_minde}.dart`).
//!
//! [CSS Color 4 gamut mapping algorithm]: https://www.w3.org/TR/2022/CRD-css-color-4-20221101/#css-gamut-mapping-algorithm

use crate::value::{fuzzy_greater_than_or_equals, fuzzy_less_than_or_equals};

use super::{ChannelKind, Color, ColorSpace};

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

/// `clampLikeCss`: NaN clamps to the lower bound.
pub(crate) fn clamp_like_css(number: f64, lower: f64, upper: f64) -> f64 {
    if number.is_nan() {
        lower
    } else {
        number.clamp(lower, upper)
    }
}

impl Color {
    /// Whether every channel of the color's own space is within its
    /// bounds, with fuzzy comparison at the edges. An unbounded space
    /// (lab, lch, oklab, oklch, xyz) holds every color.
    pub(crate) fn is_in_gamut(&self) -> bool {
        if !self.space.is_bounded() {
            return true;
        }

        self.space
            .channels()
            .iter()
            .zip(self.channels().iter())
            .all(|(channel, value)| match channel.kind {
                ChannelKind::Linear { min, max, .. } => {
                    fuzzy_less_than_or_equals(*value, max)
                        && fuzzy_greater_than_or_equals(*value, min)
                }
                ChannelKind::PolarAngle => true,
            })
    }

    /// Returns the color unchanged when it is in gamut, or maps it into
    /// gamut with `method`.
    pub(crate) fn to_gamut(&self, method: GamutMapMethod) -> Color {
        if self.is_in_gamut() {
            return self.clone();
        }

        match method {
            GamutMapMethod::Clip => self.clip(),
            GamutMapMethod::LocalMinde => self.local_minde(),
        }
    }

    /// The `clip` gamut map: clamps every linear channel of the color's
    /// own space. Missing channels stay missing; hues are unbounded.
    fn clip(&self) -> Color {
        let channels = self.space.channels();
        let clamp = |index: usize| {
            self.channels_or_none()[index].map(|value| match channels[index].kind {
                ChannelKind::Linear { min, max, .. } => clamp_like_css(value, min, max),
                ChannelKind::PolarAngle => value,
            })
        };

        Color::for_space(
            self.space,
            clamp(0),
            clamp(1),
            clamp(2),
            self.alpha_or_none(),
        )
    }

    /// The CSS Color 4 local MINDE gamut map, as Dart Sass implements it:
    /// a binary search on OKLCH chroma for the most saturated color whose
    /// clipped version is within the just-noticeable difference.
    fn local_minde(&self) -> Color {
        let origin_oklch = self.to_space(ColorSpace::Oklch, true);
        let lightness = origin_oklch.channels_or_none()[0];
        let hue = origin_oklch.channels_or_none()[2];
        let alpha = origin_oklch.alpha_or_none();

        if fuzzy_greater_than_or_equals(lightness.unwrap_or(0.0), 1.0) {
            return if self.is_legacy() {
                Color::for_space(
                    ColorSpace::Rgb,
                    Some(255.0),
                    Some(255.0),
                    Some(255.0),
                    self.alpha_or_none(),
                )
                .to_space(self.space, true)
            } else {
                Color::for_space(
                    self.space,
                    Some(1.0),
                    Some(1.0),
                    Some(1.0),
                    self.alpha_or_none(),
                )
            };
        } else if fuzzy_less_than_or_equals(lightness.unwrap_or(0.0), 0.0) {
            return Color::for_space(
                ColorSpace::Rgb,
                Some(0.0),
                Some(0.0),
                Some(0.0),
                self.alpha_or_none(),
            )
            .to_space(self.space, true);
        }

        let mut clipped = self.to_gamut(GamutMapMethod::Clip);
        if delta_e_ok(&clipped, self) < JND {
            return clipped;
        }

        let mut min = 0.0;
        let mut max = origin_oklch.channel1();
        let mut min_in_gamut = true;

        while max - min > EPSILON {
            let chroma = (min + max) / 2.0;
            let current =
                ColorSpace::Oklch.convert(self.space, lightness, Some(chroma), hue, alpha);

            // Per https://github.com/w3c/csswg-drafts/issues/10226#issuecomment-2065534713
            // the search falls through here once `min_in_gamut` is false
            // without re-checking `current.is_in_gamut()`.
            if min_in_gamut && current.is_in_gamut() {
                min = chroma;
                continue;
            }

            clipped = current.to_gamut(GamutMapMethod::Clip);
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

/// The OKLab color difference, `sqrt(dL^2 + da^2 + db^2)`.
fn delta_e_ok(color1: &Color, color2: &Color) -> f64 {
    let lab1 = color1.to_space(ColorSpace::Oklab, true);
    let lab2 = color2.to_space(ColorSpace::Oklab, true);

    ((lab1.channel0() - lab2.channel0()).powf(2.0)
        + (lab1.channel1() - lab2.channel1()).powf(2.0)
        + (lab1.channel2() - lab2.channel2()).powf(2.0))
    .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unbounded_spaces_hold_every_color() {
        let lab = Color::for_space(
            ColorSpace::Lab,
            Some(150.0),
            Some(300.0),
            Some(-300.0),
            Some(1.0),
        );
        assert!(lab.is_in_gamut());
        assert_eq!(
            lab.to_gamut(GamutMapMethod::LocalMinde).channels(),
            lab.channels()
        );
    }

    #[test]
    fn clip_clamps_each_channel_to_its_range() {
        let srgb = Color::for_space(
            ColorSpace::Srgb,
            Some(1.2),
            Some(-0.1),
            Some(0.5),
            Some(1.0),
        );
        assert!(!srgb.is_in_gamut());
        assert_eq!(
            srgb.to_gamut(GamutMapMethod::Clip).channels(),
            [1.0, 0.0, 0.5]
        );
    }

    #[test]
    fn local_minde_matches_dart_sass_for_display_p3_red() {
        // color.to-gamut(color(display-p3 1 0 0), rgb, $method: local-minde)
        //   => color(display-p3 0.9177905633 0.2107213818 0.1542354933)
        let p3_red = Color::for_space(
            ColorSpace::DisplayP3,
            Some(1.0),
            Some(0.0),
            Some(0.0),
            Some(1.0),
        );
        let mapped = p3_red
            .to_space(ColorSpace::Rgb, true)
            .to_gamut(GamutMapMethod::LocalMinde)
            .to_space(ColorSpace::DisplayP3, false);
        let [red, green, blue] = mapped.channels();
        assert!((red - 0.9177905633).abs() < 1e-9);
        assert!((green - 0.2107213818).abs() < 1e-9);
        assert!((blue - 0.1542354933).abs() < 1e-9);
    }
}
