//! CSS Color 4 colors: `lab()`, `lch()`, `oklab()`, `oklch()`, `color()`,
//! the non-legacy spaces in the `sass:color` functions, and missing
//! channels. Every expectation is the output of Dart Sass 1.103.1.

#[macro_use]
mod macros;

// ---------------------------------------------------------------------------
// Parsing lab(), lch(), oklab(), oklch(), and color()
// ---------------------------------------------------------------------------

test!(
    lab_percent_lightness,
    "a {\n  color: lab(50% 10 20);\n}\n",
    "a {\n  color: lab(50% 10 20);\n}\n"
);
test!(
    lab_unitless_lightness,
    "a {\n  color: lab(50 10 20);\n}\n",
    "a {\n  color: lab(50% 10 20);\n}\n"
);
test!(
    lab_percent_a_b,
    "a {\n  color: lab(50% 50% -50%);\n}\n",
    "a {\n  color: lab(50% 62.5 -62.5);\n}\n"
);
test!(
    lab_lightness_clamped,
    "a {\n  color: lab(150% 10 20);\n}\n",
    "a {\n  color: lab(100% 10 20);\n}\n"
);
test!(
    lab_negative_lightness_clamped,
    "a {\n  color: lab(-10% 10 20);\n}\n",
    "a {\n  color: lab(0% 10 20);\n}\n"
);
test!(
    lab_a_b_unclamped,
    "a {\n  color: lab(50% 200 -200);\n}\n",
    "a {\n  color: lab(50% 200 -200);\n}\n"
);
test!(
    lab_slash_alpha,
    "a {\n  color: lab(50% 10 20 / 0.5);\n}\n",
    "a {\n  color: lab(50% 10 20 / 0.5);\n}\n"
);
test!(
    lab_percent_alpha,
    "a {\n  color: lab(50% 10 20 / 50%);\n}\n",
    "a {\n  color: lab(50% 10 20 / 0.5);\n}\n"
);
test!(
    lab_alpha_clamped,
    "a {\n  color: lab(50% 10 20 / 2);\n}\n",
    "a {\n  color: lab(50% 10 20);\n}\n"
);
test!(
    lab_none_channel,
    "a {\n  color: lab(50% none 20);\n}\n",
    "a {\n  color: lab(50% none 20);\n}\n"
);
test!(
    lab_none_alpha,
    "a {\n  color: lab(50% 10 20 / none);\n}\n",
    "a {\n  color: lab(50% 10 20 / none);\n}\n"
);
test!(
    lab_none_uppercase,
    "a {\n  color: lab(50% 10 NONE);\n}\n",
    "a {\n  color: lab(50% 10 none);\n}\n"
);
error!(
    lab_wrong_lightness_unit,
    "a {\n  color: lab(50px 10 20);\n}\n",
    "Error: $lightness: Expected 50px to have unit \"%\" or no units."
);
error!(
    lab_wrong_a_unit,
    "a {\n  color: lab(50% 10px 20);\n}\n",
    "Error: $a: Expected 10px to have unit \"%\" or no units."
);
error!(
    lab_too_many_channels,
    "a {\n  color: lab(50% 10 20 30);\n}\n",
    "Error: $channels: The lab color space has 3 channels but (50% 10 20 30) has 4."
);
error!(
    lab_too_few_channels,
    "a {\n  color: lab(50% 10);\n}\n",
    "Error: $channels: The lab color space has 3 channels but (50% 10) has 2."
);
error!(
    lab_empty_list,
    "a {\n  color: lab(());\n}\n", "Error: $channels: Color component list may not be empty."
);
error!(
    lab_comma_list,
    "a {\n  color: lab((50%, 10, 20));\n}\n",
    "Error: $channels: Expected a space- or slash-separated list, was (50%, 10, 20)"
);
error!(
    lab_bracketed_list,
    "a {\n  color: lab([50% 10 20]);\n}\n",
    "Error: $channels: Expected an unbracketed list, was [50% 10 20]"
);
error!(
    lab_quoted_channel,
    "a {\n  color: lab(50% 10 \"a\");\n}\n",
    "Error: $channels: Expected b channel to be a number, was \"a\"."
);
error!(
    lab_unquoted_channel,
    "a {\n  color: lab(50% 10 a);\n}\n",
    "Error: $channels: Expected b channel to be a number, was a."
);
error!(
    lab_quoted_alpha,
    "a {\n  color: lab(50% 10 20 / \"a\");\n}\n", "Error: $channels: \"a\" is not a number."
);
test!(
    lab_three_slash_elements,
    "a {\n  color: lab(50% 10 20 / 1 / 2);\n}\n",
    "a {\n  color: lab(50% 10 20);\n}\n"
);
test!(
    lab_slash_number_alpha,
    "a {\n  color: lab(50% 10 20/0.5);\n}\n",
    "a {\n  color: lab(50% 10 20 / 0.5);\n}\n"
);
test!(
    lab_var_channel,
    "a {\n  color: lab(var(--l) 10 20);\n}\n",
    "a {\n  color: lab(var(--l) 10 20);\n}\n"
);
test!(
    lab_var_alpha,
    "a {\n  color: lab(50% 10 20 / var(--a));\n}\n",
    "a {\n  color: lab(50% 10 20/var(--a));\n}\n"
);
test!(
    lab_var_all,
    "a {\n  color: lab(var(--all));\n}\n",
    "a {\n  color: lab(var(--all));\n}\n"
);
test!(
    lab_var_slash_alpha,
    "a {\n  color: lab(var(--x) / 0.5);\n}\n",
    "a {\n  color: lab(var(--x)/0.5);\n}\n"
);
test!(
    lab_relative_color,
    "a {\n  color: lab(from red l a b);\n}\n",
    "a {\n  color: lab(from red l a b);\n}\n"
);
test!(
    lab_calc_channel,
    "a {\n  color: lab(1% 2 calc(1 + var(--x)));\n}\n",
    "a {\n  color: lab(1% 2 calc(1 + var(--x)));\n}\n"
);
test!(
    lch_degrees,
    "a {\n  color: lch(50% 10 20deg);\n}\n",
    "a {\n  color: lch(50% 10 20deg);\n}\n"
);
test!(
    lch_turn,
    "a {\n  color: lch(50% 10 0.5turn);\n}\n",
    "a {\n  color: lch(50% 10 180deg);\n}\n"
);
test!(
    lch_hue_wraps,
    "a {\n  color: lch(50% 10 400);\n}\n",
    "a {\n  color: lch(50% 10 40deg);\n}\n"
);
test!(
    lch_negative_hue_wraps,
    "a {\n  color: lch(50% 10 -30);\n}\n",
    "a {\n  color: lch(50% 10 330deg);\n}\n"
);
test!(
    lch_negative_chroma_clamped,
    "a {\n  color: lch(50% -10 20);\n}\n",
    "a {\n  color: lch(50% 0 20deg);\n}\n"
);
test!(
    lch_percent_chroma,
    "a {\n  color: lch(50% 50% 20);\n}\n",
    "a {\n  color: lch(50% 75 20deg);\n}\n"
);
error!(
    lch_hue_wrong_unit,
    "a {\n  color: lch(50% 10 20px);\n}\n",
    "Error: $hue: Expected 20px to have an angle unit (deg, grad, rad, turn)."
);
test!(
    lch_none_hue,
    "a {\n  color: lch(50% 10 none);\n}\n",
    "a {\n  color: lch(50% 10 none);\n}\n"
);
test!(
    oklab_unitless_lightness,
    "a {\n  color: oklab(0.5 0.1 -0.1);\n}\n",
    "a {\n  color: oklab(50% 0.1 -0.1);\n}\n"
);
test!(
    oklab_percent_lightness,
    "a {\n  color: oklab(50% 0.1 -0.1);\n}\n",
    "a {\n  color: oklab(50% 0.1 -0.1);\n}\n"
);
test!(
    oklab_lightness_clamped,
    "a {\n  color: oklab(1.5 0.1 0.1);\n}\n",
    "a {\n  color: oklab(100% 0.1 0.1);\n}\n"
);
test!(
    oklab_percent_a_b,
    "a {\n  color: oklab(0.5 50% -50%);\n}\n",
    "a {\n  color: oklab(50% 0.2 -0.2);\n}\n"
);
test!(
    oklab_none_a_b,
    "a {\n  color: oklab(0.5 none none);\n}\n",
    "a {\n  color: oklab(50% none none);\n}\n"
);
test!(
    oklch_basic,
    "a {\n  color: oklch(0.5 0.1 20);\n}\n",
    "a {\n  color: oklch(50% 0.1 20deg);\n}\n"
);
test!(
    oklch_percent_chroma,
    "a {\n  color: oklch(0.5 50% 20);\n}\n",
    "a {\n  color: oklch(50% 0.2 20deg);\n}\n"
);
test!(
    oklch_negative_chroma_rotates_hue,
    "a {\n  color: oklch(0.5 -0.1 20);\n}\n",
    "a {\n  color: oklch(50% 0 20deg);\n}\n"
);
test!(
    color_srgb,
    "a {\n  color: color(srgb 1 0 0);\n}\n",
    "a {\n  color: color(srgb 1 0 0);\n}\n"
);
test!(
    color_srgb_percent,
    "a {\n  color: color(srgb 50% 0 0);\n}\n",
    "a {\n  color: color(srgb 0.5 0 0);\n}\n"
);
test!(
    color_srgb_unclamped,
    "a {\n  color: color(srgb 1.5 -0.5 0);\n}\n",
    "a {\n  color: color(srgb 1.5 -0.5 0);\n}\n"
);
test!(
    color_srgb_alpha,
    "a {\n  color: color(srgb 1 0 0 / 0.5);\n}\n",
    "a {\n  color: color(srgb 1 0 0 / 0.5);\n}\n"
);
test!(
    color_space_name_case_insensitive,
    "a {\n  color: color(SRGB 1 0 0);\n}\n",
    "a {\n  color: color(srgb 1 0 0);\n}\n"
);
test!(
    color_xyz_d65_alias,
    "a {\n  color: color(xyz-d65 0.5 0.5 0.5);\n}\n",
    "a {\n  color: color(xyz 0.5 0.5 0.5);\n}\n"
);
test!(
    color_every_space,
    "a {\n  a: color(srgb-linear 0.5 0.5 0.5);\n  b: color(display-p3 1 0 0);\n  c: color(display-p3-linear 0.5 0.2 0.1);\n  d: color(a98-rgb 0.5 0.5 0.5);\n  e: color(prophoto-rgb 0.5 0.5 0.5);\n  f: color(rec2020 0.5 0.5 0.5);\n  g: color(xyz 0.5 0.5 0.5);\n  h: color(xyz-d50 0.5 0.5 0.5);\n}\n",
    "a {\n  a: color(srgb-linear 0.5 0.5 0.5);\n  b: color(display-p3 1 0 0);\n  c: color(display-p3-linear 0.5 0.2 0.1);\n  d: color(a98-rgb 0.5 0.5 0.5);\n  e: color(prophoto-rgb 0.5 0.5 0.5);\n  f: color(rec2020 0.5 0.5 0.5);\n  g: color(xyz 0.5 0.5 0.5);\n  h: color(xyz-d50 0.5 0.5 0.5);\n}\n"
);
test!(
    color_none_channel,
    "a {\n  color: color(srgb none 0 0);\n}\n",
    "a {\n  color: color(srgb none 0 0);\n}\n"
);
error!(
    color_unknown_space,
    "a {\n  color: color(foo 1 0 0);\n}\n", "Error: $description: Unknown color space \"foo\"."
);
error!(
    color_rejects_rgb,
    "a {\n  color: color(rgb 1 0 0);\n}\n",
    "Error: $description: The color() function doesn't support the color space rgb. Use the rgb() function instead."
);
error!(
    color_rejects_oklch,
    "a {\n  color: color(oklch 1 0 0);\n}\n",
    "Error: $description: The color() function doesn't support the color space oklch. Use the oklch() function instead."
);
error!(
    color_quoted_space,
    "a {\n  color: color(\"srgb\" 1 0 0);\n}\n",
    "Error: $description: Expected \"srgb\" to be an unquoted string."
);
error!(
    color_missing_space,
    "a {\n  color: color(1 0 0);\n}\n", "Error: $description: 1 is not a string."
);
error!(
    color_too_few_channels,
    "a {\n  color: color(srgb 1 0);\n}\n",
    "Error: $description: The srgb color space has 3 channels but (srgb 1 0) has 2."
);
error!(
    color_no_channels,
    "a {\n  color: color(srgb);\n}\n",
    "Error: $description: The srgb color space has 3 channels but srgb has 0."
);
error!(
    color_wrong_unit,
    "a {\n  color: color(srgb 1px 0 0);\n}\n",
    "Error: $red: Expected 1px to have unit \"%\" or no units."
);
test!(
    color_var_space,
    "a {\n  color: color(var(--s) 1 2 3);\n}\n",
    "a {\n  color: color(var(--s) 1 2 3);\n}\n"
);
test!(
    color_var_channel,
    "a {\n  color: color(srgb var(--r) 0 0);\n}\n",
    "a {\n  color: color(srgb var(--r) 0 0);\n}\n"
);
test!(
    color_relative,
    "a {\n  color: color(from red srgb r g b);\n}\n",
    "a {\n  color: color(from red srgb r g b);\n}\n"
);
test!(
    hsl_none_saturation,
    "a {\n  color: hsl(120 none 50%);\n}\n",
    "a {\n  color: hsl(120deg none 50%);\n}\n"
);
test!(
    hsl_none_lightness_with_alpha,
    "a {\n  color: hsl(120 50% none / 0.5);\n}\n",
    "a {\n  color: hsl(120deg 50% none / 0.5);\n}\n"
);
test!(
    hsl_none_hue,
    "a {\n  color: hsl(none 50% 50%);\n}\n",
    "a {\n  color: hsl(none 50% 50%);\n}\n"
);
test!(
    hsl_none_alpha,
    "a {\n  color: hsl(120 50% 50% / none);\n}\n",
    "a {\n  color: hsl(120deg 50% 50% / none);\n}\n"
);
error!(
    hsl_comma_none_errors,
    "a {\n  color: hsl(none, 50%, 50%);\n}\n", "Error: $hue: none is not a number."
);
test!(
    hwb_none_hue,
    "a {\n  color: hwb(none 20% 30%);\n}\n",
    "a {\n  color: hwb(none 20% 30%);\n}\n"
);
test!(
    hwb_none_whiteness,
    "a {\n  color: hwb(120 none 30%);\n}\n",
    "a {\n  color: hwb(120deg none 30%);\n}\n"
);
test!(
    rgb_none_red,
    "a {\n  color: rgb(none 1 2);\n}\n",
    "a {\n  color: rgb(none 1 2);\n}\n"
);
test!(
    rgb_none_alpha,
    "a {\n  color: rgb(10 20 30 / none);\n}\n",
    "a {\n  color: rgb(10 20 30 / none);\n}\n"
);
error!(
    rgb_four_space_separated_channels,
    "a {\n  color: rgb(1 2 3 4);\n}\n",
    "Error: $channels: The rgb color space has 3 channels but (1 2 3 4) has 4."
);
error!(
    rgb_wrong_unit_message,
    "a {\n  color: rgb(10px 2 3);\n}\n",
    "Error: $red: Expected 10px to have unit \"%\" or no units."
);
error!(
    rgb_comma_list_in_channels,
    "a {\n  color: rgb((1, 2, 3));\n}\n",
    "Error: $channels: Expected a space- or slash-separated list, was (1, 2, 3)"
);
test!(
    rgb_var_slash_alpha_keeps_slash,
    "a {\n  color: rgb(var(--x) / 0.5);\n}\n",
    "a {\n  color: rgb(var(--x)/0.5);\n}\n"
);
test!(
    rgb_var_channel_uses_commas,
    "a {\n  color: rgb(var(--r) 0 0 / 0.5);\n}\n",
    "a {\n  color: rgb(var(--r), 0, 0, 0.5);\n}\n"
);
test!(
    hwb_var_alpha_keeps_slash,
    "a {\n  color: hwb(1 2% 3% / var(--a));\n}\n",
    "a {\n  color: hwb(1 2% 3%/var(--a));\n}\n"
);
error!(
    rgb_two_arg_non_legacy,
    "a {\n  color: rgb(lab(50% 1 2), 0.5);\n}\n",
    "Error: $rgb: Expected lab(50% 1 2) to be in the legacy RGB, HSL, or HWB color space."
);
error!(
    rgba_two_arg_non_legacy,
    "a {\n  color: rgba(oklch(50% 0.1 20), 0.5);\n}\n",
    "Error: $rgba: Expected oklch(50% 0.1 20deg) to be in the legacy RGB, HSL, or HWB color space."
);
test!(
    rgb_two_arg_hwb,
    "a {\n  color: rgb(hwb(120 20% 30%), 0.5);\n}\n",
    "a {\n  color: rgba(20%, 70%, 20%, 0.5);\n}\n"
);
test!(
    module_hwb_comma_none,
    "@use \"sass:color\";\na {\n  color: color.hwb(none, 20%, 30%);\n}\n",
    "a {\n  color: hwb(none 20% 30%);\n}\n"
);
test!(
    module_hwb_comma_alpha_none,
    "@use \"sass:color\";\na {\n  color: color.hwb(120, 20%, 30%, none);\n}\n",
    "a {\n  color: hwb(120deg 20% 30% / none);\n}\n"
);
error!(
    module_hwb_unitless_whiteness,
    "@use \"sass:color\";\na {\n  color: color.hwb(120, 20, 30%);\n}\n",
    "Error: $whiteness: Expected 20 to have unit \"%\"."
);
error!(
    module_hwb_one_channel,
    "@use \"sass:color\";\na {\n  color: color.hwb(120);\n}\n",
    "Error: $channels: The hwb color space has 3 channels but 120 has 1."
);
test!(
    lab_nan_lightness,
    "@use \"sass:math\";\na {\n  color: lab(math.div(0, 0) 10 20);\n}\n",
    "a {\n  color: lab(0% 10 20);\n}\n"
);
test!(
    lab_infinite_a,
    "@use \"sass:math\";\na {\n  color: lab(50% math.div(1, 0) 20);\n}\n",
    "a {\n  color: lab(50% calc(infinity) 20);\n}\n"
);
test!(
    color_nan_channel,
    "@use \"sass:math\";\na {\n  color: color(srgb math.div(0, 0) 0 0);\n}\n",
    "a {\n  color: color(srgb calc(NaN) 0 0);\n}\n"
);
test!(
    lab_nan_alpha,
    "@use \"sass:math\";\na {\n  color: lab(50% 10 20 / math.div(0, 0));\n}\n",
    "a {\n  color: lab(50% 10 calc(NaN));\n}\n"
);

