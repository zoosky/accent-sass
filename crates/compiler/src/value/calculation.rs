use core::fmt;
use std::iter::Iterator;

use codemap::Span;

use crate::{
    Options,
    common::BinaryOp,
    error::SassResult,
    serializer::inspect_number,
    unit::Unit,
    value::{
        Number, SassNumber, Value,
        number::{fuzzy_ceil, fuzzy_floor, fuzzy_round},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalculationArg {
    Number(SassNumber),
    Calculation(SassCalculation),
    String(String),
    Operation {
        lhs: Box<Self>,
        op: BinaryOp,
        rhs: Box<Self>,
    },
    Interpolation(String),
    /// Values written next to each other with only whitespace between them, as
    /// in `calc(var(--c) 1)`. They are emitted space-separated and never
    /// simplified, since only the browser can tell what they mean.
    Space(Vec<Self>),
}

impl CalculationArg {
    pub fn parenthesize_calculation_rhs(outer: BinaryOp, right: BinaryOp) -> bool {
        if outer == BinaryOp::Div {
            true
        } else if outer == BinaryOp::Plus {
            false
        } else {
            right == BinaryOp::Plus || right == BinaryOp::Minus
        }
    }
}

/// The name of a CSS math function that Sass understands as a calculation.
///
/// Every variant is simplified at compile time when its arguments allow it and
/// otherwise serialized back out verbatim for the browser to evaluate. The set
/// mirrors the CSS Values 4 math function list that Dart Sass supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CalculationName {
    Calc,
    Min,
    Max,
    Clamp,
    Round,
    Mod,
    Rem,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Atan2,
    Pow,
    Sqrt,
    Hypot,
    Log,
    Exp,
    Abs,
    Sign,
    CalcSize,
}

impl fmt::Display for CalculationName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl CalculationName {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            CalculationName::Calc => "calc",
            CalculationName::Min => "min",
            CalculationName::Max => "max",
            CalculationName::Clamp => "clamp",
            CalculationName::Round => "round",
            CalculationName::Mod => "mod",
            CalculationName::Rem => "rem",
            CalculationName::Sin => "sin",
            CalculationName::Cos => "cos",
            CalculationName::Tan => "tan",
            CalculationName::Asin => "asin",
            CalculationName::Acos => "acos",
            CalculationName::Atan => "atan",
            CalculationName::Atan2 => "atan2",
            CalculationName::Pow => "pow",
            CalculationName::Sqrt => "sqrt",
            CalculationName::Hypot => "hypot",
            CalculationName::Log => "log",
            CalculationName::Exp => "exp",
            CalculationName::Abs => "abs",
            CalculationName::Sign => "sign",
            CalculationName::CalcSize => "calc-size",
        }
    }

    pub(crate) fn in_min_or_max(self) -> bool {
        self == CalculationName::Min || self == CalculationName::Max
    }

    /// Resolves a lowercased function name to the calculation it names.
    pub(crate) fn from_lowercase_str(name: &str) -> Option<Self> {
        Some(match name {
            "calc" => CalculationName::Calc,
            "min" => CalculationName::Min,
            "max" => CalculationName::Max,
            "clamp" => CalculationName::Clamp,
            "round" => CalculationName::Round,
            "mod" => CalculationName::Mod,
            "rem" => CalculationName::Rem,
            "sin" => CalculationName::Sin,
            "cos" => CalculationName::Cos,
            "tan" => CalculationName::Tan,
            "asin" => CalculationName::Asin,
            "acos" => CalculationName::Acos,
            "atan" => CalculationName::Atan,
            "atan2" => CalculationName::Atan2,
            "pow" => CalculationName::Pow,
            "sqrt" => CalculationName::Sqrt,
            "hypot" => CalculationName::Hypot,
            "log" => CalculationName::Log,
            "exp" => CalculationName::Exp,
            "abs" => CalculationName::Abs,
            "sign" => CalculationName::Sign,
            "calc-size" => CalculationName::CalcSize,
            _ => return None,
        })
    }

    /// Whether a failed calculation parse should fall back to parsing the name
    /// as an ordinary Sass function call.
    ///
    /// Only the four names that are also global Sass functions do; every other
    /// math function is a calculation or a syntax error.
    pub(crate) fn falls_back_to_function(self) -> bool {
        matches!(
            self,
            CalculationName::Min
                | CalculationName::Max
                | CalculationName::Round
                | CalculationName::Abs
        )
    }

    /// Whether an argument list of `len` could be the Sass function's rather
    /// than the calculation's.
    ///
    /// `math.round` and `math.abs` take a single argument, so a longer list can
    /// only have been meant as a calculation and keeps the calculation's own
    /// error. `min` and `max` are variadic.
    pub(crate) fn function_accepts_arity(self, len: usize) -> bool {
        match self {
            CalculationName::Min | CalculationName::Max => true,
            CalculationName::Round | CalculationName::Abs => len == 1,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SassCalculation {
    pub name: CalculationName,
    pub args: Vec<CalculationArg>,
}

impl SassCalculation {
    pub fn unsimplified(name: CalculationName, args: Vec<CalculationArg>) -> Self {
        Self { name, args }
    }

    pub fn calc(arg: CalculationArg) -> Value {
        let arg = Self::simplify(arg);
        match arg {
            CalculationArg::Number(n) => Value::Dimension(n),
            CalculationArg::Calculation(c) => Value::Calculation(c),
            _ => Value::Calculation(SassCalculation {
                name: CalculationName::Calc,
                args: vec![arg],
            }),
        }
    }

    pub fn min(args: Vec<CalculationArg>, options: &Options, span: Span) -> SassResult<Value> {
        let args = Self::simplify_arguments(args);
        debug_assert!(!args.is_empty(), "min() must have at least one argument.");

        let mut minimum: Option<SassNumber> = None;

        for arg in &args {
            match arg {
                CalculationArg::Number(n)
                    if minimum.is_some() && !minimum.as_ref().unwrap().is_comparable_to(n) =>
                {
                    minimum = None;
                    break;
                }
                CalculationArg::Number(n)
                    if minimum.is_none()
                        || minimum.as_ref().unwrap().num
                            > n.num.convert(&n.unit, &minimum.as_ref().unwrap().unit) =>
                {
                    minimum = Some(n.clone());
                }
                CalculationArg::Number(..) => continue,
                _ => {
                    minimum = None;
                    break;
                }
            }
        }

        Ok(match minimum {
            Some(min) => Value::Dimension(min),
            None => {
                Self::verify_compatible_numbers(&args, options, span)?;

                Value::Calculation(SassCalculation {
                    name: CalculationName::Min,
                    args,
                })
            }
        })
    }

    pub fn max(args: Vec<CalculationArg>, options: &Options, span: Span) -> SassResult<Value> {
        let args = Self::simplify_arguments(args);
        if args.is_empty() {
            return Err(("max() must have at least one argument.", span).into());
        }

        let mut maximum: Option<SassNumber> = None;

        for arg in &args {
            match arg {
                CalculationArg::Number(n)
                    if maximum.is_some() && !maximum.as_ref().unwrap().is_comparable_to(n) =>
                {
                    maximum = None;
                    break;
                }
                CalculationArg::Number(n)
                    if maximum.is_none()
                        || maximum.as_ref().unwrap().num
                            < n.num.convert(&n.unit, &maximum.as_ref().unwrap().unit) =>
                {
                    maximum = Some(n.clone());
                }
                CalculationArg::Number(..) => continue,
                _ => {
                    maximum = None;
                    break;
                }
            }
        }

        Ok(match maximum {
            Some(max) => Value::Dimension(max),
            None => {
                Self::verify_compatible_numbers(&args, options, span)?;

                Value::Calculation(SassCalculation {
                    name: CalculationName::Max,
                    args,
                })
            }
        })
    }

    pub fn clamp(
        min: CalculationArg,
        value: Option<CalculationArg>,
        max: Option<CalculationArg>,
        options: &Options,
        span: Span,
    ) -> SassResult<Value> {
        if value.is_none() && max.is_some() {
            return Err(("If value is null, max must also be null.", span).into());
        }

        let min = Self::simplify(min);
        let value = value.map(Self::simplify);
        let max = max.map(Self::simplify);

        match (min.clone(), value.clone(), max.clone()) {
            (
                CalculationArg::Number(min),
                Some(CalculationArg::Number(value)),
                Some(CalculationArg::Number(max)),
            ) => {
                if min.is_comparable_to(&value) && min.is_comparable_to(&max) {
                    if value.num <= min.num.convert(min.unit(), value.unit()) {
                        return Ok(Value::Dimension(min));
                    }

                    if value.num >= max.num.convert(max.unit(), value.unit()) {
                        return Ok(Value::Dimension(max));
                    }

                    return Ok(Value::Dimension(value));
                }
            }
            _ => {}
        }

        let mut args = vec![min];

        if let Some(value) = value {
            args.push(value);
        }

        if let Some(max) = max {
            args.push(max);
        }

        Self::verify_length(&args, 3, span)?;
        Self::verify_compatible_numbers(&args, options, span)?;

        Ok(Value::Calculation(SassCalculation {
            name: CalculationName::Clamp,
            args,
        }))
    }

    fn verify_length(args: &[CalculationArg], len: usize, span: Span) -> SassResult<()> {
        if args.len() == len {
            return Ok(());
        }

        if args.iter().any(|arg| {
            matches!(
                arg,
                CalculationArg::String(..) | CalculationArg::Interpolation(..)
            )
        }) {
            return Ok(());
        }

        let was_or_were = if args.len() == 1 { "was" } else { "were" };

        Err((
            format!(
                "{len} arguments required, but only {} {was_or_were} passed.",
                args.len(),
                len = len,
                was_or_were = was_or_were,
            ),
            span,
        )
            .into())
    }

    #[allow(clippy::needless_range_loop)]
    fn verify_compatible_numbers(
        args: &[CalculationArg],
        options: &Options,
        span: Span,
    ) -> SassResult<()> {
        for arg in args {
            match arg {
                CalculationArg::Number(num) => match &num.unit {
                    Unit::Complex(complex) => {
                        if complex.numer.len() > 1 || !complex.denom.is_empty() {
                            let num = num.clone();
                            let value = Value::Dimension(num);
                            return Err((
                                format!(
                                    "Number {} isn't compatible with CSS calculations.",
                                    value.inspect(span)?
                                ),
                                span,
                            )
                                .into());
                        }
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        for i in 0..args.len() {
            let number1 = match &args[i] {
                CalculationArg::Number(num) => num,
                _ => continue,
            };

            for j in (i + 1)..args.len() {
                let number2 = match &args[j] {
                    CalculationArg::Number(num) => num,
                    _ => continue,
                };

                if number1.has_possibly_compatible_units(number2) {
                    continue;
                }

                return Err((
                    format!(
                        "{} and {} are incompatible.",
                        inspect_number(number1, options, span)?,
                        inspect_number(number2, options, span)?
                    ),
                    span,
                )
                    .into());
            }
        }

        Ok(())
    }

    pub fn operate_internal(
        mut op: BinaryOp,
        left: CalculationArg,
        right: CalculationArg,
        in_min_or_max: bool,
        simplify: bool,
        options: &Options,
        span: Span,
    ) -> SassResult<CalculationArg> {
        if !simplify {
            return Ok(CalculationArg::Operation {
                lhs: Box::new(left),
                op,
                rhs: Box::new(right),
            });
        }

        let left = Self::simplify(left);
        let mut right = Self::simplify(right);

        if op == BinaryOp::Plus || op == BinaryOp::Minus {
            match (&left, &right) {
                (CalculationArg::Number(left), CalculationArg::Number(right))
                    if if in_min_or_max {
                        left.is_comparable_to(right)
                    } else {
                        left.has_compatible_units(&right.unit)
                    } =>
                {
                    if op == BinaryOp::Plus {
                        return Ok(CalculationArg::Number(left.clone() + right.clone()));
                    } else {
                        return Ok(CalculationArg::Number(left.clone() - right.clone()));
                    }
                }
                _ => {}
            }

            Self::verify_compatible_numbers(&[left.clone(), right.clone()], options, span)?;

            if let CalculationArg::Number(mut n) = right {
                if n.num.is_negative() {
                    n.num.0 *= -1.0;
                    op = if op == BinaryOp::Plus {
                        BinaryOp::Minus
                    } else {
                        BinaryOp::Plus
                    }
                } else {
                    // todo: do we need this branch?
                }
                right = CalculationArg::Number(n);
            }

            return Ok(CalculationArg::Operation {
                lhs: Box::new(left),
                op,
                rhs: Box::new(right),
            });
        }

        match (left, right) {
            (CalculationArg::Number(num1), CalculationArg::Number(num2)) => {
                if op == BinaryOp::Mul {
                    Ok(CalculationArg::Number(num1 * num2))
                } else {
                    Ok(CalculationArg::Number(num1 / num2))
                }
            }
            (left, right) => Ok(CalculationArg::Operation {
                lhs: Box::new(left),
                op,
                rhs: Box::new(right),
            }),
        }

        //   _verifyCompatibleNumbers([left, right]);

        // Ok(CalculationArg::Operation {
        //     lhs: Box::new(left),
        //     op,
        //     rhs: Box::new(right),
        // })
    }

    /// The `nearest`, `up`, `down` and `to-zero` strategies `round()` accepts.
    ///
    /// A strategy only ever reaches this type from literal source text; a
    /// `var()` or other opaque argument leaves the calculation unsimplified
    /// instead of erroring.
    fn parse_round_strategy(text: &str) -> Option<RoundStrategy> {
        Some(match text {
            "nearest" => RoundStrategy::Nearest,
            "up" => RoundStrategy::Up,
            "down" => RoundStrategy::Down,
            "to-zero" => RoundStrategy::ToZero,
            _ => return None,
        })
    }

    /// Builds the result of a single-argument math function, preserving the
    /// argument's unit.
    fn with_unit_of(value: f64, number: &SassNumber) -> Value {
        Value::Dimension(SassNumber {
            num: Number(value),
            unit: number.unit.clone(),
            as_slash: None,
        })
    }

    /// Emits `name(args)` unsimplified after checking that the arguments could
    /// legally appear in a CSS calculation at all.
    fn unsimplified_checked(
        name: CalculationName,
        args: Vec<CalculationArg>,
        options: &Options,
        span: Span,
    ) -> SassResult<Value> {
        Self::verify_compatible_numbers(&args, options, span)?;
        Ok(Value::Calculation(SassCalculation { name, args }))
    }

    /// Rejects a number that carries units where the CSS function requires
    /// none, matching Dart Sass's wording.
    fn assert_unitless(number: &SassNumber, options: &Options, span: Span) -> SassResult<()> {
        if number.unit == Unit::None {
            Ok(())
        } else {
            Err((
                format!(
                    "Expected {} to have no units.",
                    inspect_number(number, options, span)?
                ),
                span,
            )
                .into())
        }
    }

    /// `abs()`: the magnitude of the argument, keeping its unit.
    pub fn abs(args: Vec<CalculationArg>, options: &Options, span: Span) -> SassResult<Value> {
        let args = Self::simplify_arguments(args);
        Self::verify_exact_length(&args, 1, CalculationName::Abs, span)?;

        match &args[0] {
            // `abs()` is also the global Sass function, so it keeps working on
            // complex units that a CSS calculation could not express.
            CalculationArg::Number(number) => Ok(Self::with_unit_of(number.num.0.abs(), number)),
            _ => Self::unsimplified_checked(CalculationName::Abs, args, options, span),
        }
    }

    /// `sign()`: -1, 0 or 1 in the argument's unit. Zero keeps its sign and
    /// NaN propagates, which is what makes `math.div(1, sign(-0))` infinite.
    pub fn sign(args: Vec<CalculationArg>, options: &Options, span: Span) -> SassResult<Value> {
        let args = Self::simplify_arguments(args);
        Self::verify_exact_length(&args, 1, CalculationName::Sign, span)?;

        match &args[0] {
            // A percentage may resolve to either sign at use time, so it is
            // left for the browser.
            CalculationArg::Number(number) if number.unit != Unit::Percent => {
                let value = number.num.0;
                let signed = if value.is_nan() || value == 0.0 {
                    value
                } else {
                    value.signum()
                };
                Ok(Self::with_unit_of(signed, number))
            }
            _ => Self::unsimplified_checked(CalculationName::Sign, args, options, span),
        }
    }

    /// `exp()`, `sqrt()`: single-argument functions over unitless numbers.
    pub fn unitless_unary(
        name: CalculationName,
        args: Vec<CalculationArg>,
        options: &Options,
        span: Span,
    ) -> SassResult<Value> {
        let args = Self::simplify_arguments(args);
        Self::verify_exact_length(&args, 1, name, span)?;

        match &args[0] {
            CalculationArg::Number(number) => {
                Self::assert_unitless(number, options, span)?;
                let value = number.num.0;
                let result = match name {
                    CalculationName::Sqrt => value.sqrt(),
                    CalculationName::Exp => value.exp(),
                    _ => unreachable!("not a unitless unary function: {}", name),
                };
                Ok(Value::Dimension(SassNumber::new_unitless(result)))
            }
            _ => Self::unsimplified_checked(name, args, options, span),
        }
    }

    /// `sin()`, `cos()`, `tan()`: accept an angle or a unitless number and
    /// return a unitless ratio.
    pub fn trig(
        name: CalculationName,
        args: Vec<CalculationArg>,
        options: &Options,
        span: Span,
    ) -> SassResult<Value> {
        let args = Self::simplify_arguments(args);
        Self::verify_exact_length(&args, 1, name, span)?;

        match &args[0] {
            CalculationArg::Number(number) => {
                let radians = match number.unit {
                    Unit::None => number.num.0,
                    Unit::Rad | Unit::Deg | Unit::Grad | Unit::Turn => {
                        number.num.convert(&number.unit, &Unit::Rad).0
                    }
                    _ => return Err((
                        format!(
                            "$number: Expected {} to have an angle unit (deg, grad, rad, turn).",
                            inspect_number(number, options, span)?
                        ),
                        span,
                    )
                        .into()),
                };

                let result = match name {
                    CalculationName::Sin => radians.sin(),
                    CalculationName::Cos => radians.cos(),
                    CalculationName::Tan => radians.tan(),
                    _ => unreachable!("not a trigonometric function: {}", name),
                };

                Ok(Value::Dimension(SassNumber::new_unitless(result)))
            }
            _ => Self::unsimplified_checked(name, args, options, span),
        }
    }

    /// `asin()`, `acos()`, `atan()`: unitless input, angle output in degrees.
    pub fn inverse_trig(
        name: CalculationName,
        args: Vec<CalculationArg>,
        options: &Options,
        span: Span,
    ) -> SassResult<Value> {
        let args = Self::simplify_arguments(args);
        Self::verify_exact_length(&args, 1, name, span)?;

        match &args[0] {
            CalculationArg::Number(number) => {
                Self::assert_unitless(number, options, span)?;
                let value = number.num.0;
                let degrees = match name {
                    CalculationName::Asin => value.asin().to_degrees(),
                    CalculationName::Acos => value.acos().to_degrees(),
                    CalculationName::Atan => value.atan().to_degrees(),
                    _ => unreachable!("not an inverse trigonometric function: {}", name),
                };

                Ok(Value::Dimension(SassNumber {
                    num: Number(degrees),
                    unit: Unit::Deg,
                    as_slash: None,
                }))
            }
            _ => Self::unsimplified_checked(name, args, options, span),
        }
    }

    /// `atan2($y, $x)`: the angle of the vector, in degrees. The arguments must
    /// be mutually compatible but need not be unitless.
    pub fn atan2(args: Vec<CalculationArg>, options: &Options, span: Span) -> SassResult<Value> {
        let args = Self::simplify_arguments(args);
        Self::verify_exact_length(&args, 2, CalculationName::Atan2, span)?;

        match (&args[0], &args[1]) {
            (CalculationArg::Number(y), CalculationArg::Number(x))
                if (y.has_compatible_units(&x.unit)
                    || (y.unit == Unit::None && x.unit == Unit::None))
                    && y.unit != Unit::Percent
                    && !y.unit.is_complex()
                    && !x.unit.is_complex() =>
            {
                let x_value = x.num.convert(&x.unit, &y.unit).0;
                Ok(Value::Dimension(SassNumber {
                    num: Number(y.num.0.atan2(x_value).to_degrees()),
                    unit: Unit::Deg,
                    as_slash: None,
                }))
            }
            _ => Self::unsimplified_checked(CalculationName::Atan2, args, options, span),
        }
    }

    /// `pow($base, $exponent)` and `log($number, $base)`: both arguments must
    /// be unitless.
    pub fn unitless_binary(
        name: CalculationName,
        args: Vec<CalculationArg>,
        options: &Options,
        span: Span,
    ) -> SassResult<Value> {
        let args = Self::simplify_arguments(args);

        // `log()` takes an optional base; `pow()` requires both arguments.
        if name == CalculationName::Log {
            Self::verify_max_length(&args, 2, name, span)?;
            Self::verify_min_length(&args, 1, name, span)?;
        } else {
            Self::verify_exact_length(&args, 2, name, span)?;
        }

        let all_numbers = args
            .iter()
            .all(|arg| matches!(arg, CalculationArg::Number(..)));

        if !all_numbers {
            return Self::unsimplified_checked(name, args, options, span);
        }

        let mut numbers = Vec::with_capacity(args.len());
        for arg in &args {
            match arg {
                CalculationArg::Number(number) => {
                    Self::assert_unitless(number, options, span)?;
                    numbers.push(number.num.0);
                }
                _ => unreachable!("checked above"),
            }
        }

        let result = match name {
            CalculationName::Pow => numbers[0].powf(numbers[1]),
            CalculationName::Log => {
                if numbers.len() == 1 {
                    numbers[0].ln()
                } else {
                    numbers[0].ln() / numbers[1].ln()
                }
            }
            _ => unreachable!("not a unitless binary function: {}", name),
        };

        Ok(Value::Dimension(SassNumber::new_unitless(result)))
    }

    /// `mod()` and `rem()`: both take the unit of the dividend. `mod()` follows
    /// the sign of the divisor, `rem()` the sign of the dividend.
    pub fn modulo(
        name: CalculationName,
        args: Vec<CalculationArg>,
        options: &Options,
        span: Span,
    ) -> SassResult<Value> {
        let args = Self::simplify_arguments(args);
        Self::verify_exact_length(&args, 2, name, span)?;

        match (&args[0], &args[1]) {
            (CalculationArg::Number(dividend), CalculationArg::Number(divisor))
                if dividend.has_compatible_units(&divisor.unit)
                    && !dividend.unit.is_complex()
                    && !divisor.unit.is_complex() =>
            {
                let left = dividend.num;
                let right = divisor.num.convert(&divisor.unit, &dividend.unit);

                // `mod()` follows the divisor's sign, which is exactly Sass's
                // own `%` operator including its infinity and signed-zero
                // cases. `rem()` follows the dividend's sign, which is the
                // plain truncated remainder.
                let result = if name == CalculationName::Rem {
                    left.0 % right.0
                } else {
                    (left % right).0
                };

                Ok(Self::with_unit_of(result, dividend))
            }
            _ => Self::unsimplified_checked(name, args, options, span),
        }
    }

    /// `hypot()`: the Euclidean norm of its arguments, in the first argument's
    /// unit.
    pub fn hypot(args: Vec<CalculationArg>, options: &Options, span: Span) -> SassResult<Value> {
        let args = Self::simplify_arguments(args);
        Self::verify_min_length(&args, 1, CalculationName::Hypot, span)?;

        if args.iter().any(
            |arg| matches!(arg, CalculationArg::Number(number) if number.unit == Unit::Percent),
        ) {
            return Self::unsimplified_checked(CalculationName::Hypot, args, options, span);
        }

        let first = match &args[0] {
            CalculationArg::Number(number) if !number.unit.is_complex() => number.clone(),
            _ => return Self::unsimplified_checked(CalculationName::Hypot, args, options, span),
        };

        let mut sum = 0.0;
        for arg in &args {
            match arg {
                CalculationArg::Number(number)
                    if first.has_compatible_units(&number.unit) && !number.unit.is_complex() =>
                {
                    let value = number.num.convert(&number.unit, &first.unit).0;
                    sum += value * value;
                }
                _ => {
                    return Self::unsimplified_checked(CalculationName::Hypot, args, options, span);
                }
            }
        }

        Ok(Self::with_unit_of(sum.sqrt(), &first))
    }

    /// `calc-size()`: never simplified, but its arguments still have to be
    /// legal calculation values.
    pub fn calc_size(
        args: Vec<CalculationArg>,
        options: &Options,
        span: Span,
    ) -> SassResult<Value> {
        let args = Self::simplify_arguments(args);

        if args.is_empty() {
            return Err(("Missing argument.", span).into());
        }

        Self::verify_max_length(&args, 2, CalculationName::CalcSize, span)?;

        // `calc-size(var(--foo))` is legal: the single argument may itself
        // expand to both operands, so a short list is only rejected when every
        // argument is already known.
        if args.len() < 2
            && !args.iter().any(|arg| {
                matches!(
                    arg,
                    CalculationArg::String(..) | CalculationArg::Interpolation(..)
                )
            })
        {
            return Err(("2 arguments required, but only 1 was passed.", span).into());
        }

        Self::unsimplified_checked(CalculationName::CalcSize, args, options, span)
    }

    /// `round()` in its one-, two- and three-argument forms.
    ///
    /// With three arguments the first is a rounding strategy keyword, which
    /// must come from literal source text; an opaque first argument (a `var()`,
    /// say) leaves the whole calculation unsimplified rather than erroring.
    pub fn round(
        mut args: Vec<CalculationArg>,
        options: &Options,
        span: Span,
    ) -> SassResult<Value> {
        args = Self::simplify_arguments(args);

        if args.is_empty() {
            return Err(("Missing argument.", span).into());
        }
        Self::verify_max_length(&args, 3, CalculationName::Round, span)?;

        // A leading strategy keyword is only recognized in the three-argument
        // form; `round(nearest, 5)` is the documented "step is required" error.
        let leading_strategy = match &args[0] {
            CalculationArg::Interpolation(text) => Self::parse_round_strategy(text),
            _ => None,
        };

        if args.len() == 2 && leading_strategy.is_some() {
            // With an opaque second argument there is nothing to round yet, so
            // the calculation is emitted as-is rather than rejected.
            if !matches!(args[1], CalculationArg::Number(..)) {
                return Self::unsimplified_checked(CalculationName::Round, args, options, span);
            }

            return Err(("If strategy is not null, step is required.", span).into());
        }

        let (strategy, number, step) = if args.len() == 3 {
            let strategy = match &args[0] {
                CalculationArg::Interpolation(text) => match Self::parse_round_strategy(text) {
                    Some(strategy) => Some(strategy),
                    None => {
                        return Err((
                            format!("{} must be either nearest, up, down or to-zero.", text),
                            span,
                        )
                            .into());
                    }
                },
                CalculationArg::Number(number) => {
                    return Err((
                        format!(
                            "{} must be either nearest, up, down or to-zero.",
                            inspect_number(number, options, span)?
                        ),
                        span,
                    )
                        .into());
                }
                // An opaque first argument cannot be checked at compile time.
                _ => None,
            };

            match strategy {
                Some(strategy) => (strategy, &args[1], Some(&args[2])),
                None => {
                    return Self::unsimplified_checked(CalculationName::Round, args, options, span);
                }
            }
        } else if args.len() == 2 {
            (RoundStrategy::Nearest, &args[0], Some(&args[1]))
        } else {
            (RoundStrategy::Nearest, &args[0], None)
        };

        match (number, step) {
            (CalculationArg::Number(number), None) => {
                Ok(Self::with_unit_of(fuzzy_round(number.num.0), number))
            }
            (CalculationArg::Number(number), Some(CalculationArg::Number(step)))
                if number.has_compatible_units(&step.unit)
                    && !number.unit.is_complex()
                    && !step.unit.is_complex() =>
            {
                let step_value = step.num.convert(&step.unit, &number.unit).0;
                Ok(Self::with_unit_of(
                    round_with_step(strategy, number.num.0, step_value),
                    number,
                ))
            }
            _ => Self::unsimplified_checked(CalculationName::Round, args, options, span),
        }
    }

    /// Errors unless the calculation received exactly `len` arguments.
    fn verify_exact_length(
        args: &[CalculationArg],
        len: usize,
        name: CalculationName,
        span: Span,
    ) -> SassResult<()> {
        Self::verify_min_length(args, len, name, span)?;
        Self::verify_max_length(args, len, name, span)
    }

    /// Errors when too many arguments were passed, mirroring Dart Sass's
    /// "Only N arguments allowed" wording.
    fn verify_max_length(
        args: &[CalculationArg],
        max: usize,
        _name: CalculationName,
        span: Span,
    ) -> SassResult<()> {
        if args.len() <= max {
            return Ok(());
        }

        let argument = if max == 1 { "argument" } else { "arguments" };

        Err((
            format!(
                "Only {} {} allowed, but {} were passed.",
                max,
                argument,
                args.len()
            ),
            span,
        )
            .into())
    }

    /// Errors when too few arguments were passed. Zero arguments is always
    /// "Missing argument."; a short-but-nonempty list names the requirement.
    fn verify_min_length(
        args: &[CalculationArg],
        min: usize,
        _name: CalculationName,
        span: Span,
    ) -> SassResult<()> {
        if args.is_empty() {
            return Err(("Missing argument.", span).into());
        }

        if args.len() >= min {
            return Ok(());
        }

        let was_or_were = if args.len() == 1 { "was" } else { "were" };

        Err((
            format!(
                "{} arguments required, but only {} {} passed.",
                min,
                args.len(),
                was_or_were
            ),
            span,
        )
            .into())
    }

    fn simplify(arg: CalculationArg) -> CalculationArg {
        match arg {
            CalculationArg::Number(..)
            | CalculationArg::Operation { .. }
            | CalculationArg::Interpolation(..)
            | CalculationArg::String(..)
            | CalculationArg::Space(..) => arg,
            CalculationArg::Calculation(mut calc) => {
                if calc.name == CalculationName::Calc && !calc.args.is_empty() {
                    // Inlining a nested `calc()` around opaque text can change
                    // how the surrounding expression parses, so the text keeps
                    // parentheses when it begins or ends with something that
                    // would bind to a neighbour.
                    match calc.args.remove(0) {
                        CalculationArg::String(text) if needs_parens(&text) => {
                            CalculationArg::String(format!("({})", text))
                        }
                        CalculationArg::Interpolation(text) if needs_parens(&text) => {
                            CalculationArg::Interpolation(format!("({})", text))
                        }
                        arg => arg,
                    }
                } else {
                    CalculationArg::Calculation(calc)
                }
            }
        }
    }

    fn simplify_arguments(args: Vec<CalculationArg>) -> Vec<CalculationArg> {
        args.into_iter().map(Self::simplify).collect()
    }
}

/// The rounding strategies CSS `round()` supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RoundStrategy {
    Nearest,
    Up,
    Down,
    ToZero,
}

/// Rounds `number` to the nearest multiple of `step` under `strategy`.
///
/// Both values are already expressed in the same unit. The degenerate cases
/// follow CSS Values 4: a zero step gives NaN, two infinities give NaN, an
/// infinite number passes through, and an infinite step collapses to a signed
/// zero or infinity depending on the strategy.
pub(crate) fn round_with_step(strategy: RoundStrategy, number: f64, step: f64) -> f64 {
    if number.is_infinite() && step.is_infinite() {
        return f64::NAN;
    }

    if number.is_infinite() {
        return number;
    }

    if step.is_infinite() {
        // A zero keeps its own sign under every strategy.
        if number == 0.0 {
            return number;
        }

        return match strategy {
            RoundStrategy::Up => {
                if number.is_sign_negative() {
                    -0.0
                } else {
                    f64::INFINITY
                }
            }
            RoundStrategy::Down => {
                if number.is_sign_negative() {
                    f64::NEG_INFINITY
                } else {
                    0.0
                }
            }
            RoundStrategy::Nearest | RoundStrategy::ToZero => {
                if number.is_sign_negative() {
                    -0.0
                } else {
                    0.0
                }
            }
        };
    }

    if step == 0.0 {
        return f64::NAN;
    }

    let quotient = number / step;

    let multiple = match strategy {
        RoundStrategy::Nearest => fuzzy_round(quotient),
        // `up` means "toward positive infinity in value space", so which way
        // the quotient is rounded depends on the sign of the step.
        RoundStrategy::Up => {
            if step < 0.0 {
                fuzzy_floor(quotient)
            } else {
                fuzzy_ceil(quotient)
            }
        }
        RoundStrategy::Down => {
            if step < 0.0 {
                fuzzy_ceil(quotient)
            } else {
                fuzzy_floor(quotient)
            }
        }
        // `to-zero` rounds the quotient toward the number's own sign rather
        // than toward zero in value space; `round(to-zero, -120px, -25px)` is
        // -125px, not -100px.
        RoundStrategy::ToZero => {
            if number < 0.0 {
                fuzzy_ceil(quotient)
            } else {
                fuzzy_floor(quotient)
            }
        }
    };

    multiple * step
}

/// Whether inlining `text` into an enclosing calculation needs parentheses to
/// keep it from binding to a neighbouring term.
///
/// Dart Sass looks only at the first and last characters: leading or trailing
/// whitespace, a bracket, or an operator all make the text ambiguous once the
/// `calc()` around it is dropped.
pub(crate) fn needs_parens(text: &str) -> bool {
    let is_ambiguous =
        |c: char| c.is_whitespace() || matches!(c, '(' | ')' | '[' | ']' | '+' | '-' | '*' | '/');

    match (text.chars().next(), text.chars().next_back()) {
        (Some(first), Some(last)) => is_ambiguous(first) || is_ambiguous(last),
        _ => false,
    }
}

/// Whether `arg` contains any text Sass cannot resolve.
///
/// Whitespace-separated values in a calculation are only meaningful when at
/// least one part is opaque; otherwise the author simply forgot an operator.
pub(crate) fn contains_opaque_value(arg: &CalculationArg) -> bool {
    match arg {
        CalculationArg::Number(..) => false,
        CalculationArg::String(..) | CalculationArg::Interpolation(..) => true,
        CalculationArg::Operation { lhs, rhs, .. } => {
            contains_opaque_value(lhs) || contains_opaque_value(rhs)
        }
        CalculationArg::Space(args) => args.iter().any(contains_opaque_value),
        CalculationArg::Calculation(calc) => calc.args.iter().any(contains_opaque_value),
    }
}
