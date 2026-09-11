use std::{fmt, sync::Arc};

use crate::interner::InternedString;

pub(crate) use conversion::{UNIT_CONVERSION_TABLE, known_compatibilities_by_unit};

mod conversion;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Unit {
    // Absolute units
    /// Pixels
    Px,
    /// Millimeters
    Mm,
    /// Inches
    In,
    /// Centimeters
    Cm,
    /// Quarter-millimeters
    Q,
    /// Points
    Pt,
    /// Picas
    Pc,

    // Font relative units
    /// Font size of the parent element
    Em,
    /// Font size of the root element
    Rem,
    /// Line height of the element
    Lh,
    /// x-height of the element's font
    Ex,
    /// The advance measure (width) of the glyph "0" of the element's font
    Ch,
    /// Represents the "cap height" (nominal height of capital letters) of the element's font
    Cap,
    /// Equal to the used advance measure of the "水" (CJK water ideograph, U+6C34) glyph
    /// found in the font used to render it
    Ic,
    /// Equal to the computed value of the line-height property on the root element
    /// (typically \<html\>), converted to an absolute length
    Rlh,

    // Viewport relative units
    /// 1% of the viewport's width
    Vw,
    /// 1% of the viewport's height
    Vh,
    /// 1% of the viewport's smaller dimension
    Vmin,
    /// 1% of the viewport's larger dimension
    Vmax,
    /// Equal to 1% of the size of the initial containing block, in the direction of the root
    /// element's inline axis
    Vi,
    /// Equal to 1% of the size of the initial containing block, in the direction of the root
    /// element's block axis
    Vb,

    // Angle units
    /// Represents an angle in degrees. One full circle is 360deg
    Deg,
    /// Represents an angle in gradians. One full circle is 400grad
    Grad,
    /// Represents an angle in radians. One full circle is 2π radians which approximates to 6.283rad
    Rad,
    /// Represents an angle in a number of turns. One full circle is 1turn
    Turn,

    // Time units
    /// Represents a time in seconds
    S,
    /// Represents a time in milliseconds
    Ms,

    // Frequency units
    /// Represents a frequency in hertz
    Hz,
    /// Represents a frequency in kilohertz
    Khz,

    // Resolution units
    /// Represents the number of dots per inch
    Dpi,
    /// Represents the number of dots per centimeter
    Dpcm,
    /// Represents the number of dots per px unit
    Dppx,

    // Other units
    /// Represents a fraction of the available space in the grid container
    Fr,
    Percent,

    /// Unknown unit
    Unknown(InternedString),
    /// Unspecified unit
    None,

