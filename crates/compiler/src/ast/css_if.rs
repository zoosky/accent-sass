//! The CSS `if()` function.
//!
//! CSS Values 5 gives `if()` a `;`-separated list of `condition: value`
//! branches, which is a different shape from the Sass ternary
//! `if($condition, $if-true, $if-false)` that shares the name. The two coexist:
//! a colon after the first clause selects this form, commas select the ternary.
//!
//! A condition is a boolean expression whose leaves are either `sass(...)`,
//! which Sass evaluates itself, or text only the browser can resolve. Sass
//! collapses what it can and emits the rest, so `if(sass(true) and css(): c)`
//! becomes `if(css(): c)`.

use std::sync::Arc;

use super::{AstExpr, Interpolation};

/// One `condition: value` branch of a CSS `if()`.
#[derive(Debug, Clone)]
pub struct CssIfBranch {
    pub condition: CssIfCondition,
    /// Kept unevaluated: branches after a decided one are never looked at, so
    /// `if(sass(true): c; else: $undefined)` must not raise.
    pub value: AstExpr,
}

/// A CSS `if()` expression, in source order.
#[derive(Debug, Clone)]
pub struct CssIfExpr {
    pub branches: Vec<CssIfBranch>,
}

/// The condition of one branch.
///
/// Interpolations inside [`CssIfCondition::Raw`] and the expression inside
/// [`CssIfCondition::Sass`] are held unevaluated, because a condition that is
/// never reached must never run.
#[derive(Debug, Clone)]
pub enum CssIfCondition {
    /// The bare `else` keyword, which always matches.
    Else,
    /// `sass(<expression>)`: decided at compile time.
    Sass(Arc<AstExpr>),
    /// Text Sass does not interpret, such as `media(...)` or a `var()`
    /// substitution. Emitted verbatim once its interpolations are resolved.
    Raw(Arc<Interpolation>),
    /// A parenthesized condition. The parentheses are part of the output, so
    /// they are kept rather than folded away at parse time.
    Paren(Box<Self>),
    Not(Box<Self>),
    /// Two or more `and`-separated conditions. CSS does not let `and` and `or`
    /// mix without parentheses, so a chain is always one or the other.
    And(Vec<Self>),
    Or(Vec<Self>),
}
