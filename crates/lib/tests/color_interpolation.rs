//! `color.invert()` and `color.complement()` with a `$space`, and
//! `color.mix()` with a `$method`: the operations that convert into a legacy
//! space, work there, and convert back.
//!
//! Every expectation was verified against dart-sass 1.103.1.

#[macro_use]
mod macros;

// ---------------------------------------------------------------------------
// color.invert($color, $weight, $space)
// ---------------------------------------------------------------------------

test!(
    invert_default_is_rgb,
    "@use \"sass:color\";\na {\n  color: color.invert(#cc0f35);\n}\n",
    "a {\n  color: #33f0ca;\n}\n"
);
test!(
    invert_full_weight_is_the_same_in_every_space,
    "@use \"sass:color\";\na {\n  color: color.invert(#cc0f35, $space: hsl) color.invert(#cc0f35, $space: hwb) color.invert(#cc0f35, $space: rgb);\n}\n",
    "a {\n  color: #33f0ca #33f0ca #33f0ca;\n}\n"
);
test!(
    invert_partial_weight_rgb,
    "@use \"sass:color\";\na {\n  color: color.invert(#cc0f35, 30%, rgb);\n}\n",
    "a {\n  color: rgb(62%, 32.3529411765%, 38.3137254902%);\n}\n"
);
test!(
    invert_partial_weight_hsl,
    "@use \"sass:color\";\na {\n  color: color.invert(#cc0f35, 30%, hsl);\n}\n",
    "a {\n  color: rgb(79.6614558152%, 6.4625302176%, 87.8904109589%);\n}\n"
);
test!(
    invert_partial_weight_hwb,
    "@use \"sass:color\";\na {\n  color: color.invert(#cc0f35, 30%, hwb);\n}\n",
    "a {\n  color: rgb(76.7450980392%, 10.1176470588%, 84.2352941176%);\n}\n"
);
test!(
    invert_half_weight_hsl_interpolates_hue,
    "@use \"sass:color\";\na {\n  color: color.invert(hsl(120 100% 50%), 50%, hsl);\n}\n",
    "a {\n  color: hsl(210, 100%, 50%);\n}\n"
);
test!(
    invert_hsl_input_keeps_hsl_form,
    "@use \"sass:color\";\na {\n  color: color.invert(hsl(120, 50%, 50%));\n}\n",
    "a {\n  color: hsl(300, 50%, 50%);\n}\n"
);
test!(
    invert_hwb_input_in_hwb_space,
    "@use \"sass:color\";\na {\n  color: color.invert(color.hwb(120 20% 30%), $space: hwb);\n}\n",
    "a {\n  color: hsl(300, 55.5555555556%, 55%);\n}\n"
);
test!(
    invert_partial_weight_in_hwb_space,
    "@use \"sass:color\";\na {\n  color: color.invert(hsl(120 50% 50%), 40%, $space: hwb);\n}\n",
    "a {\n  color: hsl(192, 50%, 50%);\n}\n"
);
test!(
    invert_with_alpha_in_hsl_space,
    "@use \"sass:color\";\na {\n  color: color.invert(rgba(200, 10, 50, 0.4), 30%, hsl);\n}\n",
    "a {\n  color: rgba(79.2212885154%, 4.4257703081%, 88.5154061625%, 0.4);\n}\n"
);
test!(
    invert_zero_weight_with_space_is_identity,
    "@use \"sass:color\";\na {\n  color: color.invert(#000, 0%, hsl);\n}\n",
    "a {\n  color: #000;\n}\n"
);
test!(
    invert_zero_weight_without_space_still_converts,
    "@use \"sass:color\";\na {\n  color: color.invert(hsl(0 0% 50%), 0%);\n}\n",
    "a {\n  color: hsl(none 0% 50%);\n}\n"
);
test!(
    invert_gray_in_rgb_space,
    "@use \"sass:color\";\na {\n  color: color.invert(hsl(0 0% 50%), $space: rgb);\n}\n",
    "a {\n  color: hsl(0, 0%, 50%);\n}\n"
);
test!(
    invert_gray_hsl_in_hsl_space_has_a_hue,
    "@use \"sass:color\";\na {\n  color: color.invert(hsl(0 0% 50%), $space: hsl);\n}\n",
    "a {\n  color: hsl(180, 0%, 50%);\n}\n"
);
test!(
    invert_out_of_gamut_hsl,
    "@use \"sass:color\";\na {\n  color: color.invert(hsl(120 50% 150%));\n}\n",
    "a {\n  color: hsl(300, 50%, -50%);\n}\n"
);
test!(
    invert_half_weight_makes_gray_with_missing_hue,
    "@use \"sass:color\";\na {\n  color: color.invert(hsl(120 50% 50%), 50%);\n}\n",
    "a {\n  color: hsl(none 0% 50%);\n}\n"
);
test!(
    invert_named_arguments,
    "@use \"sass:color\";\na {\n  color: color.invert($color: #cc0f35, $space: hwb, $weight: 20%);\n}\n",
    "a {\n  color: rgb(82.8235294118%, 8.7058823529%, 68.0784313725%);\n}\n"
);
test!(
    invert_plain_css_with_default_weight,
    "@use \"sass:color\";\na {\n  color: color.invert(1, 100%);\n}\n",
    "a {\n  color: invert(1);\n}\n"
);
error!(
    invert_black_in_hsl_space,
    "@use \"sass:color\";\na {\n  color: color.invert(#000, $space: hsl);\n}\n",
    "Error: $hue: Because the CSS working group is still deciding on the best behavior, Sass doesn't currently support modifying missing channels (color: hsl(none 0% 0%))."
);
error!(
    invert_gray_in_hwb_space,
    "@use \"sass:color\";\na {\n  color: color.invert(#808080, $space: hwb);\n}\n",
    "Error: $hue: Because the CSS working group is still deciding on the best behavior, Sass doesn't currently support modifying missing channels (color: hwb(none 50.1960784314% 49.8039215686%))."
);
error!(
    invert_transparent_black_in_hsl_space,
    "@use \"sass:color\";\na {\n  color: color.invert(rgba(0, 0, 0, 0.4), $space: hsl);\n}\n",
    "Error: $hue: Because the CSS working group is still deciding on the best behavior, Sass doesn't currently support modifying missing channels (color: hsl(none 0% 0% / 0.4))."
);
error!(
    invert_quoted_space,
    "@use \"sass:color\";\na {\n  color: color.invert(#cc0f35, $space: \"hsl\");\n}\n",
    "Error: $space: Expected \"hsl\" to be an unquoted string."
);
error!(
    invert_unknown_space,
    "@use \"sass:color\";\na {\n  color: color.invert(#cc0f35, $space: nope);\n}\n",
    "Error: $space: Unknown color space \"nope\"."
);
error!(
    invert_weight_out_of_range,
    "@use \"sass:color\";\na {\n  color: color.invert(#cc0f35, 130%, hsl);\n}\n",
    "Error: $weight: Expected 130% to be within 0% and 100%."
);
error!(
    invert_plain_css_with_weight,
    "@use \"sass:color\";\na {\n  color: color.invert(1, 50%);\n}\n",
    "Error: Only one argument may be passed to the plain-CSS invert() function."
);

