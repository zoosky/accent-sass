//! The metadata of a single color channel: its name, its range, and how
//! Sass treats values written for it.
//!
//! This mirrors `ColorChannel` and `LinearChannel` in Dart Sass
//! (`lib/src/value/color/channel.dart`). The flags decide how `lab()`,
//! `color.change()` and friends interpret a channel argument: whether a
//! percentage is required, what a percentage scales to, and whether the
//! parsed value is clamped at either end of the range.

use crate::unit::Unit;

/// How a channel's values are interpreted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ChannelKind {
    /// A channel with a numeric range, such as `red` or `lightness`.
    Linear {
        /// The lower bound of the range.
        min: f64,
        /// The upper bound of the range. A percentage argument scales to it.
        max: f64,
        /// Whether a stylesheet must write the channel as a percentage
        /// (hsl saturation and lightness, hwb whiteness and blackness).
        requires_percent: bool,
        /// Whether a parsed value below `min` is clamped to `min`.
        lower_clamped: bool,
        /// Whether a parsed value above `max` is clamped to `max`.
        upper_clamped: bool,
        /// Whether the channel is conventionally written as a percentage
        /// even though its range is not `0..100` (oklab lightness).
        conventionally_percent: bool,
    },
    /// A hue, measured in degrees and wrapped to `0..360`.
    PolarAngle,
}

/// One channel of a color space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ColorChannel {
    /// The name `color.channel()` and the keyword arguments of
    /// `color.change()` use for the channel.
    pub name: &'static str,
    pub kind: ChannelKind,
}

impl ColorChannel {
    /// A linear channel with no clamping and no percentage requirement.
    pub(crate) const fn linear(name: &'static str, min: f64, max: f64) -> ColorChannel {
        ColorChannel {
            name,
            kind: ChannelKind::Linear {
                min,
                max,
                requires_percent: false,
                lower_clamped: false,
                upper_clamped: false,
                conventionally_percent: false,
            },
        }
    }

    /// The hue channel shared by every polar space.
    pub(crate) const HUE: ColorChannel = ColorChannel {
        name: "hue",
        kind: ChannelKind::PolarAngle,
    };

    /// The alpha channel, which every space shares.
    pub(crate) const ALPHA: ColorChannel = ColorChannel::linear("alpha", 0.0, 1.0);

    /// Whether the channel is a hue.
    pub(crate) fn is_polar_angle(&self) -> bool {
        matches!(self.kind, ChannelKind::PolarAngle)
    }

    /// The unit `color.channel()` reports the channel with: `deg` for a
    /// hue, `%` for a channel that is conventionally a percentage or whose
    /// range is `0..100`, and no unit otherwise.
    pub(crate) fn associated_unit(&self) -> Unit {
        match self.kind {
            ChannelKind::PolarAngle => Unit::Deg,
            ChannelKind::Linear {
                min,
                max,
                conventionally_percent,
                ..
            } => {
                if conventionally_percent || (min == 0.0 && max == 100.0) {
                    Unit::Percent
                } else {
                    Unit::None
                }
            }
        }
    }
}
