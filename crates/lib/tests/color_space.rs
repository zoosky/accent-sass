//! The color-space introspection family: `color.space()`, `color.to-space()`,
//! `color.is-legacy()`, `color.is-in-gamut()`, `color.to-gamut()`, and
//! `color.same()`, plus the space-driven serialization rules and missing-hue
//! semantics they depend on.
//!
//! Every expectation was verified against dart-sass 1.103.1.

#[macro_use]
mod macros;

// ---------------------------------------------------------------------------
// color.space()
// ---------------------------------------------------------------------------

test!(
    space_of_hex_is_rgb,
    "@use \"sass:color\";\na {\n  color: color.space(#cc0f35);\n}\n",
    "a {\n  color: rgb;\n}\n"
);
test!(
    space_of_named_color_is_rgb,
    "@use \"sass:color\";\na {\n  color: color.space(red);\n}\n",
    "a {\n  color: rgb;\n}\n"
);
test!(
    space_of_hsl_is_hsl,
    "@use \"sass:color\";\na {\n  color: color.space(hsl(1 2% 3%));\n}\n",
    "a {\n  color: hsl;\n}\n"
);
test!(
    space_of_hwb_is_hwb,
    "@use \"sass:color\";\na {\n  color: color.space(color.hwb(1 2% 3%));\n}\n",
    "a {\n  color: hwb;\n}\n"
);
test!(
    space_of_global_hwb_is_hwb,
    "@use \"sass:color\";\na {\n  color: color.space(hwb(120 20% 30%));\n}\n",
    "a {\n  color: hwb;\n}\n"
);
test!(
    space_survives_adjust,
    "@use \"sass:color\";\na {\n  color: color.space(color.adjust(hsl(1 2% 3%), $red: 10));\n}\n",
    "a {\n  color: hsl;\n}\n"
);
test!(
    space_survives_change_with_space_argument,
    "@use \"sass:color\";\na {\n  color: color.space(color.change(#cc0f35, $hue: 10deg, $space: hwb));\n}\n",
    "a {\n  color: rgb;\n}\n"
);
test!(
    space_of_legacy_mix_is_rgb,
    "@use \"sass:color\";\na {\n  color: color.space(color.mix(hsl(1 2% 3%), #cc0f35));\n}\n",
    "a {\n  color: rgb;\n}\n"
);
test!(
    space_of_mix_with_method_is_first_colors,
    "@use \"sass:color\";\na {\n  color: color.space(color.mix(hsl(1 2% 3%), #cc0f35, $method: hwb));\n}\n",
    "a {\n  color: hsl;\n}\n"
);
test!(
    space_of_rgba_from_color_is_rgb,
    "@use \"sass:color\";\na {\n  color: color.space(rgba(hsl(120 100% 50%), 0.5));\n}\n",
    "a {\n  color: rgb;\n}\n"
);
test!(
    space_of_to_gamut_is_input_space,
    "@use \"sass:color\";\na {\n  color: color.space(color.to-gamut(hsl(120 200% 50%), $space: rgb, $method: clip));\n}\n",
    "a {\n  color: hsl;\n}\n"
);
error!(
    space_non_color,
    "@use \"sass:color\";\na {\n  color: color.space(1);\n}\n", "Error: $color: 1 is not a color."
);
error!(
    space_too_many_args,
    "@use \"sass:color\";\na {\n  color: color.space(#cc0f35, hsl);\n}\n",
    "Error: Only 1 argument allowed, but 2 were passed."
);

// ---------------------------------------------------------------------------
// color.to-space()
// ---------------------------------------------------------------------------