// ---------------------------------------------------------------------------
// Serialization
// ---------------------------------------------------------------------------

error!(
    inspect_lab,
    "@use \"sass:color\";\na {\n  color: meta.inspect(lab(50% 10 20));\n}\n",
    "Error: There is no module with the namespace \"meta\"."
);
error!(
    inspect_oklch_alpha,
    "@use \"sass:color\";\na {\n  color: meta.inspect(oklch(50% 0.1 20 / 0.5));\n}\n",
    "Error: There is no module with the namespace \"meta\"."
);
error!(
    inspect_color_function,
    "@use \"sass:color\";\na {\n  color: meta.inspect(color(srgb 0.1 0.2 0.3 / 0.5));\n}\n",
    "Error: There is no module with the namespace \"meta\"."
);
error!(
    inspect_missing_hue,
    "@use \"sass:color\";\na {\n  color: meta.inspect(hsl(120 none 50%));\n}\n",
    "Error: There is no module with the namespace \"meta\"."
);
error!(
    inspect_out_of_range_lab,
    "@use \"sass:color\";\na {\n  color: meta.inspect(lab(150% 10 20));\n}\n",
    "Error: There is no module with the namespace \"meta\"."
);
test!(
    compressed_lab,
    "@use \"sass:color\";\na {\n  color: lab(50% 10 20 / 0.5);\n}\n",
    "a{color:lab(50 10 20/.5)}",
    grass::Options::default().style(grass::OutputStyle::Compressed)
);
test!(
    compressed_oklch,
    "@use \"sass:color\";\na {\n  color: oklch(50% 0.1 20);\n}\n",
    "a{color:oklch(.5 .1 20)}",
    grass::Options::default().style(grass::OutputStyle::Compressed)
);
test!(
    compressed_oklab_negative_a,
    "@use \"sass:color\";\na {\n  color: oklab(0.7 -0.1 0.1);\n}\n",
    "a{color:oklab(.7 -0.1 .1)}",
    grass::Options::default().style(grass::OutputStyle::Compressed)
);
test!(
    compressed_color_function,
    "@use \"sass:color\";\na {\n  color: color(display-p3 0.5 0.25 1 / 0.5);\n}\n",
    "a{color:color(display-p3 .5 .25 1/.5)}",
    grass::Options::default().style(grass::OutputStyle::Compressed)
);
test!(
    compressed_missing_hue,
    "@use \"sass:color\";\na {\n  color: hsl(none 50% 50% / 0.5);\n}\n",
    "a{color:hsl(none 50% 50%/.5)}",
    grass::Options::default().style(grass::OutputStyle::Compressed)
);
test!(
    compressed_missing_red,
    "@use \"sass:color\";\na {\n  color: rgb(none 1 2 / 0.5);\n}\n",
    "a{color:rgb(none 1 2/.5)}",
    grass::Options::default().style(grass::OutputStyle::Compressed)
);
test!(
    color_mix_fallback_lab,
    "@use \"sass:color\";\na {\n  color: color.change(lab(50% 10 20), $lightness: 150%);\n}\n",
    "a {\n  color: color-mix(in lab, color(xyz 2.87028635 2.9172111384 2.5646783747) 100%, black);\n}\n"
);
test!(
    color_mix_fallback_negative_lab,
    "@use \"sass:color\";\na {\n  color: color.change(lab(50% 10 20), $lightness: -50%);\n}\n",
    "a {\n  color: color-mix(in lab, color(xyz -0.0509143142 -0.0556460378 -0.0743483124) 100%, black);\n}\n"
);
test!(
    color_mix_fallback_oklch,
    "@use \"sass:color\";\na {\n  color: color.change(oklch(0.5 0.1 20), $lightness: 1.5);\n}\n",
    "a {\n  color: color-mix(in oklch, color(xyz 3.5372424372 3.2968396803 3.1423195885) 100%, black);\n}\n"
);
test!(
    color_mix_fallback_compressed,
    "@use \"sass:color\";\na {\n  color: color.change(lab(50% 10 20), $lightness: 150%);\n}\n",
    "a{color:color-mix(in lab,color(xyz 2.87028635 2.9172111384 2.5646783747)100%,red)}",
    grass::Options::default().style(grass::OutputStyle::Compressed)
);
test!(
    color_mix_fallback_with_alpha,
    "@use \"sass:color\";\na {\n  color: color.change(lab(50% 10 20 / 0.5), $lightness: 150%);\n}\n",
    "a {\n  color: color-mix(in lab, color(xyz 2.87028635 2.9172111384 2.5646783747 / 0.5) 100%, black);\n}\n"
);
test!(
    relative_from_black_with_missing_chroma,
    "@use \"sass:color\";\na {\n  color: color.change(lch(50% 10 20), $lightness: 150%, $chroma: none);\n}\n",
    "a {\n  color: lch(from black 150% none 20deg);\n}\n"
);
test!(
    relative_from_black_with_missing_hue,
    "@use \"sass:color\";\na {\n  color: color.change(lch(50% 10 20), $lightness: 150%, $hue: none);\n}\n",
    "a {\n  color: lch(from black 150% 10 none);\n}\n"
);
test!(
    relative_from_red_compressed,
    "@use \"sass:color\";\na {\n  color: color.change(lch(50% 10 20), $lightness: 150%, $hue: none);\n}\n",
    "a{color:lch(from red 150 10 none)}", grass::Options::default().style(grass::OutputStyle::Compressed)
);
test!(
    out_of_gamut_srgb_serializes_unclamped,
    "@use \"sass:color\";\na {\n  color: color(srgb 1.5 -0.5 0);\n}\n",
    "a {\n  color: color(srgb 1.5 -0.5 0);\n}\n"
);
test!(
    missing_alpha_serializes_none,
    "@use \"sass:color\";\na {\n  color: color(display-p3 0.5 none 0.5 / none);\n}\n",
    "a {\n  color: color(display-p3 0.5 none 0.5 / none);\n}\n"
);

