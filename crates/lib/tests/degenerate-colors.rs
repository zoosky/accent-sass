//! Degenerate color channels, which dart-sass normalizes from 1.104.0 on.
//!
//! A channel that is `NaN` or negative zero becomes `0`, and so does a hue
//! that is infinite, as CSS Color 4 specifies. dart-sass does this in
//! `SassColor.forSpaceInternal`, which every color function and conversion
//! goes through except the legacy rgb constructor: `color.change()` on an rgb
//! color keeps a `NaN` or `-0` channel until the color is converted to
//! another space.
//!
//! Every expectation here was compared against the dart-sass 1.104.0 binary
//! before being committed.

#[macro_use]
mod macros;

test!(
    nan_hue_becomes_zero,
    "@use \"sass:math\";\na {\n  b: hsl(math.div(0, 0), 50%, 50%);\n}\n",
    "a {\n  b: hsl(0, 50%, 50%);\n}\n"
);
test!(
    infinite_hue_becomes_zero,
    "@use \"sass:math\";\na {\n  b: hsl(math.div(1, 0), 50%, 50%);\n}\n",
    "a {\n  b: hsl(0, 50%, 50%);\n}\n"
);
test!(
    nan_lch_hue_becomes_zero,
    "@use \"sass:math\";\na {\n  b: lch(50% 30 math.div(0, 0));\n}\n",
    "a {\n  b: lch(50% 30 0deg);\n}\n"
);
test!(
    negative_infinite_oklch_hue_becomes_zero,
    "@use \"sass:math\";\na {\n  b: oklch(60% 0.1 math.div(-1, 0));\n}\n",
    "a {\n  b: oklch(60% 0.1 0deg);\n}\n"
);
test!(
    change_lab_nan_channel_becomes_zero,
    "@use \"sass:color\";\n@use \"sass:math\";\na {\n  b: color.change(lab(50% 0 0), $a: math.div(0, 0));\n}\n",
    "a {\n  b: lab(50% 0 0);\n}\n"
);
test!(
    change_srgb_nan_channel_becomes_zero,
    "@use \"sass:color\";\n@use \"sass:math\";\na {\n  b: color.change(color(srgb 0.1 0.2 0.3), $red: math.div(0, 0));\n}\n",
    "a {\n  b: color(srgb 0 0.2 0.3);\n}\n"
);
test!(
    change_hsl_nan_saturation_becomes_zero,
    "@use \"sass:color\";\n@use \"sass:math\";\na {\n  b: color.change(hsl(10 20% 30%), $saturation: math.div(0, 0) * 1%);\n}\n",
    "a {\n  b: hsl(10, 0%, 30%);\n}\n"
);
test!(
    adjust_rgb_nan_channel_becomes_zero,
    "@use \"sass:color\";\n@use \"sass:math\";\na {\n  b: color.adjust(#123456, $red: math.div(0, 0));\n}\n",
    "a {\n  b: #003456;\n}\n"
);
// `color.change()` builds a legacy rgb color with `SassColor.rgbInternal`,
// which does not normalize, so the `NaN` survives until the serializer
// converts the color to hsl.
test!(
    change_rgb_keeps_nan_channel,
    "@use \"sass:color\";\n@use \"sass:math\";\na {\n  b: color.channel(color.change(#123456, $red: math.div(0, 0)), \"red\");\n}\n",
    "a {\n  b: calc(NaN);\n}\n"
);
test!(
    change_rgb_nan_channel_serializes_through_hsl,
    "@use \"sass:color\";\n@use \"sass:math\";\na {\n  b: color.change(#123456, $red: math.div(0, 0));\n}\n",
    "a {\n  b: hsl(0, 0%, 0%);\n}\n"
);
test!(
    change_rgb_keeps_negative_zero_channel,
    "@use \"sass:color\";\na {\n  b: color.channel(color.change(#123456, $red: -0), \"red\");\n}\n",
    "a {\n  b: -0;\n}\n"
);
// The legacy channel accessors round to an integer, which has no negative
// zero, so a kept `-0` channel reads back as `0`.
test!(
    red_of_negative_zero_channel,
    "@use \"sass:color\";\na {\n  b: red(color.change(#123456, $red: -0));\n}\n",
    "a {\n  b: 0;\n}\n"
);
test!(
    color_red_of_channel_rounding_to_zero,
    "@use \"sass:color\";\na {\n  b: color.red(color.change(#123456, $red: -0.4));\n}\n",
    "a {\n  b: 0;\n}\n"
);
// `rgb()` clamps its channels, and Dart's `clamp` orders `-0` below `0`.
test!(
    rgb_negative_zero_channel_clamps_to_zero,
    "@use \"sass:color\";\na {\n  b: color.channel(rgb(-0, 10, 20), \"red\");\n}\n",
    "a {\n  b: 0;\n}\n"
);
test!(
    negative_zero_alpha_clamps_to_zero,
    "@use \"sass:color\";\na {\n  b: color.alpha(rgb(1 2 3 / -0));\n}\n",
    "a {\n  b: 0;\n}\n"
);
