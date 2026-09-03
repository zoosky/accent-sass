use std::{fmt, sync::Arc};

use crate::{
    ast::ArgumentResult,
    common::Identifier,
    error::SassResult,
    evaluate::{Environment, Visitor},
};

pub(crate) type BuiltinMixin = fn(ArgumentResult, &mut Visitor) -> SassResult<()>;

pub(crate) use crate::ast::AstMixin as UserDefinedMixin;

#[derive(Clone)]
pub(crate) enum Mixin {
    UserDefined(Arc<UserDefinedMixin>, Environment),
    /// A mixin implemented in Rust. The name and whether it takes a `@content`
    /// block are carried alongside the implementation so that first-class mixin
    /// values can report them, the way `AstMixin` already does for user-defined
    /// ones.
    Builtin(BuiltinMixin, Identifier, bool),
}

impl Mixin {
    /// The name the mixin was declared under, for `meta.inspect`.
    pub fn name(&self) -> Identifier {
        match self {
            Self::UserDefined(mixin, ..) => mixin.name,
            Self::Builtin(_, name, ..) => *name,
        }
    }

    /// Whether `@include`ing this mixin may be given a `@content` block.
    pub fn accepts_content(&self) -> bool {
        match self {
            Self::UserDefined(mixin, ..) => mixin.has_content,
            Self::Builtin(_, _, accepts_content) => *accepts_content,
        }
    }
}

impl PartialEq for Mixin {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Two mixins are the same mixin only if they come from the same
            // declaration: redefining a mixin under the same name produces a
            // value that is not equal to a reference taken beforehand.
            (Self::UserDefined(a, ..), Self::UserDefined(b, ..)) => Arc::ptr_eq(a, b),
            (Self::Builtin(a, name_a, ..), Self::Builtin(b, name_b, ..)) => {
                name_a == name_b && std::ptr::eq(*a as *const (), *b as *const ())
            }
            _ => false,
        }
    }
}

impl Eq for Mixin {}

impl fmt::Debug for Mixin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UserDefined(u, ..) => f
                .debug_struct("AstMixin")
                .field("name", &u.name)
                .field("args", &u.args)
                .field("body", &u.body)
                .field("has_content", &u.has_content)
                .finish(),
            Self::Builtin(_, name, accepts_content) => f
                .debug_struct("BuiltinMixin")
                .field("name", name)
                .field("accepts_content", accepts_content)
                .finish(),
        }
    }
}

/// A first-class mixin value, as returned by `meta.get-mixin`.
///
/// This wraps [`Mixin`] so that a mixin can live in the public [`Value`] enum
/// without exposing the evaluator's internals.
///
/// [`Value`]: crate::value::Value
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SassMixin(pub(crate) Box<Mixin>);

impl SassMixin {
    pub(crate) fn new(mixin: Mixin) -> Self {
        Self(Box::new(mixin))
    }

    pub(crate) fn inner(&self) -> &Mixin {
        &self.0
    }
}