// ---------------------------------------------------------------------------
// Conversions
// ---------------------------------------------------------------------------

test!(
    to_space_lab,
    "@use \"sass:color\";\na {\n  color: color.to-space(#cc0f35, lab);\n}\n",
    "a {\n  color: lab(44.2229117293% 67.6217073989 34.5537259027);\n}\n"
);
test!(
    to_space_lch,
    "@use \"sass:color\";\na {\n  color: color.to-space(#cc0f35, lch);\n}\n",
    "a {\n  color: lch(44.2229117293% 75.9384967279 27.0663830082deg);\n}\n"
);
test!(
    to_space_oklab,
    "@use \"sass:color\";\na {\n  color: color.to-space(#cc0f35, oklab);\n}\n",
    "a {\n  color: oklab(53.8574934869% 0.1973637333 0.0737989032);\n}\n"
);
test!(
    to_space_display_p3,
    "@use \"sass:color\";\na {\n  color: color.to-space(#cc0f35, display-p3);\n}\n",
    "a {\n  color: color(display-p3 0.7336902375 0.1705609368 0.2295679504);\n}\n"
);
test!(
    to_space_srgb,
    "@use \"sass:color\";\na {\n  color: color.to-space(#cc0f35, srgb);\n}\n",
    "a {\n  color: color(srgb 0.8 0.0588235294 0.2078431373);\n}\n"
);
test!(
    to_space_srgb_linear,
    "@use \"sass:color\";\na {\n  color: color.to-space(#cc0f35, srgb-linear);\n}\n",
    "a {\n  color: color(srgb-linear 0.6038273389 0.0047769535 0.0356013149);\n}\n"
);
test!(
    to_space_a98,
    "@use \"sass:color\";\na {\n  color: color.to-space(#cc0f35, a98-rgb);\n}\n",
    "a {\n  color: color(a98-rgb 0.6835771139 0.0880428086 0.2158615313);\n}\n"
);
test!(
    to_space_prophoto,
    "@use \"sass:color\";\na {\n  color: color.to-space(#cc0f35, prophoto-rgb);\n}\n",
    "a {\n  color: color(prophoto-rgb 0.5366531494 0.2182279629 0.1708512299);\n}\n"
);
test!(
    to_space_rec2020,
    "@use \"sass:color\";\na {\n  color: color.to-space(#cc0f35, rec2020);\n}\n",
    "a {\n  color: color(rec2020 0.669638046 0.2785156946 0.2674384368);\n}\n"
);
test!(
    to_space_xyz,
    "@use \"sass:color\";\na {\n  color: color.to-space(#cc0f35, xyz);\n}\n",
    "a {\n  color: color(xyz 0.257146356 0.1343837139 0.0460820592);\n}\n"
);
test!(
    to_space_xyz_d50,
    "@use \"sass:color\";\na {\n  color: color.to-space(#cc0f35, xyz-d50);\n}\n",
    "a {\n  color: color(xyz-d50 0.2702420502 0.1399301439 0.0342942735);\n}\n"
);
test!(
    to_space_lab_to_rgb,
    "@use \"sass:color\";\na {\n  color: color.to-space(lab(50% 10 20), rgb);\n}\n",
    "a {\n  color: rgb(56.3680322249%, 44.040338116%, 33.4681021539%);\n}\n"
);
test!(
    to_space_lab_to_hsl,
    "@use \"sass:color\";\na {\n  color: color.to-space(lab(50% 10 20), hsl);\n}\n",
    "a {\n  color: hsl(27.7002661474, 25.4907785484%, 44.9180671894%);\n}\n"
);
test!(
    to_space_oklch_to_hwb,
    "@use \"sass:color\";\na {\n  color: color.to-space(oklch(50% 0.1 20), hwb);\n}\n",
    "a {\n  color: hsl(358.7278995881, 33.5405531375%, 43.3924538914%);\n}\n"
);
test!(
    to_space_display_p3_out_of_gamut_rgb,
    "@use \"sass:color\";\na {\n  color: color.to-space(color(display-p3 1 0 0), rgb);\n}\n",
    "a {\n  color: hsl(356.5173401072, 152.3457440386%, 43.3162194433%);\n}\n"
);
test!(
    to_space_display_p3_to_srgb,
    "@use \"sass:color\";\na {\n  color: color.to-space(color(display-p3 1 0 0), srgb);\n}\n",
    "a {\n  color: color(srgb 1.0930663624 -0.2267419736 -0.1501345809);\n}\n"
);
test!(
    to_space_prophoto_to_rec2020,
    "@use \"sass:color\";\na {\n  color: color.to-space(color(prophoto-rgb 0.5 0.5 0.5), rec2020);\n}\n",
    "a {\n  color: color(rec2020 0.5946035575 0.5946035575 0.5946035575);\n}\n"
);
test!(
    to_space_xyz_d50_to_oklch,
    "@use \"sass:color\";\na {\n  color: color.to-space(color(xyz-d50 0.2 0.4 0.6), oklch);\n}\n",
    "a {\n  color: oklch(71.8118372774% 0.2420834428 201.5843445791deg);\n}\n"
);
test!(
    to_space_a98_to_lab,
    "@use \"sass:color\";\na {\n  color: color.to-space(color(a98-rgb 0.2 0.4 0.6), lab);\n}\n",
    "a {\n  color: lab(40.0797299464% -11.9449257398 -37.7930988533);\n}\n"
);
test!(
    to_space_white_to_display_p3,
    "@use \"sass:color\";\na {\n  color: color.to-space(white, display-p3);\n}\n",
    "a {\n  color: color(display-p3 1 1 1);\n}\n"
);
test!(
    to_space_white_to_xyz,
    "@use \"sass:color\";\na {\n  color: color.to-space(white, xyz);\n}\n",
    "a {\n  color: color(xyz 0.9504559271 1 1.0890577508);\n}\n"
);
test!(
    to_space_gray_to_oklch_has_missing_hue,
    "@use \"sass:color\";\na {\n  color: color.to-space(gray, oklch);\n}\n",
    "a {\n  color: oklch(59.9870805622% 0 none);\n}\n"
);
test!(
    to_space_gray_to_lch_has_missing_hue,
    "@use \"sass:color\";\na {\n  color: color.to-space(gray, lch);\n}\n",
    "a {\n  color: lch(53.5850134522% 0 none);\n}\n"
);
test!(
    to_space_missing_hue_to_lab,
    "@use \"sass:color\";\na {\n  color: color.to-space(hsl(none 0% 50%), lab);\n}\n",
    "a {\n  color: lab(53.3889647411% 0 0);\n}\n"
);
test!(
    to_space_missing_whiteness_blackness_to_lch,
    "@use \"sass:color\";\na {\n  color: color.to-space(hwb(20 none none), lch);\n}\n",
    "a {\n  color: lch(none none 48.3267322369deg);\n}\n"
);
test!(
    to_space_missing_lightness_to_oklch,
    "@use \"sass:color\";\na {\n  color: color.to-space(lab(none 10 20), oklch);\n}\n",
    "a {\n  color: oklch(none 0.4252336568 13.7521936893deg);\n}\n"
);
test!(
    to_space_missing_chroma_to_lab,
    "@use \"sass:color\";\na {\n  color: color.to-space(oklch(50% none 20), lab);\n}\n",
    "a {\n  color: lab(42% 0 0);\n}\n"
);
test!(
    to_space_missing_hue_keeps_hue_missing_across_polar,
    "@use \"sass:color\";\na {\n  color: color.to-space(oklch(50% 0.1 none), lch);\n}\n",
    "a {\n  color: lch(40.7423531284% 32.8495186995 none);\n}\n"
);
test!(
    to_space_missing_red_to_hsl,
    "@use \"sass:color\";\na {\n  color: color.to-space(rgb(none 20 30), hsl);\n}\n",
    "a {\n  color: hsl(200, 100%, 5.8823529412%);\n}\n"
);
test!(
    to_space_missing_red_to_lab,
    "@use \"sass:color\";\na {\n  color: color.to-space(rgb(none 20 30), lab);\n}\n",
    "a {\n  color: lab(5.2408778377% -4.2109320426 -9.2418429616);\n}\n"
);
test!(
    to_space_missing_saturation_to_rgb,
    "@use \"sass:color\";\na {\n  color: color.to-space(hsl(20 none 50%), rgb);\n}\n",
    "a {\n  color: rgb(50%, 50%, 50%);\n}\n"
);
test!(
    to_space_missing_lightness_to_lch,
    "@use \"sass:color\";\na {\n  color: color.to-space(hsl(20 50% none), lch);\n}\n",
    "a {\n  color: lch(none 0 none);\n}\n"
);
test!(
    to_space_missing_all_to_xyz,
    "@use \"sass:color\";\na {\n  color: color.to-space(lch(none none none), xyz);\n}\n",
    "a {\n  color: color(xyz none none none);\n}\n"
);
test!(
    to_space_missing_alpha_becomes_zero,
    "@use \"sass:color\";\na {\n  color: color.to-space(color(display-p3 0.5 none 0.5 / none), srgb);\n}\n",
    "a {\n  color: color(srgb 0.5489590817 none 0.5177839899 / 0);\n}\n"
);
test!(
    to_space_same_space_keeps_missing,
    "@use \"sass:color\";\na {\n  color: color.to-space(lab(50% none 20), lab);\n}\n",
    "a {\n  color: lab(50% none 20);\n}\n"
);
test!(
    to_space_null_space,
    "@use \"sass:color\";\na {\n  color: color.to-space(lab(50% 10 20), $space: null);\n}\n",
    "a {\n  color: lab(50% 10 20);\n}\n"
);
error!(
    to_space_unknown_space,
    "@use \"sass:color\";\na {\n  color: color.to-space(red, foo);\n}\n",
    "Error: $space: Unknown color space \"foo\"."
);
error!(
    to_space_quoted_space,
    "@use \"sass:color\";\na {\n  color: color.to-space(red, \"rgb\");\n}\n",
    "Error: $space: Expected \"rgb\" to be an unquoted string."
);
test!(
    to_space_space_name_case_insensitive,
    "@use \"sass:color\";\na {\n  color: color.to-space(red, Lab);\n}\n",
    "a {\n  color: lab(54.2905414047% 80.8049281704 69.8909647686);\n}\n"
);
test!(
    to_space_xyz_alias,
    "@use \"sass:color\";\na {\n  color: color.to-space(red, XYZ-D65);\n}\n",
    "a {\n  color: color(xyz 0.4123907993 0.2126390059 0.0193308187);\n}\n"
);