// ---------------------------------------------------------------------------
// color.complement($color, $space)
// ---------------------------------------------------------------------------

test!(
    complement_default,
    "@use \"sass:color\";\na {\n  color: color.complement(#cc0f35);\n}\n",
    "a {\n  color: #0fcca6;\n}\n"
);
test!(
    complement_hwb_space,
    "@use \"sass:color\";\na {\n  color: color.complement(#cc0f35, $space: hwb);\n}\n",
    "a {\n  color: #0fcca6;\n}\n"
);
test!(
    complement_hwb_input,
    "@use \"sass:color\";\na {\n  color: color.complement(color.hwb(120 20% 30%));\n}\n",
    "a {\n  color: hsl(300, 55.5555555556%, 45%);\n}\n"
);
test!(
    complement_with_alpha,
    "@use \"sass:color\";\na {\n  color: color.complement(rgba(200, 10, 50, 0.4), $space: hsl);\n}\n",
    "a {\n  color: rgba(3.9215686275%, 78.431372549%, 62.7450980392%, 0.4);\n}\n"
);
test!(
    complement_black_without_space_reads_hue_as_zero,
    "@use \"sass:color\";\na {\n  color: color.complement(#000);\n}\n",
    "a {\n  color: black;\n}\n"
);
test!(
    complement_gray_hsl_keeps_its_hue,
    "@use \"sass:color\";\na {\n  color: color.complement(hsl(0 0% 0%), $space: hsl);\n}\n",
    "a {\n  color: hsl(180, 0%, 0%);\n}\n"
);
test!(
    complement_out_of_gamut,
    "@use \"sass:color\";\na {\n  color: color.complement(hsl(120 50% 150%));\n}\n",
    "a {\n  color: hsl(300, 50%, 150%);\n}\n"
);
error!(
    complement_black_in_hsl_space,
    "@use \"sass:color\";\na {\n  color: color.complement(#000, $space: hsl);\n}\n",
    "Error: $hue: Because the CSS working group is still deciding on the best behavior, Sass doesn't currently support modifying missing channels (color: hsl(none 0% 0%))."
);
error!(
    complement_black_in_hwb_space,
    "@use \"sass:color\";\na {\n  color: color.complement(#000, $space: hwb);\n}\n",
    "Error: $hue: Because the CSS working group is still deciding on the best behavior, Sass doesn't currently support modifying missing channels (color: hwb(none 0% 100%))."
);
error!(
    complement_missing_hue,
    "@use \"sass:color\";\na {\n  color: color.complement(color.invert(hsl(0 0% 50%)));\n}\n",
    "Error: $hue: Because the CSS working group is still deciding on the best behavior, Sass doesn't currently support modifying missing channels (color: hsl(none 0% 50%))."
);
error!(
    complement_rgb_space,
    "@use \"sass:color\";\na {\n  color: color.complement(#cc0f35, $space: rgb);\n}\n",
    "Error: $space: Color space rgb doesn't have a hue channel."
);
error!(
    complement_rectangular_non_legacy_space,
    "@use \"sass:color\";\na {\n  color: color.complement(#cc0f35, $space: lab);\n}\n",
    "Error: $space: Color space lab doesn't have a hue channel."
);
error!(
    complement_quoted_space,
    "@use \"sass:color\";\na {\n  color: color.complement(#cc0f35, $space: \"hsl\");\n}\n",
    "Error: $space: Expected \"hsl\" to be an unquoted string."
);
error!(
    complement_too_many_args,
    "@use \"sass:color\";\na {\n  color: color.complement(#cc0f35, hsl, 1);\n}\n",
    "Error: Only 2 arguments allowed, but 3 were passed."
);