test!(
    to_space_hsl_serializes_as_hsl,
    "@use \"sass:color\";\na {\n  color: color.to-space(#cc0f35, hsl);\n}\n",
    "a {\n  color: hsl(347.9365079365, 86.301369863%, 42.9411764706%);\n}\n"
);
test!(
    to_space_hwb_with_whole_rgb_keeps_hex,
    "@use \"sass:color\";\na {\n  color: color.to-space(#cc0f35, hwb);\n}\n",
    "a {\n  color: #cc0f35;\n}\n"
);
test!(
    to_space_same_space_is_identity,
    "@use \"sass:color\";\na {\n  color: color.to-space(#cc0f35, rgb);\n}\n",
    "a {\n  color: #cc0f35;\n}\n"
);
test!(
    to_space_black_to_hsl_has_zero_hue_not_missing,
    "@use \"sass:color\";\na {\n  color: color.to-space(#000, hsl);\n}\n",
    "a {\n  color: hsl(0, 0%, 0%);\n}\n"
);
test!(
    to_space_black_to_hwb,
    "@use \"sass:color\";\na {\n  color: color.to-space(#000, hwb);\n}\n",
    "a {\n  color: black;\n}\n"
);
test!(
    to_space_hsl_to_rgb,
    "@use \"sass:color\";\na {\n  color: color.to-space(hsl(120 50% 50%), rgb);\n}\n",
    "a {\n  color: rgb(25%, 75%, 25%);\n}\n"
);
test!(
    to_space_hsl_to_hwb,
    "@use \"sass:color\";\na {\n  color: color.to-space(hsl(120 50% 50%), hwb);\n}\n",
    "a {\n  color: hsl(120, 50%, 50%);\n}\n"
);
test!(
    to_space_hwb_to_hsl,
    "@use \"sass:color\";\na {\n  color: color.to-space(color.hwb(120 20% 30%), hsl);\n}\n",
    "a {\n  color: hsl(120, 55.5555555556%, 45%);\n}\n"
);
test!(
    to_space_with_alpha,
    "@use \"sass:color\";\na {\n  color: color.to-space(rgba(200, 10, 50, 0.4), hsl);\n}\n",
    "a {\n  color: hsla(347.3684210526, 90.4761904762%, 41.1764705882%, 0.4);\n}\n"
);
test!(
    to_space_keeps_out_of_gamut_hsl,
    "@use \"sass:color\";\na {\n  color: color.to-space(hsl(120 50% 150%), rgb);\n}\n",
    "a {\n  color: hsl(120, 50%, 150%);\n}\n"
);
test!(
    to_space_achromatic_hsl_to_hwb_drops_hue,
    "@use \"sass:color\";\na {\n  color: color.to-space(hsl(120 0% 50%), hwb);\n}\n",
    "a {\n  color: hsl(0, 0%, 50%);\n}\n"
);
test!(
    to_space_name_is_case_insensitive,
    "@use \"sass:color\";\na {\n  color: color.to-space(#cc0f35, HSL);\n}\n",
    "a {\n  color: hsl(347.9365079365, 86.301369863%, 42.9411764706%);\n}\n"
);
test!(
    to_space_hue_of_converted_black_is_zero,
    "@use \"sass:color\";\na {\n  color: color.adjust(color.to-space(#000, hsl), $hue: 10deg);\n}\n",
    "a {\n  color: hsl(10, 0%, 0%);\n}\n"
);
error!(
    to_space_quoted_space,
    "@use \"sass:color\";\na {\n  color: color.to-space(#cc0f35, \"hsl\");\n}\n",
    "Error: $space: Expected \"hsl\" to be an unquoted string."
);
error!(
    to_space_missing_space,
    "@use \"sass:color\";\na {\n  color: color.to-space(#cc0f35);\n}\n",
    "Error: Missing argument $space."
);
error!(
    to_space_unknown_space,
    "@use \"sass:color\";\na {\n  color: color.to-space(#cc0f35, nope);\n}\n",
    "Error: $space: Unknown color space \"nope\"."
);
test!(
    to_space_oklch,
    "@use \"sass:color\";\na {\n  color: color.to-space(#cc0f35, oklch);\n}\n",
    "a {\n  color: oklch(53.8574934869% 0.210710041 20.5019425917deg);\n}\n"
);

// ---------------------------------------------------------------------------
// color.is-legacy()
// ---------------------------------------------------------------------------

test!(
    is_legacy_hex,
    "@use \"sass:color\";\na {\n  color: color.is-legacy(#cc0f35);\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    is_legacy_hwb,
    "@use \"sass:color\";\na {\n  color: color.is-legacy(color.hwb(1 2% 3%));\n}\n",
    "a {\n  color: true;\n}\n"
);
error!(
    is_legacy_non_color,
    "@use \"sass:color\";\na {\n  color: color.is-legacy(1);\n}\n",
    "Error: $color: 1 is not a color."
);