// ---------------------------------------------------------------------------
// color.space(), color.is-legacy(), equality
// ---------------------------------------------------------------------------

test!(
    space_of_each_function,
    "@use \"sass:color\";\na {\n  a: color.space(lab(50% 10 20));\n  b: color.space(lch(50% 10 20));\n  c: color.space(oklab(0.5 0.1 0.1));\n  d: color.space(oklch(0.5 0.1 20));\n  e: color.space(color(xyz-d65 1 2 3));\n  f: color.space(color(xyz-d50 1 2 3));\n  g: color.space(color(display-p3-linear 1 2 3));\n}\n",
    "a {\n  a: lab;\n  b: lch;\n  c: oklab;\n  d: oklch;\n  e: xyz;\n  f: xyz-d50;\n  g: display-p3-linear;\n}\n"
);
test!(
    is_legacy_false_for_lab,
    "@use \"sass:color\";\na {\n  color: color.is-legacy(lab(50% 10 20));\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    is_legacy_false_for_srgb,
    "@use \"sass:color\";\na {\n  color: color.is-legacy(color(srgb 1 0 0));\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    equality_same_space,
    "@use \"sass:color\";\na {\n  color: lab(50% 10 20) == lab(50% 10 20);\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    equality_different_non_legacy_spaces,
    "@use \"sass:color\";\na {\n  color: lab(50% 10 20) == lch(50% 10 20);\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    equality_legacy_vs_srgb,
    "@use \"sass:color\";\na {\n  color: color(srgb 1 0 0) == red;\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    equality_legacy_across_spaces,
    "@use \"sass:color\";\na {\n  color: red == hsl(0 100% 50%);\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    equality_missing_channels,
    "@use \"sass:color\";\na {\n  color: lab(50% none 20) == lab(50% none 20);\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    equality_missing_vs_zero,
    "@use \"sass:color\";\na {\n  color: lab(50% none 20) == lab(50% 0 20);\n}\n",
    "a {\n  color: false;\n}\n"
);

// ---------------------------------------------------------------------------
// color.channel(), color.is-missing(), color.is-powerless()
// ---------------------------------------------------------------------------

