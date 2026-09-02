use crate::builtin::{
    color::{
        hsl::{complement, hue, lightness, module_grayscale, module_invert, saturation},
        hwb::{blackness, hwb, whiteness},
        opacity::{module_alpha, module_opacity},
        other::{
            adjust_color, change_color, channel, ie_hex_str, is_in_gamut, is_legacy, is_missing,
            is_powerless, same, scale_color, space, to_gamut, to_space,
        },
        rgb::{blue, green, mix, red},
    },
    modules::Module,
};

pub(crate) fn declare(f: &mut Module) {
    f.insert_builtin("adjust", adjust_color);
    f.insert_builtin("channel", channel);
    f.insert_builtin("alpha", module_alpha);
    f.insert_builtin("blue", blue);
    f.insert_builtin("change", change_color);
    f.insert_builtin("complement", complement);
    f.insert_builtin("grayscale", module_grayscale);
    f.insert_builtin("green", green);
    f.insert_builtin("hue", hue);
    f.insert_builtin("ie-hex-str", ie_hex_str);
    f.insert_builtin("invert", module_invert);
    f.insert_builtin("is-in-gamut", is_in_gamut);
    f.insert_builtin("is-legacy", is_legacy);
    f.insert_builtin("is-missing", is_missing);
    f.insert_builtin("is-powerless", is_powerless);
    f.insert_builtin("lightness", lightness);
    f.insert_builtin("mix", mix);
    f.insert_builtin("opacity", module_opacity);
    f.insert_builtin("red", red);
    f.insert_builtin("same", same);
    f.insert_builtin("saturation", saturation);
    f.insert_builtin("scale", scale_color);
    f.insert_builtin("space", space);
    f.insert_builtin("to-gamut", to_gamut);
    f.insert_builtin("to-space", to_space);
    f.insert_builtin("blackness", blackness);
    f.insert_builtin("whiteness", whiteness);
    f.insert_builtin("hwb", hwb);
}