// ---------------------------------------------------------------------------
// color.is-in-gamut()
// ---------------------------------------------------------------------------

test!(
    is_in_gamut_hex,
    "@use \"sass:color\";\na {\n  color: color.is-in-gamut(#cc0f35);\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    is_in_gamut_saturation_above_100,
    "@use \"sass:color\";\na {\n  color: color.is-in-gamut(hsl(120 200% 50%));\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    is_in_gamut_lightness_above_100,
    "@use \"sass:color\";\na {\n  color: color.is-in-gamut(hsl(0 100% 120%));\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    is_in_gamut_negative_lightness,
    "@use \"sass:color\";\na {\n  color: color.is-in-gamut(hsl(0 100% -20%));\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    is_in_gamut_rgb_is_clamped_at_parse,
    "@use \"sass:color\";\na {\n  color: color.is-in-gamut(rgb(300 0 0));\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    is_in_gamut_in_another_space,
    "@use \"sass:color\";\na {\n  color: color.is-in-gamut(hsl(120 200% 50%), rgb);\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    is_in_gamut_hwb_sum_over_100_is_in_gamut,
    "@use \"sass:color\";\na {\n  color: color.is-in-gamut(color.hwb(120 60% 60%));\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    is_in_gamut_negative_whiteness,
    "@use \"sass:color\";\na {\n  color: color.is-in-gamut(hwb(120 -10% 60%));\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    is_in_gamut_just_past_the_edge,
    "@use \"sass:color\";\na {\n  color: color.is-in-gamut(hsl(120 100.0000000001% 50%));\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    is_in_gamut_unbounded_space_is_always_true,
    "@use \"sass:color\";\na {\n  color: color.is-in-gamut(hsl(120 50% 150%), oklch);\n}\n",
    "a {\n  color: true;\n}\n"
);
error!(
    is_in_gamut_quoted_space,
    "@use \"sass:color\";\na {\n  color: color.is-in-gamut(hsl(120 50% 150%), \"rgb\");\n}\n",
    "Error: $space: Expected \"rgb\" to be an unquoted string."
);

// ---------------------------------------------------------------------------
// color.to-gamut()
// ---------------------------------------------------------------------------