test!(
    channel_lab_lightness_is_percent,
    "@use \"sass:color\";\na {\n  color: color.channel(lab(50% 10 20), \"lightness\");\n}\n",
    "a {\n  color: 50%;\n}\n"
);
test!(
    channel_oklab_lightness_is_percent,
    "@use \"sass:color\";\na {\n  color: color.channel(oklab(0.5 0.1 0.1), \"lightness\");\n}\n",
    "a {\n  color: 50%;\n}\n"
);
test!(
    channel_lch_chroma_unitless,
    "@use \"sass:color\";\na {\n  color: color.channel(lch(50% 10 20), \"chroma\");\n}\n",
    "a {\n  color: 10;\n}\n"
);
test!(
    channel_oklch_hue_degrees,
    "@use \"sass:color\";\na {\n  color: color.channel(oklch(0.5 0.1 20), \"hue\");\n}\n",
    "a {\n  color: 20deg;\n}\n"
);
test!(
    channel_xyz,
    "@use \"sass:color\";\na {\n  color: color.channel(color(xyz 0.1 0.2 0.3), \"x\");\n}\n",
    "a {\n  color: 0.1;\n}\n"
);
test!(
    channel_srgb_unitless,
    "@use \"sass:color\";\na {\n  color: color.channel(color(srgb 0.1 0.2 0.3), \"green\");\n}\n",
    "a {\n  color: 0.2;\n}\n"
);
test!(
    channel_missing_reads_zero,
    "@use \"sass:color\";\na {\n  color: color.channel(lab(50% none 20), \"a\");\n}\n",
    "a {\n  color: 0;\n}\n"
);
test!(
    channel_alpha_of_lab,
    "@use \"sass:color\";\na {\n  color: color.channel(lab(50% 10 20 / 0.5), \"alpha\");\n}\n",
    "a {\n  color: 0.5;\n}\n"
);
test!(
    channel_with_non_legacy_space,
    "@use \"sass:color\";\na {\n  color: color.channel(#cc0f35, \"lightness\", $space: oklch);\n}\n",
    "a {\n  color: 53.8574934869%;\n}\n"
);
test!(
    channel_with_non_legacy_space_hue,
    "@use \"sass:color\";\na {\n  color: color.channel(#cc0f35, \"hue\", $space: lch);\n}\n",
    "a {\n  color: 27.0663830082deg;\n}\n"
);
error!(
    channel_unknown_in_lab,
    "@use \"sass:color\";\na {\n  color: color.channel(lab(50% 10 20), \"red\");\n}\n",
    "Error: $channel: Color lab(50% 10 20) has no channel named red."
);
error!(
    channel_unknown_with_missing_channel,
    "@use \"sass:color\";\na {\n  color: color.channel(lab(50% none 20), \"foo\");\n}\n",
    "Error: $channel: Color lab(50% none 20) has no channel named foo."
);
test!(
    channel_gray_in_oklch_hue_is_zero,
    "@use \"sass:color\";\na {\n  color: color.channel(gray, \"hue\", $space: oklch);\n}\n",
    "a {\n  color: 0deg;\n}\n"
);
test!(
    is_missing_true,
    "@use \"sass:color\";\na {\n  color: color.is-missing(lab(50% none 20), \"a\");\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    is_missing_false,
    "@use \"sass:color\";\na {\n  color: color.is-missing(lab(50% none 20), \"b\");\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    is_missing_alpha,
    "@use \"sass:color\";\na {\n  color: color.is-missing(lab(50% 10 20 / none), \"alpha\");\n}\n",
    "a {\n  color: true;\n}\n"
);
error!(
    is_missing_unknown_channel,
    "@use \"sass:color\";\na {\n  color: color.is-missing(lab(50% none 20), \"foo\");\n}\n",
    "Error: $channel: Color lab(50% none 20) doesn't have a channel named \"foo\"."
);
error!(
    is_missing_unquoted_channel,
    "@use \"sass:color\";\na {\n  color: color.is-missing(lab(50% none 20), a);\n}\n",
    "Error: $channel: Expected a to be a quoted string."
);
test!(
    is_powerless_lch_hue,
    "@use \"sass:color\";\na {\n  color: color.is-powerless(lch(50% 0 20), \"hue\");\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    is_powerless_oklch_hue_with_chroma,
    "@use \"sass:color\";\na {\n  color: color.is-powerless(oklch(0.5 0.1 20), \"hue\");\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    is_powerless_hsl_hue,
    "@use \"sass:color\";\na {\n  color: color.is-powerless(hsl(120 0% 50%), \"hue\");\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    is_powerless_hwb_hue,
    "@use \"sass:color\";\na {\n  color: color.is-powerless(hwb(120 60% 40%), \"hue\");\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    is_powerless_with_space,
    "@use \"sass:color\";\na {\n  color: color.is-powerless(gray, \"hue\", $space: lch);\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    is_powerless_lab_never,
    "@use \"sass:color\";\na {\n  color: color.is-powerless(lab(0% 10 20), \"a\");\n}\n",
    "a {\n  color: false;\n}\n"
);

// ---------------------------------------------------------------------------
// Gamut mapping
// ---------------------------------------------------------------------------

test!(
    is_in_gamut_unbounded_space,
    "@use \"sass:color\";\na {\n  color: color.is-in-gamut(lab(150% 300 -300));\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    is_in_gamut_srgb_out,
    "@use \"sass:color\";\na {\n  color: color.is-in-gamut(color(srgb 1.5 -0.5 0));\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    is_in_gamut_display_p3_in_rgb,
    "@use \"sass:color\";\na {\n  color: color.is-in-gamut(color(display-p3 1 0 0), rgb);\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    is_in_gamut_display_p3_own,
    "@use \"sass:color\";\na {\n  color: color.is-in-gamut(color(display-p3 1 0 0));\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    is_in_gamut_rgb_in_lab,
    "@use \"sass:color\";\na {\n  color: color.is-in-gamut(rgb(300 -10 20), lab);\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    is_in_gamut_fuzzy_edge,
    "@use \"sass:color\";\na {\n  color: color.is-in-gamut(color(srgb 1.00000000001 0 0));\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    to_gamut_clip_srgb,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(color(srgb 1.5 -0.5 0), $method: clip);\n}\n",
    "a {\n  color: color(srgb 1 0 0);\n}\n"
);
test!(
    to_gamut_local_minde_srgb,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(color(srgb 1.5 -0.5 0), $method: local-minde);\n}\n",
    "a {\n  color: color(srgb 1 0.5589019555 0.5573766325);\n}\n"
);
test!(
    to_gamut_display_p3_in_rgb_local_minde,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(color(display-p3 1 0 0), rgb, $method: local-minde);\n}\n",
    "a {\n  color: color(display-p3 0.9177905633 0.2107213818 0.1542354933);\n}\n"
);
test!(
    to_gamut_display_p3_in_rgb_clip,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(color(display-p3 1 0 0), rgb, $method: clip);\n}\n",
    "a {\n  color: color(display-p3 0.9174875573 0.2002868077 0.1385605912);\n}\n"
);
test!(
    to_gamut_display_p3_in_srgb,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(color(display-p3 1 0 0), srgb, $method: local-minde);\n}\n",
    "a {\n  color: color(display-p3 0.9177905633 0.2107213818 0.1542354933);\n}\n"
);
test!(
    to_gamut_oklch_own_space_is_noop,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(oklch(80% 0.3 20), $method: local-minde);\n}\n",
    "a {\n  color: oklch(80% 0.3 20deg);\n}\n"
);
test!(
    to_gamut_oklch_in_srgb,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(oklch(80% 0.3 20), srgb, $method: local-minde);\n}\n",
    "a {\n  color: oklch(78.5610752438% 0.1247043722 19.5813943967deg);\n}\n"
);
test!(
    to_gamut_oklch_in_rec2020,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(oklch(80% 0.3 20), rec2020, $method: local-minde);\n}\n",
    "a {\n  color: oklch(78.609982363% 0.2340605092 19.9356857702deg);\n}\n"
);
test!(
    to_gamut_prophoto_in_display_p3,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(color(prophoto-rgb 1 0 0), display-p3, $method: local-minde);\n}\n",
    "a {\n  color: color(prophoto-rgb 0.7941689602 0.333114564 0.2810484031);\n}\n"
);
test!(
    to_gamut_lab_in_rgb_clip,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(lab(50% 150 -150), rgb, $method: clip);\n}\n",
    "a {\n  color: lab(54.9558781963% 88.4343401878 -69.1902378812);\n}\n"
);
test!(
    to_gamut_light_oklch_in_srgb,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(oklch(100% 0.3 20), srgb, $method: local-minde);\n}\n",
    "a {\n  color: oklch(100% 0 none);\n}\n"
);
test!(
    to_gamut_dark_oklch_in_display_p3,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(oklch(0% 0.3 20), display-p3, $method: local-minde);\n}\n",
    "a {\n  color: oklch(0% 0 none);\n}\n"
);
test!(
    to_gamut_with_missing_channel,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(color(srgb 1.5 none 0), $method: clip);\n}\n",
    "a {\n  color: color(srgb 1 none 0);\n}\n"
);
test!(
    to_gamut_in_hwb,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(color(display-p3 1 0 0), hwb, $method: local-minde);\n}\n",
    "a {\n  color: color(display-p3 0.9177905633 0.2107213818 0.1542179948);\n}\n"
);
test!(
    to_gamut_unbounded_space_arg,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(color(srgb 1.5 -0.5 0), lab, $method: clip);\n}\n",
    "a {\n  color: color(srgb 1.5 -0.5 0);\n}\n"
);

// ---------------------------------------------------------------------------
// color.adjust(), color.change(), color.scale()
// ---------------------------------------------------------------------------

