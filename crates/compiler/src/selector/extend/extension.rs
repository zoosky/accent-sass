use codemap::Span;

use crate::{ast::CssMediaQuery, error::SassError};

use super::{ComplexSelector, SimpleSelector};

#[derive(Clone, Debug)]
pub(crate) struct Extension {
    /// The selector in which the `@extend` appeared.
    pub extender: ComplexSelector,

    /// The selector that's being extended.
    ///
    /// `None` for one-off extensions.
    pub target: Option<SimpleSelector>,

    /// The minimum specificity required for any selector generated from this
    /// extender.
    pub specificity: i32,

    /// Whether this extension is optional.
    pub is_optional: bool,

    /// Whether this is a one-off extender representing a selector that was
    /// originally in the document, rather than one defined with `@extend`.
    pub is_original: bool,

    /// The media query context to which this extend is restricted, or `None` if
    /// it can apply within any context.
    pub media_context: Option<Vec<CssMediaQuery>>,

    /// The span in which `extender` was defined.
    pub span: Span,

    /// The two extensions this one was merged from, when it is the result of
    /// `MergedExtension::merge` on two extensions that must both stay
    /// resolvable. `None` on ordinary extensions.
    pub left: Option<Box<Extension>>,
    pub right: Option<Box<Extension>>,
}

impl Extension {
    pub fn one_off(
        extender: ComplexSelector,
        specificity: Option<i32>,
        is_original: bool,
        span: Span,
    ) -> Self {
        Self {
            specificity: specificity.unwrap_or_else(|| extender.max_specificity()),
            extender,
            target: None,
            span,
            is_optional: true,
            is_original,
            media_context: None,
            left: None,
            right: None,
        }
    }

    /// Checks that `media_context` -- the media query context of the selector
    /// being extended -- is compatible with the context this extension was
    /// written in.
    ///
    /// An extension written at the top level applies anywhere, because the
    /// rule it came from is not itself inside a query. One written inside a
    /// query may only extend a selector in the identical query: extending
    /// across queries would need the extended rule to exist in a context it
    /// was never written for.
    ///
    /// Optionality does not enter into it. `!optional` says the extension
    /// need not match anything; it does not permit a match that would be
    /// wrong.
    ///
    /// Returns the error rather than raising it, because the extend machinery
    /// returns `Option` to mean "extension did not apply" and cannot carry a
    /// `Result` out of the middle of that. See `ExtensionStore::defer_error`.
    pub fn check_media_context(
        &self,
        media_context: &Option<Vec<CssMediaQuery>>,
    ) -> Option<Box<SassError>> {
        let expected = self.media_context.as_ref()?;

        if media_context.as_ref() == Some(expected) {
            return None;
        }

        Some(
            (
                "You may not @extend selectors across media queries.",
                self.span,
            )
                .into(),
        )
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn with_extender(mut self, extender: ComplexSelector) -> Self {
        self.extender = extender;
        self
    }

    /// The plain extensions this one stands for: itself, or -- for a merged
    /// extension -- the sides it was merged from, recursively.
    pub fn unmerge(&self) -> Vec<Extension> {
        match (&self.left, &self.right) {
            (Some(left), Some(right)) => {
                let mut result = left.unmerge();
                result.extend(right.unmerge());
                result
            }
            _ => vec![self.clone()],
        }
    }
}