// ---------------------------------------------------------------------------
// color.mix($color1, $color2, $weight, $method)
// ---------------------------------------------------------------------------

test!(
    mix_legacy,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34);\n}\n",
    "a {\n  color: rgb(43.5294117647%, 36.4705882353%, 20.5882352941%);\n}\n"
);
test!(
    mix_method_rgb_equals_legacy_when_opaque,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, $method: rgb);\n}\n",
    "a {\n  color: rgb(43.5294117647%, 36.4705882353%, 20.5882352941%);\n}\n"
);
test!(
    mix_method_rgb_premultiplies_alpha,
    "@use \"sass:color\";\na {\n  color: color.mix(rgba(200, 10, 50, 0.4), rgba(18, 171, 52, 0.8), 30%, rgb);\n}\n",
    "a {\n  color: rgba(19.6539792388%, 55.9169550173%, 20.2537485582%, 0.68);\n}\n"
);
test!(
    mix_legacy_alpha_differs_from_rgb_method,
    "@use \"sass:color\";\na {\n  color: color.mix(rgba(200, 10, 50, 0.4), rgba(18, 171, 52, 0.8), 30%);\n}\n",
    "a {\n  color: rgba(18.1338742394%, 57.261663286%, 20.2704530088%, 0.68);\n}\n"
);
test!(
    mix_method_hsl,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, $method: hsl);\n}\n",
    "a {\n  color: rgb(72.7427977787%, 73.4507501631%, 6.5492498369%);\n}\n"
);
test!(
    mix_method_hwb,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, $method: hwb);\n}\n",
    "a {\n  color: rgb(72.8197945845%, 73.5294117647%, 6.4705882353%);\n}\n"
);
test!(
    mix_method_hsl_shorter_hue_is_default,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, $method: hsl shorter hue);\n}\n",
    "a {\n  color: rgb(72.7427977787%, 73.4507501631%, 6.5492498369%);\n}\n"
);
test!(
    mix_method_hsl_longer_hue,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, $method: hsl longer hue);\n}\n",
    "a {\n  color: rgb(7.2572022213%, 6.5492498369%, 73.4507501631%);\n}\n"
);
test!(
    mix_method_hwb_longer_hue,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, $method: hwb longer hue);\n}\n",
    "a {\n  color: rgb(7.1802054155%, 6.4705882353%, 73.5294117647%);\n}\n"
);
test!(
    mix_method_is_case_insensitive,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, $method: HSL LONGER HUE);\n}\n",
    "a {\n  color: rgb(7.2572022213%, 6.5492498369%, 73.4507501631%);\n}\n"
);
test!(
    mix_method_in_parentheses,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, $method: (hsl longer hue));\n}\n",
    "a {\n  color: rgb(7.2572022213%, 6.5492498369%, 73.4507501631%);\n}\n"
);
test!(
    mix_method_with_weight,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, 30%, hsl);\n}\n",
    "a {\n  color: rgb(39.1287824719%, 70.8751007252%, 6.7719580983%);\n}\n"
);
test!(
    mix_method_with_named_weight,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, $method: hsl, $weight: 25%);\n}\n",
    "a {\n  color: rgb(31.1481883469%, 70.2351214458%, 6.8237020836%);\n}\n"
);
test!(
    mix_method_with_alpha_in_hsl,
    "@use \"sass:color\";\na {\n  color: color.mix(rgba(200, 10, 50, 0.4), rgba(18, 171, 52, 0.8), 30%, hsl);\n}\n",
    "a {\n  color: rgba(38.2602071363%, 69.0087522898%, 6.5621819662%, 0.68);\n}\n"
);
test!(
    mix_hue_shorter_crosses_zero,
    "@use \"sass:color\";\na {\n  color: color.mix(hsl(350 100% 50%), hsl(10 100% 50%), $method: hsl);\n}\n",
    "a {\n  color: hsl(0, 100%, 50%);\n}\n"
);
test!(
    mix_hue_longer_crosses_zero,
    "@use \"sass:color\";\na {\n  color: color.mix(hsl(350 100% 50%), hsl(10 100% 50%), $method: hsl longer hue);\n}\n",
    "a {\n  color: hsl(180, 100%, 50%);\n}\n"
);
test!(
    mix_hue_increasing,
    "@use \"sass:color\";\na {\n  color: color.mix(hsl(200 100% 50%), hsl(10 100% 50%), $method: hsl increasing hue);\n}\n",
    "a {\n  color: hsl(285, 100%, 50%);\n}\n"
);
test!(
    mix_hue_decreasing,
    "@use \"sass:color\";\na {\n  color: color.mix(hsl(200 100% 50%), hsl(10 100% 50%), $method: hsl decreasing hue);\n}\n",
    "a {\n  color: hsl(105, 100%, 50%);\n}\n"
);
test!(
    mix_hue_shorter_exactly_180_apart,
    "@use \"sass:color\";\na {\n  color: color.mix(hsl(10 100% 50%), hsl(190 100% 50%), $method: hsl longer hue);\n}\n",
    "a {\n  color: hsl(100, 100%, 50%);\n}\n"
);
test!(
    mix_missing_hue_takes_the_other_hue,
    "@use \"sass:color\";\na {\n  color: color.mix(#000, #cc0f35, $method: hsl);\n}\n",
    "a {\n  color: rgb(30.7352941176%, 12.2058823529%, 15.931372549%);\n}\n"
);
test!(
    mix_both_hues_missing,
    "@use \"sass:color\";\na {\n  color: color.mix(#000, #fff, $method: hsl);\n}\n",
    "a {\n  color: rgb(50%, 50%, 50%);\n}\n"
);
test!(
    mix_result_is_in_first_colors_space,
    "@use \"sass:color\";\na {\n  color: color.mix(hsl(120 50% 50%), #12ab34, $method: hsl);\n}\n",
    "a {\n  color: hsl(126.6666666667, 65.4761904762%, 43.5294117647%);\n}\n"
);
test!(
    mix_zero_weight_returns_second_color,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, 0%, hsl);\n}\n",
    "a {\n  color: #12ab34;\n}\n"
);
test!(
    mix_full_weight_returns_first_color,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, 100%, hsl);\n}\n",
    "a {\n  color: #cc0f35;\n}\n"
);
test!(
    mix_fully_transparent_colors_give_nan_channels,
    "@use \"sass:color\";\na {\n  color: color.mix(rgba(200, 10, 50, 0), rgba(18, 171, 52, 0), $method: hsl);\n}\n",
    "a {\n  color: hsla(calc(NaN), calc(NaN * 1%), calc(NaN * 1%), 0);\n}\n"
);
test!(
    mix_out_of_gamut_hsl,
    "@use \"sass:color\";\na {\n  color: mix(hsl(120 50% 150%), #cc0f35);\n}\n",
    "a {\n  color: hsl(328.6255924171, 879.1666666667%, 96.4705882353%);\n}\n"
);
error!(
    mix_method_missing_hue_keyword,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, $method: hsl longer);\n}\n",
    "Error: $method: Expected unquoted string \"hue\" after (hsl longer)."
);
error!(
    mix_method_hue_for_rectangular_space,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, $method: rgb shorter hue);\n}\n",
    "Error: $method: Hue interpolation method \"HueInterpolationMethod.shorter hue\" may not be set for rectangular color space rgb."
);
error!(
    mix_method_unknown_hue_method,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, $method: hsl bogus hue);\n}\n",
    "Error: $method: Unknown hue interpolation method bogus."
);
error!(
    mix_method_trailing_words,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, $method: hsl longer hue extra);\n}\n",
    "Error: $method: Expected nothing after \"hue\" in (hsl longer hue extra)."
);
error!(
    mix_method_comma_list,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, $method: (hsl, longer, hue));\n}\n",
    "Error: $method: Expected a space-separated list, was (hsl, longer, hue)"
);
error!(
    mix_method_bracketed_list,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, $method: [hsl longer hue]);\n}\n",
    "Error: $method: Expected an unbracketed list, was [hsl longer hue]"
);
error!(
    mix_method_empty_list,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, $method: ());\n}\n",
    "Error: $method: Expected a color interpolation method, got an empty list."
);
error!(
    mix_method_quoted,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, $method: \"hsl\");\n}\n",
    "Error: $method: Expected \"hsl\" to be an unquoted string."
);
error!(
    mix_method_unknown_space,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, $method: nope);\n}\n",
    "Error: $method: Unknown color space \"nope\"."
);
test!(
    mix_method_oklch,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, $method: oklch);\n}\n",
    "a {\n  color: hsl(42.4826732136, 247.3743230932%, 20.6427981471%);\n}\n"
);
error!(
    mix_weight_out_of_range,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, 120%, hsl);\n}\n",
    "Error: $weight: Expected 120% to be within 0% and 100%."
);
error!(
    mix_too_many_args,
    "@use \"sass:color\";\na {\n  color: color.mix(#cc0f35, #12ab34, 50%, hsl, 1);\n}\n",
    "Error: Only 4 arguments allowed, but 5 were passed."
);