test!(
    adjust_lab_lightness,
    "@use \"sass:color\";\na {\n  color: color.adjust(lab(50% 10 20), $lightness: 10%);\n}\n",
    "a {\n  color: lab(60% 10 20);\n}\n"
);
test!(
    adjust_lab_lightness_unitless,
    "@use \"sass:color\";\na {\n  color: color.adjust(lab(50% 10 20), $lightness: 10);\n}\n",
    "a {\n  color: lab(60% 10 20);\n}\n"
);
test!(
    adjust_lab_lightness_clamped,
    "@use \"sass:color\";\na {\n  color: color.adjust(lab(50% 10 20), $lightness: 60%);\n}\n",
    "a {\n  color: lab(100% 10 20);\n}\n"
);
test!(
    adjust_lab_a_percent,
    "@use \"sass:color\";\na {\n  color: color.adjust(lab(50% 10 20), $a: 10%);\n}\n",
    "a {\n  color: lab(50% 22.5 20);\n}\n"
);
test!(
    adjust_lab_a_unclamped,
    "@use \"sass:color\";\na {\n  color: color.adjust(lab(50% 10 20), $a: 200);\n}\n",
    "a {\n  color: lab(50% 210 20);\n}\n"
);
error!(
    adjust_lab_wrong_unit,
    "@use \"sass:color\";\na {\n  color: color.adjust(lab(50% 10 20), $a: 10px);\n}\n",
    "Error: $a: Expected 10px to have unit \"%\" or no units."
);
error!(
    adjust_lab_unknown_channel,
    "@use \"sass:color\";\na {\n  color: color.adjust(lab(50% 10 20), $red: 10);\n}\n",
    "Error: $red: Color space lab doesn't have a channel with this name."
);
error!(
    adjust_lab_missing_channel,
    "@use \"sass:color\";\na {\n  color: color.adjust(lab(50% none 20), $a: 10);\n}\n",
    "Error: $a: Because the CSS working group is still deciding on the best behavior, Sass doesn't currently support modifying missing channels (color: lab(50% none 20))."
);
test!(
    adjust_oklch_hue_and_chroma,
    "@use \"sass:color\";\na {\n  color: color.adjust(oklch(50% 0.1 20), $hue: 30deg, $chroma: -0.2);\n}\n",
    "a {\n  color: oklch(50% 0 50deg);\n}\n"
);
test!(
    adjust_oklch_hue_turn,
    "@use \"sass:color\";\na {\n  color: color.adjust(oklch(50% 0.1 20), $hue: 0.25turn);\n}\n",
    "a {\n  color: oklch(50% 0.1 110deg);\n}\n"
);
error!(
    adjust_lch_hue_wrong_unit,
    "@use \"sass:color\";\na {\n  color: color.adjust(lch(50% 10 20), $hue: 30px);\n}\n",
    "Error: $hue: Expected 30px to have an angle unit (deg, grad, rad, turn)."
);
test!(
    adjust_lch_chroma_lower_clamped,
    "@use \"sass:color\";\na {\n  color: color.adjust(lch(50% 10 20), $chroma: -30);\n}\n",
    "a {\n  color: lch(50% 0 20deg);\n}\n"
);
test!(
    adjust_srgb_red,
    "@use \"sass:color\";\na {\n  color: color.adjust(color(srgb 0.2 0.4 0.6), $red: 0.1);\n}\n",
    "a {\n  color: color(srgb 0.3 0.4 0.6);\n}\n"
);
test!(
    adjust_srgb_red_percent,
    "@use \"sass:color\";\na {\n  color: color.adjust(color(srgb 0.2 0.4 0.6), $red: 10%);\n}\n",
    "a {\n  color: color(srgb 0.3 0.4 0.6);\n}\n"
);
test!(
    adjust_srgb_unclamped,
    "@use \"sass:color\";\na {\n  color: color.adjust(color(srgb 0.2 0.4 0.6), $red: 2);\n}\n",
    "a {\n  color: color(srgb 2.2 0.4 0.6);\n}\n"
);
test!(
    adjust_xyz,
    "@use \"sass:color\";\na {\n  color: color.adjust(color(xyz 0.3 0.2 0.1), $y: 10%);\n}\n",
    "a {\n  color: color(xyz 0.3 0.3 0.1);\n}\n"
);
test!(
    adjust_legacy_in_non_legacy_space,
    "@use \"sass:color\";\na {\n  color: color.adjust(#cc0f35, $lightness: -10%, $space: lab);\n}\n",
    "a {\n  color: hsl(340.5143819459, 151.5823035854%, 26.9121364166%);\n}\n"
);
test!(
    adjust_legacy_in_oklch_space,
    "@use \"sass:color\";\na {\n  color: color.adjust(hsl(210 40% 60%), $chroma: 0.05, $space: oklch);\n}\n",
    "a {\n  color: hsl(209.1440879616, 68.3756020546%, 59.5464383723%);\n}\n"
);
test!(
    adjust_non_legacy_in_legacy_space,
    "@use \"sass:color\";\na {\n  color: color.adjust(lab(50% 10 20), $red: 10, $space: rgb);\n}\n",
    "a {\n  color: lab(51.1530371793% 14.3260997697 21.7626938931);\n}\n"
);
test!(
    adjust_non_legacy_without_space_uses_own,
    "@use \"sass:color\";\na {\n  color: color.adjust(lab(50% 10 20), $a: 10);\n}\n",
    "a {\n  color: lab(50% 20 20);\n}\n"
);
error!(
    adjust_non_legacy_legacy_keyword_errors,
    "@use \"sass:color\";\na {\n  color: color.adjust(lab(50% 10 20), $red: 10);\n}\n",
    "Error: $red: Color space lab doesn't have a channel with this name."
);
test!(
    adjust_alpha_of_lab,
    "@use \"sass:color\";\na {\n  color: color.adjust(lab(50% 10 20 / 0.5), $alpha: -0.3);\n}\n",
    "a {\n  color: lab(50% 10 20 / 0.2);\n}\n"
);
test!(
    adjust_alpha_percent_is_deprecated_unit,
    "@use \"sass:color\";\na {\n  color: color.adjust(rgba(255, 0, 0, 0.5), $alpha: -30%);\n}\n",
    "a {\n  color: rgba(255, 0, 0, 0);\n}\n"
);
test!(
    adjust_missing_alpha_is_zero_after_conversion,
    "@use \"sass:color\";\na {\n  color: color.adjust(lab(50% 10 20 / none), $blue: 0.5, $alpha: -0.5, $space: srgb);\n}\n",
    "a {\n  color: lab(53.9103908035% 29.3409141895 -48.0370101443 / 0);\n}\n"
);
error!(
    adjust_space_null_errors,
    "@use \"sass:color\";\na {\n  color: color.adjust(red, $lightness: 10%, $space: null);\n}\n",
    "Error: $space: null is not a string."
);
test!(
    change_lab_lightness,
    "@use \"sass:color\";\na {\n  color: color.change(lab(50% 10 20), $lightness: 60%);\n}\n",
    "a {\n  color: lab(60% 10 20);\n}\n"
);
test!(
    change_lab_lightness_unclamped,
    "@use \"sass:color\";\na {\n  color: color.change(lab(50% 10 20), $lightness: 150%);\n}\n",
    "a {\n  color: color-mix(in lab, color(xyz 2.87028635 2.9172111384 2.5646783747) 100%, black);\n}\n"
);
test!(
    change_lab_a_to_none,
    "@use \"sass:color\";\na {\n  color: color.change(oklab(0.5 0.1 0.1), $a: none);\n}\n",
    "a {\n  color: oklab(50% none 0.1);\n}\n"
);
test!(
    change_lab_alpha_to_none,
    "@use \"sass:color\";\na {\n  color: color.change(lab(50% 10 20), $alpha: none);\n}\n",
    "a {\n  color: lab(50% 10 20 / none);\n}\n"
);
test!(
    change_lab_alpha_percent,
    "@use \"sass:color\";\na {\n  color: color.change(lab(50% 10 20), $alpha: 50%);\n}\n",
    "a {\n  color: lab(50% 10 20 / 0.5);\n}\n"
);
error!(
    change_lab_alpha_out_of_range,
    "@use \"sass:color\";\na {\n  color: color.change(lab(50% 10 20), $alpha: 2);\n}\n",
    "Error: $alpha: Expected 2 to be within 0 and 1."
);
error!(
    change_lab_not_a_number,
    "@use \"sass:color\";\na {\n  color: color.change(lab(50% 10 20), $lightness: \"a\");\n}\n",
    "Error: $lightness: \"a\" is not a number or unquoted \"none\"."
);
test!(
    change_oklch_hue_none,
    "@use \"sass:color\";\na {\n  color: color.change(oklch(50% 0.1 20), $hue: none);\n}\n",
    "a {\n  color: oklch(50% 0.1 none);\n}\n"
);
test!(
    change_legacy_hue_to_none,
    "@use \"sass:color\";\na {\n  color: color.change(red, $hue: none);\n}\n",
    "a {\n  color: red;\n}\n"
);
test!(
    change_hsl_hue_to_none,
    "@use \"sass:color\";\na {\n  color: color.change(hsl(120 50% 50%), $hue: none);\n}\n",
    "a {\n  color: hsl(none 50% 50%);\n}\n"
);
test!(
    change_legacy_in_oklch_to_none,
    "@use \"sass:color\";\na {\n  color: color.change(#cc0f35, $lightness: none, $space: oklch);\n}\n",
    "a {\n  color: hsl(19.833005922, 831.9137340693%, 0.4331534011%);\n}\n"
);
test!(
    change_all_to_none,
    "@use \"sass:color\";\na {\n  color: color.change(lab(50% 10 20), $lightness: none, $a: none, $b: none);\n}\n",
    "a {\n  color: lab(none none none);\n}\n"
);
test!(
    change_srgb_percent,
    "@use \"sass:color\";\na {\n  color: color.change(color(srgb 0.2 0.4 0.6), $red: 50%);\n}\n",
    "a {\n  color: color(srgb 0.5 0.4 0.6);\n}\n"
);
test!(
    change_xyz_none,
    "@use \"sass:color\";\na {\n  color: color.change(color(xyz 0.3 0.2 0.1), $x: none);\n}\n",
    "a {\n  color: color(xyz none 0.2 0.1);\n}\n"
);
test!(
    scale_lab_lightness,
    "@use \"sass:color\";\na {\n  color: color.scale(lab(50% 10 20), $lightness: 50%);\n}\n",
    "a {\n  color: lab(75% 10 20);\n}\n"
);
test!(
    scale_lab_a,
    "@use \"sass:color\";\na {\n  color: color.scale(lab(50% 10 20), $a: 50%);\n}\n",
    "a {\n  color: lab(50% 67.5 20);\n}\n"
);
test!(
    scale_lab_a_negative,
    "@use \"sass:color\";\na {\n  color: color.scale(lab(50% 10 20), $a: -50%);\n}\n",
    "a {\n  color: lab(50% -57.5 20);\n}\n"
);
test!(
    scale_oklch_chroma,
    "@use \"sass:color\";\na {\n  color: color.scale(oklch(50% 0.1 20), $chroma: 50%);\n}\n",
    "a {\n  color: oklch(50% 0.25 20deg);\n}\n"
);
error!(
    scale_oklch_hue_errors,
    "@use \"sass:color\";\na {\n  color: color.scale(oklch(50% 0.1 20), $hue: 50%);\n}\n",
    "Error: $hue: Channel isn't scalable."
);
test!(
    scale_srgb_with_alpha,
    "@use \"sass:color\";\na {\n  color: color.scale(color(srgb 0.2 0.4 0.6), $red: -50%, $alpha: -50%);\n}\n",
    "a {\n  color: color(srgb 0.1 0.4 0.6 / 0.5);\n}\n"
);
test!(
    scale_legacy_in_oklch,
    "@use \"sass:color\";\na {\n  color: color.scale(#cc0f35, $lightness: 50%, $space: oklch);\n}\n",
    "a {\n  color: hsl(356.8557509174, 161.7107174734%, 78.5054775196%);\n}\n"
);
error!(
    scale_out_of_range_factor,
    "@use \"sass:color\";\na {\n  color: color.scale(lab(50% 10 20), $lightness: 150%);\n}\n",
    "Error: $lightness: Expected 150% to be within -100% and 100%."
);
error!(
    scale_unitless_factor,
    "@use \"sass:color\";\na {\n  color: color.scale(lab(50% 10 20), $lightness: 50);\n}\n",
    "Error: $lightness: Expected 50 to have unit \"%\"."
);
error!(
    scale_missing_channel,
    "@use \"sass:color\";\na {\n  color: color.scale(lab(50% none 20), $a: 50%);\n}\n",
    "Error: $a: Because the CSS working group is still deciding on the best behavior, Sass doesn't currently support modifying missing channels (color: lab(50% none 20))."
);

