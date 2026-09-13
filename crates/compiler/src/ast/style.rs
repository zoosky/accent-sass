use codemap::{Span, Spanned};

use crate::{interner::InternedString, value::Value};

/// A style: `color: red`
#[derive(Clone, Debug)]
pub(crate) struct Style {
    pub property: InternedString,
    pub value: Box<Spanned<Value>>,
    pub parsed_as_sass_script: bool,
    /// Where the declaration was written, from its name to the end of its
    /// value, which is what a warning about the rule holding it points at.
    pub span: Span,
}