test!(
    to_gamut_clip_clamps_saturation,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(hsl(120 200% 50%), $method: clip);\n}\n",
    "a {\n  color: hsl(120, 100%, 50%);\n}\n"
);
test!(
    to_gamut_clip_in_rgb_space,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(hsl(120 200% 50%), $space: rgb, $method: clip);\n}\n",
    "a {\n  color: hsl(120, 100%, 50%);\n}\n"
);
test!(
    to_gamut_clip_lightness_above_100,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(hsl(0 100% 120%), $method: clip);\n}\n",
    "a {\n  color: hsl(0, 100%, 100%);\n}\n"
);
test!(
    to_gamut_in_gamut_color_is_unchanged,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(#cc0f35, $method: local-minde);\n}\n",
    "a {\n  color: #cc0f35;\n}\n"
);
test!(
    to_gamut_local_minde_white_keeps_missing_hue,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(hsl(120 200% 50%), $method: local-minde);\n}\n",
    "a {\n  color: hsl(none 0% 100%);\n}\n"
);
test!(
    to_gamut_local_minde_white_in_rgb_space,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(hsl(120 200% 50%), $space: rgb, $method: local-minde);\n}\n",
    "a {\n  color: hsl(0, 0%, 100%);\n}\n"
);
test!(
    to_gamut_local_minde_black,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(hsl(0 100% -20%), $method: local-minde);\n}\n",
    "a {\n  color: hsl(none 0% 0%);\n}\n"
);
test!(
    to_gamut_local_minde_searches_chroma,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(hsl(300 150% 60%), $method: local-minde);\n}\n",
    "a {\n  color: hsl(301.710353365, 100%, 77.8771352257%);\n}\n"
);
test!(
    to_gamut_local_minde_in_rgb_space,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(hsl(300 150% 60%), $space: rgb, $method: local-minde);\n}\n",
    "a {\n  color: hsl(300, 100%, 75.6395021777%);\n}\n"
);
test!(
    to_gamut_local_minde_dark_blue,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(hsl(200 300% 30%), $space: rgb, $method: local-minde);\n}\n",
    "a {\n  color: hsl(196.5480993755, 100%, 41.5970106043%);\n}\n"
);
test!(
    to_gamut_local_minde_hwb_input,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(hwb(120 -10% 60%), $method: local-minde);\n}\n",
    "a {\n  color: hsl(119.7951697464, 100%, 19.9608675659%);\n}\n"
);
test!(
    to_gamut_local_minde_out_of_gamut_rgb,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(color.change(#cc0f35, $red: 300), $method: local-minde);\n}\n",
    "a {\n  color: rgb(100%, 38.5892568657%, 36.69309463%);\n}\n"
);
test!(
    to_gamut_local_minde_with_alpha,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(hsla(45 110% 55% / 0.3), $method: local-minde);\n}\n",
    "a {\n  color: hsla(43.8101054773, 100%, 59.6511888315%, 0.3);\n}\n"
);
test!(
    to_gamut_clip_hwb,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(hwb(120 -10% 60%), $method: clip);\n}\n",
    "a {\n  color: #006600;\n}\n"
);
test!(
    to_gamut_unbounded_space_returns_color,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(hsl(120 200% 50%), $space: lab, $method: clip);\n}\n",
    "a {\n  color: hsl(120, 200%, 50%);\n}\n"
);
error!(
    to_gamut_requires_method,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(hsl(120 200% 50%));\n}\n",
    "Error: $method: color.to-gamut() requires a $method argument for forwards-compatibility with changes in the CSS spec. Suggestion:"
);
error!(
    to_gamut_unknown_method,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(#cc0f35, $method: bogus);\n}\n",
    "Error: Unknown gamut map method \"bogus\"."
);
error!(
    to_gamut_method_is_case_sensitive,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(#cc0f35, $method: CLIP);\n}\n",
    "Error: Unknown gamut map method \"CLIP\"."
);
error!(
    to_gamut_quoted_method,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(#cc0f35, $method: \"clip\");\n}\n",
    "Error: $method: Expected \"clip\" to be an unquoted string."
);
error!(
    to_gamut_method_must_be_string,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(#cc0f35, $method: 1);\n}\n",
    "Error: $method: 1 is not a string."
);
error!(
    to_gamut_unknown_space_beats_unknown_method,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(#cc0f35, $method: bogus, $space: nope);\n}\n",
    "Error: $space: Unknown color space \"nope\"."
);

// ---------------------------------------------------------------------------
// color.same()
// ---------------------------------------------------------------------------

test!(
    same_hex_and_rgb,
    "@use \"sass:color\";\na {\n  color: color.same(#cc0f35, rgb(204 15 53));\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    same_different_colors,
    "@use \"sass:color\";\na {\n  color: color.same(#cc0f35, hsl(120 50% 50%));\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    same_across_spaces_compares_rgb,
    "@use \"sass:color\";\na {\n  color: color.same(#000, hsl(120 0% 0%));\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    same_alpha_differs,
    "@use \"sass:color\";\na {\n  color: color.same(rgba(0, 0, 0, 0.5), #000);\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    same_is_not_rounded,
    "@use \"sass:color\";\na {\n  color: color.same(hsl(120 50% 50%), #40bf40);\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    same_exact_float_channels,
    "@use \"sass:color\";\na {\n  color: color.same(hsl(120 50% 50%), rgb(63.75 191.25 63.75));\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    same_after_to_space,
    "@use \"sass:color\";\na {\n  color: color.same(#cc0f35, color.to-space(#cc0f35, hsl));\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    same_hue_wraps,
    "@use \"sass:color\";\na {\n  color: color.same(hsl(120 50% 50%), hsl(480 50% 50%));\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    same_space_compares_channels_not_rgb,
    "@use \"sass:color\";\na {\n  color: color.same(hsl(120 0% 50%), hsl(0 0% 50%));\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    same_space_hwb_compares_stored_hue,
    "@use \"sass:color\";\na {\n  color: color.same(color.hwb(120 60% 60%), color.hwb(0 60% 60%));\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    same_out_of_gamut_hsl,
    "@use \"sass:color\";\na {\n  color: color.same(hsl(120 50% 150%), hsl(120 50% 150%));\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    same_out_of_gamut_hsl_is_not_white,
    "@use \"sass:color\";\na {\n  color: color.same(hsl(120 50% 150%), rgb(255 255 255));\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    same_is_fuzzy,
    "@use \"sass:color\";\na {\n  color: color.same(#cc0f35, rgb(204.000000000001 15 53));\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    same_missing_hue_reads_as_zero,
    "@use \"sass:color\";\na {\n  color: color.same(color.invert(hsl(0 0% 50%)), hsl(0 0% 50%));\n}\n",
    "a {\n  color: true;\n}\n"
);
error!(
    same_non_color,
    "@use \"sass:color\";\na {\n  color: color.same(#cc0f35, 1);\n}\n",
    "Error: $color2: 1 is not a color."
);

// ---------------------------------------------------------------------------
// Serialization by space
// ---------------------------------------------------------------------------

test!(
    hsl_space_keeps_hsl_form_when_whole_rgb,
    "@use \"sass:color\";\na {\n  color: color.adjust(hsl(120 100% 50%), $lightness: -50%);\n}\n",
    "a {\n  color: hsl(120, 100%, 0%);\n}\n"
);
test!(
    hsl_space_scale_keeps_hsl_form,
    "@use \"sass:color\";\na {\n  color: color.scale(hsl(0 100% 50%), $lightness: 0%);\n}\n",
    "a {\n  color: hsl(0, 100%, 50%);\n}\n"
);
test!(
    hsl_space_with_alpha_keeps_hsla_form,
    "@use \"sass:color\";\na {\n  color: color.change(hsl(120 100% 50%), $alpha: 0.5);\n}\n",
    "a {\n  color: hsla(120, 100%, 50%, 0.5);\n}\n"
);
test!(
    hwb_space_with_whole_rgb_is_named,
    "@use \"sass:color\";\na {\n  color: color.hwb(120 0% 0%);\n}\n",
    "a {\n  color: lime;\n}\n"
);
test!(
    hwb_space_with_fractional_rgb_is_hsl,
    "@use \"sass:color\";\na {\n  color: color.hwb(120 25% 25%);\n}\n",
    "a {\n  color: hsl(120, 50%, 50%);\n}\n"
);
test!(
    hwb_hue_reads_as_zero_when_achromatic,
    "@use \"sass:color\";\na {\n  color: color.adjust(color.hwb(120 20% 30%), $whiteness: 90%);\n}\n",
    "a {\n  color: hsl(0, 0%, 78.5714285714%);\n}\n"
);
test!(
    legacy_mix_of_hsl_colors_is_rgb,
    "@use \"sass:color\";\na {\n  color: color.mix(hsl(120 50% 50%), hsl(300 50% 50%));\n}\n",
    "a {\n  color: rgb(50%, 50%, 50%);\n}\n"
);
test!(
    rgba_of_hsl_color_is_rgb,
    "@use \"sass:color\";\na {\n  color: rgba(hsl(120 100% 50%), 0.5);\n}\n",
    "a {\n  color: rgba(0, 255, 0, 0.5);\n}\n"
);
test!(
    hsl_round_trip_float_noise_prints_percentages,
    "@use \"sass:color\";\na {\n  color: color.invert(rgba(200, 10, 50, 0.4), $space: hsl);\n}\n",
    "a {\n  color: rgba(21.568627451%, 96.0784313725%, 80.3921568627%, 0.4);\n}\n"
);
test!(
    global_hwb_is_a_color,
    "a {\n  color: hwb(120 20% 30%);\n}\n",
    "a {\n  color: hsl(120, 55.5555555556%, 45%);\n}\n"
);
test!(
    global_hwb_with_alpha,
    "a {\n  color: hwb(120 20% 30% / 0.5);\n}\n",
    "a {\n  color: hsla(120, 55.5555555556%, 45%, 0.5);\n}\n"
);
error!(
    global_hwb_takes_one_argument,
    "a {\n  color: hwb(120, 20%, 30%);\n}\n", "Error: Only 1 argument allowed, but 3 were passed."
);
test!(
    hwb_negative_whiteness_is_out_of_gamut,
    "a {\n  color: hwb(120 -20% 30%);\n}\n",
    "a {\n  color: hsl(120, 180%, 25%);\n}\n"
);
test!(
    hwb_whiteness_and_blackness_over_100_are_scaled,
    "@use \"sass:color\";\na {\n  color: color.channel(color.hwb(120 70% 70%), \"whiteness\");\n}\n",
    "a {\n  color: 50%;\n}\n"
);
test!(
    change_unclamped_red_is_out_of_gamut,
    "@use \"sass:color\";\na {\n  color: color.change(#cc0f35, $red: 300);\n}\n",
    "a {\n  color: hsl(352, 146.1538461538%, 61.7647058824%);\n}\n"
);
test!(
    change_negative_red_rotates_hue,
    "@use \"sass:color\";\na {\n  color: color.change(#cc0f35, $red: -5);\n}\n",
    "a {\n  color: hsl(219.3103448276, 120.8333333333%, 9.4117647059%);\n}\n"
);
test!(
    change_negative_saturation_rotates_hue,
    "@use \"sass:color\";\na {\n  color: color.change(hsl(120 50% 50%), $saturation: -50%);\n}\n",
    "a {\n  color: hsl(300, 50%, 50%);\n}\n"
);
test!(
    compressed_picks_shorter_of_rgb_and_hsl,
    "@use \"sass:color\";\na {\n  color: color.invert(rgba(0, 0, 0, 0.5), 30%, $space: rgb);\n}\n",
    "a{color:hsla(0,0%,30%,.5)}",
    grass::Options::default().style(grass::OutputStyle::Compressed)
);
test!(
    compressed_out_of_gamut_is_hsl,
    "a {\n  color: hsl(120 50% 150%);\n}\n",
    "a{color:hsl(120,50%,150%)}",
    grass::Options::default().style(grass::OutputStyle::Compressed)
);

// ---------------------------------------------------------------------------
// Missing hue
// ---------------------------------------------------------------------------

test!(
    invert_of_gray_hsl_has_missing_hue,
    "@use \"sass:color\";\na {\n  color: color.invert(hsl(0 0% 50%));\n}\n",
    "a {\n  color: hsl(none 0% 50%);\n}\n"
);
test!(
    missing_hue_with_alpha,
    "@use \"sass:color\";\na {\n  color: color.invert(hsla(0 0% 50% / 0.4));\n}\n",
    "a {\n  color: hsl(none 0% 50% / 0.4);\n}\n"
);
test!(
    missing_hue_in_hwb,
    "@use \"sass:color\";\na {\n  color: color.invert(color.hwb(0 50% 50%));\n}\n",
    "a {\n  color: hwb(none 50% 50%);\n}\n"
);
test!(
    missing_hue_compressed,
    "@use \"sass:color\";\na {\n  color: color.invert(hsla(0 0% 50% / 0.4));\n}\n",
    "a{color:hsl(none 0% 50%/.4)}",
    grass::Options::default().style(grass::OutputStyle::Compressed)
);
test!(
    missing_hue_survives_lightness_adjust,
    "@use \"sass:color\";\na {\n  color: color.adjust(color.invert(hsl(0 0% 50%)), $lightness: 10%);\n}\n",
    "a {\n  color: hsl(none 0% 60%);\n}\n"
);
test!(
    missing_hue_survives_alpha_adjust,
    "@use \"sass:color\";\na {\n  color: color.adjust(color.invert(hsl(0 0% 50%)), $alpha: -0.1);\n}\n",
    "a {\n  color: hsl(none 0% 50% / 0.9);\n}\n"
);
test!(
    missing_hue_survives_scale,
    "@use \"sass:color\";\na {\n  color: color.scale(color.invert(hsl(0 0% 50%)), $lightness: 10%);\n}\n",
    "a {\n  color: hsl(none 0% 55%);\n}\n"
);
test!(
    missing_hue_survives_grayscale,
    "@use \"sass:color\";\na {\n  color: color.grayscale(color.invert(hsl(0 0% 50%)));\n}\n",
    "a {\n  color: hsl(none 0% 50%);\n}\n"
);
test!(
    missing_hue_survives_clip,
    "@use \"sass:color\";\na {\n  color: color.to-gamut(color.invert(hsl(0 0% 50%)), $method: clip);\n}\n",
    "a {\n  color: hsl(none 0% 50%);\n}\n"
);
test!(
    change_replaces_missing_hue,
    "@use \"sass:color\";\na {\n  color: color.change(color.invert(hsl(0 0% 50%)), $hue: 10deg);\n}\n",
    "a {\n  color: hsl(10, 0%, 50%);\n}\n"
);
test!(
    missing_hue_channel_reads_as_zero,
    "@use \"sass:color\";\na {\n  color: color.channel(color.invert(hsl(0 0% 50%)), \"hue\");\n}\n",
    "a {\n  color: 0deg;\n}\n"
);
test!(
    missing_hue_converts_to_rgb,
    "@use \"sass:color\";\na {\n  color: color.to-space(color.invert(hsl(0 0% 50%)), rgb);\n}\n",
    "a {\n  color: rgb(50%, 50%, 50%);\n}\n"
);
test!(
    missing_hue_is_dropped_by_conversion,
    "@use \"sass:color\";\na {\n  color: color.adjust(color.invert(hsl(0 0% 50%)), $red: 10);\n}\n",
    "a {\n  color: hsl(0, 4.0816326531%, 51.9607843137%);\n}\n"
);
test!(
    missing_hue_is_dropped_by_legacy_alpha_functions,
    "@use \"sass:color\";\na {\n  color: transparentize(color.invert(hsl(0 0% 50%)), 0.1);\n}\n",
    "a {\n  color: hsla(0, 0%, 50%, 0.9);\n}\n"
);
test!(
    missing_hue_space,
    "@use \"sass:color\";\na {\n  color: color.space(color.invert(hsl(0 0% 50%)));\n}\n",
    "a {\n  color: hsl;\n}\n"
);
error!(
    adjust_missing_hue,
    "@use \"sass:color\";\na {\n  color: color.adjust(color.invert(hsl(0 0% 50%)), $hue: 10deg);\n}\n",
    "Error: $hue: Because the CSS working group is still deciding on the best behavior, Sass doesn't currently support modifying missing channels (color: hsl(none 0% 50%))."
);
error!(
    adjust_hue_of_black_in_explicit_hsl_space,
    "@use \"sass:color\";\na {\n  color: color.adjust(#000, $hue: 10deg, $space: hsl);\n}\n",
    "Error: $hue: Because the CSS working group is still deciding on the best behavior, Sass doesn't currently support modifying missing channels (color: hsl(none 0% 0%))."
);
error!(
    adjust_hue_of_gray_in_explicit_hwb_space,
    "@use \"sass:color\";\na {\n  color: color.adjust(#808080, $hue: 10deg, $space: hwb);\n}\n",
    "Error: $hue: Because the CSS working group is still deciding on the best behavior, Sass doesn't currently support modifying missing channels (color: hwb(none 50.1960784314% 49.8039215686%))."
);
test!(
    adjust_hue_of_black_without_space_reads_hue_as_zero,
    "@use \"sass:color\";\na {\n  color: color.adjust(#000, $hue: 10deg);\n}\n",
    "a {\n  color: black;\n}\n"
);
test!(
    change_hue_of_black_in_explicit_hsl_space,
    "@use \"sass:color\";\na {\n  color: color.change(#000, $hue: 10deg, $space: hsl);\n}\n",
    "a {\n  color: black;\n}\n"
);
test!(
    channel_hue_of_black_in_hsl_space,
    "@use \"sass:color\";\na {\n  color: color.channel(#000, \"hue\", $space: hsl);\n}\n",
    "a {\n  color: 0deg;\n}\n"
);