// ---------------------------------------------------------------------------
// color.mix() with a $method
// ---------------------------------------------------------------------------

test!(
    mix_lab,
    "@use \"sass:color\";\na {\n  color: color.mix(red, blue, $method: lab);\n}\n",
    "a {\n  color: hsl(315.3237547832, 136.091460698%, 32.0563868691%);\n}\n"
);
test!(
    mix_oklab_weight,
    "@use \"sass:color\";\na {\n  color: color.mix(red, blue, 25%, $method: oklab);\n}\n",
    "a {\n  color: rgb(31.6417475857%, 27.8837896877%, 82.1808614036%);\n}\n"
);
test!(
    mix_non_legacy_inputs_in_oklch,
    "@use \"sass:color\";\na {\n  color: color.mix(lab(50% 10 20), lch(60% 30 100), $method: oklch);\n}\n",
    "a {\n  color: lab(54.959052126% 4.0857519129 26.6999101243);\n}\n"
);
test!(
    mix_result_in_first_space,
    "@use \"sass:color\";\na {\n  color: color.mix(oklch(60% 0.15 200), color(display-p3 0.9 0.2 0.3), $method: srgb);\n}\n",
    "a {\n  color: oklch(44.6250251586% 0.0621834102 288.086740693deg);\n}\n"
);
test!(
    mix_display_p3,
    "@use \"sass:color\";\na {\n  color: color.mix(red, blue, $method: display-p3);\n}\n",
    "a {\n  color: rgb(50.1639701929%, 3.861662382%, 56.9228082255%);\n}\n"
);
test!(
    mix_xyz,
    "@use \"sass:color\";\na {\n  color: color.mix(red, blue, $method: xyz);\n}\n",
    "a {\n  color: rgb(73.5356983052%, 0%, 73.5356983052%);\n}\n"
);
test!(
    mix_xyz_d50,
    "@use \"sass:color\";\na {\n  color: color.mix(red, blue, $method: xyz-d50);\n}\n",
    "a {\n  color: rgb(73.5356983052%, 0%, 73.5356983052%);\n}\n"
);
test!(
    mix_srgb_linear,
    "@use \"sass:color\";\na {\n  color: color.mix(red, blue, $method: srgb-linear);\n}\n",
    "a {\n  color: rgb(73.5356983052%, 0%, 73.5356983052%);\n}\n"
);
test!(
    mix_lch_longer_hue,
    "@use \"sass:color\";\na {\n  color: color.mix(red, blue, $method: lch longer hue);\n}\n",
    "a {\n  color: hsl(163.8638094145, 1697.810274%, 2.8350391198%);\n}\n"
);
test!(
    mix_oklch_increasing_hue,
    "@use \"sass:color\";\na {\n  color: color.mix(red, blue, 30%, $method: oklch increasing hue);\n}\n",
    "a {\n  color: hsl(181.7941370341, 2445.1921053229%, 2.2852831574%);\n}\n"
);
test!(
    mix_oklch_decreasing_hue,
    "@use \"sass:color\";\na {\n  color: color.mix(red, blue, 30%, $method: oklch decreasing hue);\n}\n",
    "a {\n  color: hsl(278.3280552511, 140.7206707006%, 38.4827863205%);\n}\n"
);
test!(
    mix_missing_channels_take_other_side,
    "@use \"sass:color\";\na {\n  color: color.mix(lab(50% none 20), lab(60% 10 none), $method: lab);\n}\n",
    "a {\n  color: lab(55% 10 20);\n}\n"
);
test!(
    mix_missing_on_both_sides_stays_missing,
    "@use \"sass:color\";\na {\n  color: color.mix(oklch(50% 0.1 none), oklch(60% 0.2 none), $method: oklch);\n}\n",
    "a {\n  color: oklch(55% 0.15 none);\n}\n"
);
test!(
    mix_missing_alpha,
    "@use \"sass:color\";\na {\n  color: color.mix(lab(50% 10 20 / none), lab(60% 10 20 / 0.5), $method: lab);\n}\n",
    "a {\n  color: lab(80% 15 30 / 0.5);\n}\n"
);
test!(
    mix_premultiplied_alpha_lab,
    "@use \"sass:color\";\na {\n  color: color.mix(rgba(255, 0, 0, 0.5), rgba(0, 0, 255, 0.2), $method: lab);\n}\n",
    "a {\n  color: hsla(332.6538637951, 126.1420891197%, 38.5873148564%, 0.35);\n}\n"
);
test!(
    mix_weight_zero_returns_second,
    "@use \"sass:color\";\na {\n  color: color.mix(lab(50% 10 20), red, 0%, $method: lab);\n}\n",
    "a {\n  color: red;\n}\n"
);
test!(
    mix_weight_hundred_returns_first,
    "@use \"sass:color\";\na {\n  color: color.mix(lab(50% 10 20), red, 100%, $method: lab);\n}\n",
    "a {\n  color: lab(50% 10 20);\n}\n"
);
error!(
    mix_non_legacy_without_method,
    "@use \"sass:color\";\na {\n  color: color.mix(lab(50% 1 2), red);\n}\n",
    "Error: $color1: To use color.mix() with non-legacy color lab(50% 1 2), you must provide a $method."
);
error!(
    mix_second_non_legacy_without_method,
    "@use \"sass:color\";\na {\n  color: color.mix(red, lab(50% 1 2));\n}\n",
    "Error: $color2: To use color.mix() with non-legacy color lab(50% 1 2), you must provide a $method."
);
error!(
    mix_hue_method_on_rectangular_space,
    "@use \"sass:color\";\na {\n  color: color.mix(red, blue, $method: lab shorter hue);\n}\n",
    "Error: $method: Hue interpolation method \"HueInterpolationMethod.shorter hue\" may not be set for rectangular color space lab."
);
error!(
    mix_unknown_space,
    "@use \"sass:color\";\na {\n  color: color.mix(red, blue, $method: foo);\n}\n",
    "Error: $method: Unknown color space \"foo\"."
);
error!(
    mix_weight_out_of_range_with_method,
    "@use \"sass:color\";\na {\n  color: color.mix(red, blue, 120%, $method: lab);\n}\n",
    "Error: $weight: Expected 120% to be within 0% and 100%."
);
test!(
    mix_weight_wrong_unit_with_method,
    "@use \"sass:color\";\na {\n  color: color.mix(red, blue, 50px, $method: lab);\n}\n",
    "a {\n  color: hsl(315.3237547832, 136.091460698%, 32.0563868691%);\n}\n"
);

// ---------------------------------------------------------------------------
// color.invert(), color.complement(), color.grayscale()
// ---------------------------------------------------------------------------