    Complex(Arc<ComplexUnit>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComplexUnit {
    pub numer: Vec<Unit>,
    pub denom: Vec<Unit>,
}

/// The factor that turns a value in the single unit `from` into `to`, or
/// `None` if they do not convert. dart-sass's `conversionFactor(to, from)`.
fn simple_factor(from: &Unit, to: &Unit) -> Option<f64> {
    if from == to {
        return Some(1.0);
    }

    UNIT_CONVERSION_TABLE.get(to)?.get(from).copied()
}

pub(crate) fn are_any_convertible(units1: &[Unit], units2: &[Unit]) -> bool {
    for unit1 in units1 {
        for unit2 in units2 {
            if unit1.comparable(unit2) {
                return true;
            }
        }
    }

    false
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum UnitKind {
    Absolute,
    FontRelative,
    ViewportRelative,
    Angle,
    Time,
    Frequency,
    Resolution,
    Other,
    None,
}

impl Unit {
    pub(crate) fn new(mut numer: Vec<Self>, denom: Vec<Self>) -> Self {
        if denom.is_empty() && numer.is_empty() {
            Unit::None
        } else if denom.is_empty() && numer.len() == 1 {
            numer.pop().unwrap()
        } else {
            Unit::Complex(Arc::new(ComplexUnit { numer, denom }))
        }
    }

    pub(crate) fn numer_and_denom(self) -> (Vec<Unit>, Vec<Unit>) {
        match self {
            Self::Complex(complex) => (complex.numer.clone(), complex.denom.clone()),
            Self::None => (Vec::new(), Vec::new()),
            v => (vec![v], Vec::new()),
        }
    }

    pub(crate) fn invert(self) -> Self {
        let (numer, denom) = self.numer_and_denom();

        Self::new(denom, numer)
    }

    pub(crate) fn is_complex(&self) -> bool {
        matches!(self, Unit::Complex(complex) if complex.numer.len() != 1 || !complex.denom.is_empty())
    }

    pub(crate) fn comparable(&self, other: &Unit) -> bool {
        // A unitless number combines with any unit, complex or not, so this
        // comes before the complex check: `1 + 1px/1em` is `2px/em`.
        if other == &Unit::None || self == &Unit::None {
            return true;
        }
        if matches!(self, Unit::Complex(..)) || matches!(other, Unit::Complex(..)) {
            return self.factor_to(other).is_some();
        }
        match self.kind() {
            UnitKind::FontRelative | UnitKind::ViewportRelative | UnitKind::Other => self == other,
            UnitKind::None => true,
            u => other.kind() == u,
        }
    }

    /// The factor that turns a value in `self` into the same quantity in
    /// `to`, or `None` if the units are incompatible.
    ///
    /// A port of dart-sass 1.103.1's `SassNumber._coerceOrConvertValue`,
    /// without its unitless shortcut, which callers handle. Each unit in
    /// `to`'s numerator is paired with the first convertible unit left in
    /// `self`'s numerator, and likewise for the denominators. Any unit left
    /// unpaired on either side makes them incompatible. So `in/fu` converts
    /// to `cm/fu`, where comparing the complex units whole rejected it
    /// (libsass `units/simple`).
    pub(crate) fn factor_to(&self, to: &Unit) -> Option<f64> {
        if self == to {
            return Some(1.0);
        }

        let (mut old_numer, mut old_denom) = self.clone().numer_and_denom();
        let (new_numer, new_denom) = to.clone().numer_and_denom();

        let mut factor = 1.0;
        for new in &new_numer {
            let i = old_numer
                .iter()
                .position(|old| simple_factor(old, new).is_some())?;
            factor *= simple_factor(&old_numer.remove(i), new)?;
        }
        for new in &new_denom {
            let i = old_denom
                .iter()
                .position(|old| simple_factor(old, new).is_some())?;
            factor /= simple_factor(&old_denom.remove(i), new)?;
        }

        (old_numer.is_empty() && old_denom.is_empty()).then_some(factor)
    }

    /// The one spelling that names this unit.
    ///
    /// `None` for the three that have no fixed spelling: an unknown unit
    /// carries its own, a complex one is built from its parts, and
    /// [`Unit::None`] has none at all.
    ///
    /// This is what makes the case rule cheap to state and to apply. A name is
    /// the known unit only when it matches this exactly, which is the test
    /// [`Unit::from`] makes, and it is what [`fmt::Display`] writes.
    pub(crate) fn canonical_name(&self) -> Option<&'static str> {
        Some(match self {
            Unit::Px => "px",
            Unit::Mm => "mm",
            Unit::In => "in",
            Unit::Cm => "cm",
            Unit::Q => "q",
            Unit::Pt => "pt",
            Unit::Pc => "pc",
            Unit::Em => "em",
            Unit::Rem => "rem",
            Unit::Lh => "lh",
            Unit::Percent => "%",
            Unit::Ex => "ex",
            Unit::Ch => "ch",
            Unit::Cap => "cap",
            Unit::Ic => "ic",
            Unit::Rlh => "rlh",
            Unit::Vw => "vw",
            Unit::Vh => "vh",
            Unit::Vmin => "vmin",
            Unit::Vmax => "vmax",
            Unit::Vi => "vi",
            Unit::Vb => "vb",
            Unit::Deg => "deg",
            Unit::Grad => "grad",
            Unit::Rad => "rad",
            Unit::Turn => "turn",
            Unit::S => "s",
            Unit::Ms => "ms",
            Unit::Hz => "Hz",
            Unit::Khz => "kHz",
            Unit::Dpi => "dpi",
            Unit::Dpcm => "dpcm",
            Unit::Dppx => "dppx",
            Unit::Fr => "fr",
            Unit::Unknown(..) | Unit::None | Unit::Complex(..) => return None,
        })
    }

    /// The known unit this one names when case is ignored, or a clone of it.
    ///
    /// dart-sass splits the two readings of a unit name. Conversion and
    /// printing are case-sensitive, so `1Q` is an unknown unit that never
    /// becomes millimetres -- but the known-compatibility check that decides
    /// whether `calc(1Q + 1deg)` is an error lowercases first, so `Q` counts
    /// as a length *there* and the compile fails the way dart-sass's does.
    /// Verified against dart-sass 1.103.1: `calc(1Q + 1mm)` compiles,
    /// `calc(1Q + 1deg)` and `calc(1HZ + 1deg)` do not, and
    /// `calc(1foo + 1deg)` does, an unknown name being compatible with
    /// anything.
    ///
    /// Use it only for that check. Anywhere else it would erase the
    /// distinction the rest of this type exists to keep.
    pub(crate) fn ignoring_case(&self) -> Unit {
        match self {
            Unit::Unknown(name) => {
                known_unit_ignoring_case(name.resolve_ref()).unwrap_or_else(|| self.clone())
            }
            _ => self.clone(),
        }
    }

    /// Used internally to determine if two units are comparable or not
    fn kind(&self) -> UnitKind {
        match self {
            Unit::Px | Unit::Mm | Unit::In | Unit::Cm | Unit::Q | Unit::Pt | Unit::Pc => {
                UnitKind::Absolute
            }
            Unit::Em
            | Unit::Rem
            | Unit::Lh
            | Unit::Ex
            | Unit::Ch
            | Unit::Cap
            | Unit::Ic
            | Unit::Rlh => UnitKind::FontRelative,
            Unit::Vw | Unit::Vh | Unit::Vmin | Unit::Vmax | Unit::Vi | Unit::Vb => {
                UnitKind::ViewportRelative
            }
            Unit::Deg | Unit::Grad | Unit::Rad | Unit::Turn => UnitKind::Angle,
            Unit::S | Unit::Ms => UnitKind::Time,
            Unit::Hz | Unit::Khz => UnitKind::Frequency,
            Unit::Dpi | Unit::Dpcm | Unit::Dppx => UnitKind::Resolution,
            Unit::None => UnitKind::None,
            Unit::Fr | Unit::Percent | Unit::Unknown(..) | Unit::Complex { .. } => UnitKind::Other,
        }
    }
}

/// The known unit a name refers to when case is ignored, if any.
///
/// This is the only table of unit spellings; [`Unit::from`] narrows it to the
/// canonical casing, and [`Unit::ignoring_case`] uses it as it stands.
fn known_unit_ignoring_case(name: &str) -> Option<Unit> {
    Some(match name.to_ascii_lowercase().as_str() {
        "px" => Unit::Px,
        "mm" => Unit::Mm,
        "in" => Unit::In,
        "cm" => Unit::Cm,
        "q" => Unit::Q,
        "pt" => Unit::Pt,
        "pc" => Unit::Pc,
        "em" => Unit::Em,
        "rem" => Unit::Rem,
        "lh" => Unit::Lh,
        "%" => Unit::Percent,
        "ex" => Unit::Ex,
        "ch" => Unit::Ch,
        "cap" => Unit::Cap,
        "ic" => Unit::Ic,
        "rlh" => Unit::Rlh,
        "vw" => Unit::Vw,
        "vh" => Unit::Vh,
        "vmin" => Unit::Vmin,
        "vmax" => Unit::Vmax,
        "vi" => Unit::Vi,
        "vb" => Unit::Vb,
        "deg" => Unit::Deg,
        "grad" => Unit::Grad,
        "rad" => Unit::Rad,
        "turn" => Unit::Turn,
        "s" => Unit::S,
        "ms" => Unit::Ms,
        "hz" => Unit::Hz,
        "khz" => Unit::Khz,
        "dpi" => Unit::Dpi,
        "dpcm" => Unit::Dpcm,
        "dppx" => Unit::Dppx,
        "fr" => Unit::Fr,
        _ => return None,
    })
}

impl From<String> for Unit {
    /// Recognise a unit by its exact spelling.
    ///
    /// **The match is case-sensitive, deliberately.** CSS treats unit names
    /// case-insensitively, but dart-sass does not: only the canonical
    /// spelling is a known unit, and every other casing is an unknown one
    /// that never converts and prints back as written. `1Q` is not the
    /// quarter-millimetre unit, `math.div(1kHz, 1hz)` does not simplify, and
    /// `1PX + 1px` is an error about incompatible units. Lowercasing here
    /// made all three disagree with the reference implementation, and printed
    /// `1Q` as `1q`.
    ///
    /// The one place case is ignored is [`Unit::ignoring_case`], which says
    /// why.
    fn from(unit: String) -> Self {
        match known_unit_ignoring_case(&unit) {
            Some(known) if known.canonical_name() == Some(unit.as_str()) => known,
            _ => Unit::Unknown(InternedString::get_or_intern(unit)),
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Unit::Unknown(s) => write!(f, "{}", s),
            Unit::None => Ok(()),
            Unit::Complex(complex) => {
                let numer = &complex.numer;
                let denom = &complex.denom;
                debug_assert!(
                    numer.len() > 1 || !denom.is_empty(),
                    "unsimplified complex unit"
                );

                let numer_rendered = numer
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join("*");

                let denom_rendered = denom
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join("*");

                if denom.is_empty() {
                    write!(f, "{}", numer_rendered)
                } else if numer.is_empty() && denom.len() == 1 {
                    write!(f, "{}^-1", denom_rendered)
                } else if numer.is_empty() {
                    write!(f, "({})^-1", denom_rendered)
                } else if denom.len() == 1 {
                    write!(f, "{}/{}", numer_rendered, denom_rendered)
                } else {
                    write!(f, "{}/({})", numer_rendered, denom_rendered)
                }
            }
            // Every other unit has one spelling, and `canonical_name` is where
            // it lives, so that the name this writes and the name
            // `Unit::from` accepts cannot drift apart.
            simple => f.write_str(
                simple
                    .canonical_name()
                    .expect("a unit with no canonical name is handled above"),
            ),
        }
    }
}
