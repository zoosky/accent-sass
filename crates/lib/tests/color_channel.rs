#[macro_use]
mod macros;

test!(
    channel_red_with_space,
    "@use \"sass:color\";\na {\n  color: color.channel(#cc0f35, \"red\", $space: rgb);\n}\n",
    "a {\n  color: 204;\n}\n"
);
test!(
    channel_green_without_space,
    "@use \"sass:color\";\na {\n  color: color.channel(#cc0f35, \"green\");\n}\n",
    "a {\n  color: 15;\n}\n"
);
test!(
    channel_blue_with_space,
    "@use \"sass:color\";\na {\n  color: color.channel(#cc0f35, \"blue\", $space: rgb);\n}\n",
    "a {\n  color: 53;\n}\n"
);
test!(
    channel_hue_has_deg_unit,
    "@use \"sass:color\";\na {\n  color: color.channel(hsl(221, 14%, 4%), \"hue\", $space: hsl);\n}\n",
    "a {\n  color: 221deg;\n}\n"
);
test!(
    channel_hue_of_named_color,
    "@use \"sass:color\";\na {\n  color: color.channel(red, \"hue\", $space: hsl);\n}\n",
    "a {\n  color: 0deg;\n}\n"
);
test!(
    channel_saturation_has_percent_unit,
    "@use \"sass:color\";\na {\n  color: color.channel(hsl(221, 14%, 40%), \"saturation\", $space: hsl);\n}\n",
    "a {\n  color: 14%;\n}\n"
);
test!(
    channel_lightness_has_percent_unit,
    "@use \"sass:color\";\na {\n  color: color.channel(hsl(221, 14%, 40%), \"lightness\", $space: hsl);\n}\n",
    "a {\n  color: 40%;\n}\n"
);
test!(
    channel_alpha_is_unitless,
    "@use \"sass:color\";\na {\n  color: color.channel(rgba(10, 20, 30, 0.5), \"alpha\");\n}\n",
    "a {\n  color: 0.5;\n}\n"
);
test!(
    channel_name_is_case_insensitive,
    "@use \"sass:color\";\na {\n  color: color.channel(#cc0f35, \"RED\", $space: rgb);\n}\n",
    "a {\n  color: 204;\n}\n"
);
test!(
    channel_space_passed_positionally,
    "@use \"sass:color\";\na {\n  color: color.channel(#cc0f35, \"red\", rgb);\n}\n",
    "a {\n  color: 204;\n}\n"
);
test!(
    adjust_lightness_with_hsl_space,
    "@use \"sass:color\";\na {\n  color: color.adjust(hsl(221, 14%, 40%), $lightness: 10%, $space: hsl);\n}\n",
    "a {\n  color: #6e7991;\n}\n"
);
test!(
    adjust_red_with_rgb_space,
    "@use \"sass:color\";\na {\n  color: color.adjust(#485fc7, $red: 10, $space: rgb);\n}\n",
    "a {\n  color: #525fc7;\n}\n"
);
test!(
    change_lightness_with_hsl_space,
    "@use \"sass:color\";\na {\n  color: color.change(hsl(221, 14%, 40%), $lightness: 96%, $space: hsl);\n}\n",
    "a {\n  color: #f3f4f6;\n}\n"
);
test!(
    scale_lightness_with_hsl_space,
    "@use \"sass:color\";\na {\n  color: color.scale(#cc0f35, $lightness: 20%, $space: hsl);\n}\n",
    "a {\n  color: #ef264f;\n}\n"
);
error!(
    channel_unknown_channel_name,
    "@use \"sass:color\";\na {\n  color: color.channel(#fff, \"chroma\");\n}\n",
    "Error: $channel: Unknown channel name \"chroma\"."
);
error!(
    channel_unsupported_space,
    "@use \"sass:color\";\na {\n  color: color.channel(#fff, \"red\", $space: oklch);\n}\n",
    "Error: $space: Color space oklch is not supported by this implementation (rgb, hsl, and hwb are)."
);
error!(
    adjust_unsupported_space,
    "@use \"sass:color\";\na {\n  color: color.adjust(#fff, $lightness: -10%, $space: oklch);\n}\n",
    "Error: $space: Color space oklch is not supported by this implementation (rgb, hsl, and hwb are)."
);

// The shapes below are lifted from real-world consumers of these APIs
// (Bulma 1.0.4, Pico CSS 2.1.1, Foundation 6.9.0, USWDS 3.13.0). Expected
// values are cross-checked against Dart Sass 1.103.1.
test!(
    channel_in_arithmetic,
    "@use \"sass:color\";\na {\n  color: color.channel(#19d3c5, \"red\", $space: rgb) + 1;\n}\n",
    "a {\n  color: 26;\n}\n"
);
test!(
    channel_in_math_round,
    "@use \"sass:color\";\n@use \"sass:math\";\na {\n  color: math.round(color.channel(color.mix(#485fc7, #fff, 50%), \"red\"));\n}\n",
    "a {\n  color: 164;\n}\n"
);
test!(
    channel_in_math_div,
    "@use \"sass:color\";\n@use \"sass:math\";\na {\n  color: math.div(color.channel(#cc0f35, \"green\"), 255);\n}\n",
    "a {\n  color: 0.0588235294;\n}\n"
);
test!(
    channel_lightness_compared_in_if_light,
    "@use \"sass:color\";\na {\n  @if color.channel(#f3f4f6, \"lightness\", $space: hsl) > 60% {\n    color: light;\n  } @else {\n    color: dark;\n  }\n}\n",
    "a {\n  color: light;\n}\n"
);
test!(
    channel_lightness_compared_in_if_dark,
    "@use \"sass:color\";\na {\n  @if color.channel(#14191f, \"lightness\", $space: hsl) < 60% {\n    color: dark;\n  } @else {\n    color: light;\n  }\n}\n",
    "a {\n  color: dark;\n}\n"
);
test!(
    channel_reads_back_adjusted_lightness,
    "@use \"sass:color\";\na {\n  color: color.channel(color.adjust(#cc0f35, $lightness: 8%, $space: hsl), \"lightness\", $space: hsl);\n}\n",
    "a {\n  color: 50.9411764706%;\n}\n"
);