test!(
    invert_lab,
    "@use \"sass:color\";\na {\n  color: color.invert(lab(50% 10 20), $space: lab);\n}\n",
    "a {\n  color: lab(50% -10 -20);\n}\n"
);
test!(
    invert_oklch,
    "@use \"sass:color\";\na {\n  color: color.invert(oklch(50% 0.1 20), $space: oklch);\n}\n",
    "a {\n  color: oklch(50% 0.1 200deg);\n}\n"
);
test!(
    invert_srgb,
    "@use \"sass:color\";\na {\n  color: color.invert(color(srgb 0.2 0.4 0.6), $space: srgb);\n}\n",
    "a {\n  color: color(srgb 0.8 0.6 0.4);\n}\n"
);
test!(
    invert_xyz,
    "@use \"sass:color\";\na {\n  color: color.invert(color(xyz 0.3 0.2 0.1), $space: xyz);\n}\n",
    "a {\n  color: color(xyz 0.7 0.8 0.9);\n}\n"
);
test!(
    invert_legacy_in_lab,
    "@use \"sass:color\";\na {\n  color: color.invert(#cc0f35, $space: lab);\n}\n",
    "a {\n  color: hsl(185.5591502211, 571.6449944415%, 11.2251093975%);\n}\n"
);
test!(
    invert_legacy_in_lab_weighted,
    "@use \"sass:color\";\na {\n  color: color.invert(#cc0f35, 30%, $space: lab);\n}\n",
    "a {\n  color: rgb(62.6747475604%, 37.0491083914%, 35.8336729413%);\n}\n"
);
test!(
    invert_lab_in_hwb,
    "@use \"sass:color\";\na {\n  color: color.invert(lab(50% 10 20), $space: hwb);\n}\n",
    "a {\n  color: lab(57.6368536936% -6.3215302943 -18.0560688502);\n}\n"
);
error!(
    invert_missing_channel,
    "@use \"sass:color\";\na {\n  color: color.invert(lab(50% none 20), $space: lab);\n}\n",
    "Error: $a: Because the CSS working group is still deciding on the best behavior, Sass doesn't currently support modifying missing channels (color: lab(50% none 20))."
);
error!(
    invert_non_legacy_without_space,
    "@use \"sass:color\";\na {\n  color: color.invert(lab(50% 10 20));\n}\n",
    "Error: $color: To use color.invert() with non-legacy color lab(50% 10 20), you must provide a $space."
);
test!(
    invert_weight_zero,
    "@use \"sass:color\";\na {\n  color: color.invert(lab(50% 10 20), 0%, $space: lab);\n}\n",
    "a {\n  color: lab(50% 10 20);\n}\n"
);
test!(
    invert_weight_unitless_with_space,
    "@use \"sass:color\";\na {\n  color: color.invert(red, 30, $space: lab);\n}\n",
    "a {\n  color: rgb(71.2985524721%, 39.051645349%, 30.3226829126%);\n}\n"
);
test!(
    complement_oklch,
    "@use \"sass:color\";\na {\n  color: color.complement(oklch(50% 0.1 20), oklch);\n}\n",
    "a {\n  color: oklch(50% 0.1 200deg);\n}\n"
);
test!(
    complement_lch,
    "@use \"sass:color\";\na {\n  color: color.complement(lch(50% 10 20), lch);\n}\n",
    "a {\n  color: lch(50% 10 200deg);\n}\n"
);
test!(
    complement_legacy_in_oklch,
    "@use \"sass:color\";\na {\n  color: color.complement(#cc0f35, oklch);\n}\n",
    "a {\n  color: hsl(184.0861330467, 628.566055297%, 8.5466208264%);\n}\n"
);
test!(
    complement_lab_in_hsl,
    "@use \"sass:color\";\na {\n  color: color.complement(lab(50% 10 20), hsl);\n}\n",
    "a {\n  color: lab(47.5247323017% -6.154477339 -18.582455879);\n}\n"
);
error!(
    complement_non_legacy_without_space,
    "@use \"sass:color\";\na {\n  color: color.complement(oklch(50% 0.1 20));\n}\n",
    "Error: $space: null is not a string."
);
error!(
    complement_rectangular_space,
    "@use \"sass:color\";\na {\n  color: color.complement(red, lab);\n}\n",
    "Error: $space: Color space lab doesn't have a hue channel."
);
error!(
    complement_missing_hue,
    "@use \"sass:color\";\na {\n  color: color.complement(oklch(50% 0.1 none), oklch);\n}\n",
    "Error: $hue: Because the CSS working group is still deciding on the best behavior, Sass doesn't currently support modifying missing channels (color: oklch(50% 0.1 none))."
);
test!(
    grayscale_lab,
    "@use \"sass:color\";\na {\n  color: color.grayscale(lab(50% 10 20));\n}\n",
    "a {\n  color: lab(50.1379045038% 0 0);\n}\n"
);
test!(
    grayscale_oklch,
    "@use \"sass:color\";\na {\n  color: color.grayscale(oklch(60% 0.15 200));\n}\n",
    "a {\n  color: oklch(60% 0 200deg);\n}\n"
);
test!(
    grayscale_display_p3,
    "@use \"sass:color\";\na {\n  color: color.grayscale(color(display-p3 0.9 0.2 0.3));\n}\n",
    "a {\n  color: color(display-p3 0.5311492557 0.5311492557 0.5311492557);\n}\n"
);
test!(
    grayscale_global_var,
    "a {\n  color: grayscale(var(--x));\n}\n",
    "a {\n  color: grayscale(var(--x));\n}\n"
);
error!(
    grayscale_module_var,
    "@use \"sass:color\";\na {\n  color: color.grayscale(var(--x));\n}\n",
    "Error: $color: var(--x) is not a color."
);
test!(
    opacity_global_var,
    "a {\n  color: opacity(var(--x));\n}\n",
    "a {\n  color: opacity(var(--x));\n}\n"
);
test!(
    module_opacity_of_lab,
    "@use \"sass:color\";\na {\n  color: color.opacity(lab(50% 10 20 / 0.5));\n}\n",
    "a {\n  color: 0.5;\n}\n"
);

// ---------------------------------------------------------------------------
// color.same() and color.ie-hex-str()
// ---------------------------------------------------------------------------

test!(
    same_lab_and_its_rgb,
    "@use \"sass:color\";\na {\n  color: color.same(lab(50% 10 20), color.to-space(lab(50% 10 20), rgb));\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    same_different_colors,
    "@use \"sass:color\";\na {\n  color: color.same(lab(50% 10 20), red);\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    same_missing_reads_zero,
    "@use \"sass:color\";\na {\n  color: color.same(lab(50% none 20), lab(50% 0 20));\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    same_missing_across_spaces,
    "@use \"sass:color\";\na {\n  color: color.same(oklch(50% 0.1 none), color.to-space(oklch(50% 0.1 0), lab));\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    same_xyz_alias,
    "@use \"sass:color\";\na {\n  color: color.same(color(xyz 0.3 0.2 0.1), color(xyz-d65 0.3 0.2 0.1));\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    same_round_trip_through_oklch,
    "@use \"sass:color\";\na {\n  color: color.same(#cc0f35, color.to-space(color.to-space(#cc0f35, oklch), rgb));\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    ie_hex_str_lab,
    "@use \"sass:color\";\na {\n  color: color.ie-hex-str(lab(50% 10 20));\n}\n",
    "a {\n  color: #FF907055;\n}\n"
);
test!(
    ie_hex_str_out_of_gamut_display_p3,
    "@use \"sass:color\";\na {\n  color: color.ie-hex-str(color(display-p3 1 0 0));\n}\n",
    "a {\n  color: #FFFF0B0C;\n}\n"
);
test!(
    ie_hex_str_with_alpha,
    "@use \"sass:color\";\na {\n  color: color.ie-hex-str(oklch(60% 0.15 200 / 0.5));\n}\n",
    "a {\n  color: #8000959D;\n}\n"
);

// ---------------------------------------------------------------------------
// Legacy functions reject non-legacy colors
// ---------------------------------------------------------------------------

error!(
    red_of_lab,
    "@use \"sass:color\";\na {\n  color: color.red(lab(50% 1 2));\n}\n",
    "Error: color.red() is only supported for legacy colors. Please use color.channel() instead with an explicit $space argument."
);
error!(
    global_hue_of_lab,
    "a {\n  color: hue(lab(50% 1 2));\n}\n",
    "Error: color.hue() is only supported for legacy colors. Please use color.channel() instead with an explicit $space argument."
);
error!(
    whiteness_of_srgb,
    "@use \"sass:color\";\na {\n  color: color.whiteness(color(srgb 1 0 0));\n}\n",
    "Error: color.whiteness() is only supported for legacy colors. Please use color.channel() instead with an explicit $space argument."
);
error!(
    alpha_of_lab,
    "a {\n  color: alpha(lab(50% 10 20));\n}\n",
    "Error: alpha() is only supported for legacy colors. Please use color.channel() instead."
);
error!(
    module_alpha_of_lab,
    "@use \"sass:color\";\na {\n  color: color.alpha(lab(50% 10 20));\n}\n",
    "Error: color.alpha() is only supported for legacy colors. Please use color.channel() instead."
);
error!(
    lighten_lab,
    "a {\n  color: lighten(lab(50% 10 20), 10%);\n}\n",
    "Error: lighten() is only supported for legacy colors. Please use color.adjust() instead with an explicit $space argument."
);
error!(
    darken_oklch,
    "a {\n  color: darken(oklch(50% 0.1 20), 10%);\n}\n",
    "Error: darken() is only supported for legacy colors. Please use color.adjust() instead with an explicit $space argument."
);
error!(
    saturate_srgb,
    "a {\n  color: saturate(color(srgb 1 0 0), 10%);\n}\n",
    "Error: saturate() is only supported for legacy colors. Please use color.adjust() instead with an explicit $space argument."
);
error!(
    adjust_hue_lab,
    "a {\n  color: adjust-hue(lab(50% 10 20), 10);\n}\n",
    "Error: adjust-hue() is only supported for legacy colors. Please use color.adjust() instead with an explicit $space argument."
);
error!(
    opacify_lab,
    "a {\n  color: opacify(lab(50% 10 20), 0.1);\n}\n",
    "Error: opacify() is only supported for legacy colors. Please use color.adjust() instead with an explicit $space argument."
);
error!(
    fade_out_lab,
    "a {\n  color: fade-out(lab(50% 10 20), 0.1);\n}\n",
    "Error: fade-out() is only supported for legacy colors. Please use color.adjust() instead with an explicit $space argument."
);
error!(
    global_invert_lab,
    "a {\n  color: invert(lab(50% 10 20));\n}\n",
    "Error: $color: To use color.invert() with non-legacy color lab(50% 10 20), you must provide a $space."
);
error!(
    global_mix_lab,
    "a {\n  color: mix(lab(50% 10 20), red);\n}\n",
    "Error: $color1: To use color.mix() with non-legacy color lab(50% 10 20), you must provide a $method."
);
error!(
    global_complement_lab,
    "a {\n  color: complement(lab(50% 10 20));\n}\n", "Error: $space: null is not a string."
);
test!(
    global_grayscale_lab_works,
    "a {\n  color: grayscale(lab(50% 10 20));\n}\n",
    "a {\n  color: lab(50.1379045038% 0 0);\n}\n"
);
test!(
    opacity_of_lab_works,
    "a {\n  color: opacity(lab(50% 10 20 / 0.3));\n}\n",
    "a {\n  color: 0.3;\n}\n"
);
