use std::{
    collections::HashSet,
    fmt::{self, Display, Write},
    hash::{Hash, Hasher},
    sync::atomic::{AtomicU32, Ordering as AtomicOrdering},
};

use codemap::Span;

use crate::error::SassResult;

use super::{CompoundSelector, Pseudo, SelectorList, SimpleSelector, Specificity};

pub(crate) static COMPLEX_SELECTOR_UNIQUE_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Debug)]
pub(crate) struct ComplexSelectorHashSet(HashSet<u32>);

impl ComplexSelectorHashSet {
    pub fn new() -> Self {
        Self(HashSet::new())
    }

    pub fn insert(&mut self, complex: &ComplexSelector) -> bool {
        self.0.insert(complex.unique_id)
    }

    pub fn contains(&self, complex: &ComplexSelector) -> bool {
        self.0.contains(&complex.unique_id)
    }

    pub fn extend<'a>(&mut self, complexes: impl Iterator<Item = &'a ComplexSelector>) {
        self.0.extend(complexes.map(|complex| complex.unique_id));
    }
}

/// A complex selector.
///
/// A complex selector is composed of `CompoundSelector`s separated by
/// `Combinator`s. It selects elements based on their parent selectors.
#[derive(Clone, Debug)]
pub(crate) struct ComplexSelector {
    /// The components of this selector.
    ///
    /// This is never empty.
    ///
    /// Descendant combinators aren't explicitly represented here. If two
    /// `CompoundSelector`s are adjacent to one another, there's an implicit
    /// descendant combinator between them.
    ///
    /// It's possible for multiple `Combinator`s to be adjacent to one another.
    /// This isn't valid CSS, but Sass supports it for CSS hack purposes.
    pub components: Vec<ComplexSelectorComponent>,

    /// Whether a line break should be emitted *before* this selector.
    pub line_break: bool,

    /// A unique identifier for this complex selector. Used to perform a pointer
    /// equality check, like would be done for objects in a language like JavaScript
    /// or dart
    unique_id: u32,
}

impl PartialEq for ComplexSelector {
    fn eq(&self, other: &Self) -> bool {
        self.components == other.components
    }
}

impl Eq for ComplexSelector {}

impl Hash for ComplexSelector {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.components.hash(state);
    }
}

impl fmt::Display for ComplexSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut last_component = None;

        for component in &self.components {
            if let Some(c) = last_component
                && !omit_spaces_around(c)
                && !omit_spaces_around(component)
            {
                f.write_char(' ')?;
            }
            write!(f, "{}", component)?;
            last_component = Some(component);
        }
        Ok(())
    }
}

/// When `style` is `OutputStyle::compressed`, omit spaces around combinators.
fn omit_spaces_around(component: &ComplexSelectorComponent) -> bool {
    // todo: compressed
    let is_compressed = false;
    is_compressed && matches!(component, ComplexSelectorComponent::Combinator(..))
}

impl ComplexSelector {
    pub fn new(components: Vec<ComplexSelectorComponent>, line_break: bool) -> Self {
        Self {
            components,
            line_break,
            unique_id: COMPLEX_SELECTOR_UNIQUE_ID.fetch_add(1, AtomicOrdering::Relaxed),
        }
    }

    pub fn max_specificity(&self) -> i32 {
        self.specificity().min
    }

    pub fn min_specificity(&self) -> i32 {
        self.specificity().max
    }

    pub fn specificity(&self) -> Specificity {
        let mut min = 0;
        let mut max = 0;
        for component in &self.components {
            if let ComplexSelectorComponent::Compound(compound) = component {
                min += compound.min_specificity();
                max += compound.max_specificity();
            }
        }
        Specificity::new(min, max)
    }

    pub fn is_invisible(&self) -> bool {
        self.components
            .iter()
            .any(ComplexSelectorComponent::is_invisible)
    }

    /// Returns whether `self` is a superselector of `other`.
    ///
    /// That is, whether `self` matches every element that `other` matches, as well
    /// as possibly additional elements.
    ///
    /// A port of dart-sass 1.103.1's `complexIsSuperselector`. The walk
    /// matches each compound of `self` against the earliest compound of
    /// `other` it is a superselector of, then checks the combinators on both
    /// sides of that match. Selectors with a leading or trailing combinator,
    /// or with two combinators in a row, are never superselectors of anything
    /// and never have one.
    pub fn is_super_selector(&self, other: &Self) -> bool {
        let (Some(steps1), Some(steps2)) = (
            SuperselectorStep::split(&self.components),
            SuperselectorStep::split(&other.components),
        ) else {
            return false;
        };

        // Selectors with trailing operators are neither superselectors nor
        // subselectors.
        match (steps1.last(), steps2.last()) {
            (Some(last1), Some(last2))
                if last1.combinators.is_empty() && last2.combinators.is_empty() => {}
            _ => return false,
        }

        // The components of `other` from step `from` up to, not including,
        // step `to`, with the combinators that follow each one: the parents a
        // selector pseudo in `self` may need to see. dart-sass passes them
        // only to compounds whose superselector semantics depend on them.
        let parents = |compound1: &CompoundSelector, from: usize, to: usize| {
            compound1
                .has_complicated_superselector_semantics()
                .then(|| other.components[steps2[from].start..steps2[to].start].to_vec())
        };

        let mut i1 = 0;
        let mut i2 = 0;
        let mut previous_combinator: Option<Combinator> = None;

        loop {
            let remaining1 = steps1.len() - i1;
            let remaining2 = steps2.len() - i2;
            if remaining1 == 0 || remaining2 == 0 {
                return false;
            }

            // More complex selectors are never superselectors of less complex
            // ones.
            if remaining1 > remaining2 {
                return false;
            }

            let component1 = &steps1[i1];
            if component1.combinators.len() > 1 {
                return false;
            }

            if remaining1 == 1 {
                if steps2.iter().any(|parent| parent.combinators.len() > 1) {
                    return false;
                }
                let last = steps2.len() - 1;
                return component1.compound.is_super_selector(
                    steps2[last].compound,
                    &parents(component1.compound, i2, last),
                );
            }

            // Find the first step `end` of `other` such that the steps from
            // `i2` through `end` are a subselector of `component1`.
            let mut end = i2;
            loop {
                let component2 = &steps2[end];
                if component2.combinators.len() > 1 {
                    return false;
                }
                if component1
                    .compound
                    .is_super_selector(component2.compound, &parents(component1.compound, i2, end))
                {
                    break;
                }

                end += 1;
                if end == steps2.len() - 1 {
                    // Stop before the superselector would encompass all of
                    // `other`: `self` has more than one step left, and
                    // consuming all of `other` would leave nothing for the
                    // rest of it to match.
                    return false;
                }
            }

            if !compatible_with_previous_combinator(previous_combinator, &steps2[i2..end]) {
                return false;
            }

            let combinator1 = component1.combinators.first().copied();
            let combinator2 = steps2[end].combinators.first().copied();
            if !is_supercombinator(combinator1, combinator2) {
                return false;
            }

            i1 += 1;
            i2 = end + 1;
            previous_combinator = combinator1;

            if steps1.len() - i1 == 1 {
                let rejected = match combinator1 {
                    // `.foo ~ .bar` is only a superselector of selectors that
                    // *exclusively* contain subcombinators of `~`.
                    Some(Combinator::FollowingSibling) => {
                        !steps2[i2..steps2.len() - 1].iter().all(|component| {
                            is_supercombinator(combinator1, component.combinators.first().copied())
                        })
                    }
                    // `.foo > .bar` and `.foo + .bar` aren't superselectors of
                    // any selectors with more than one combinator.
                    Some(_) => steps2.len() - i2 > 1,
                    None => false,
                };
                if rejected {
                    return false;
                }
            }
        }
    }

    pub fn contains_parent_selector(&self) -> bool {
        self.components.iter().any(|c| {
            if let ComplexSelectorComponent::Compound(compound) = c {
                compound.components.iter().any(|simple| {
                    if simple.is_parent() {
                        return true;
                    }
                    if let SimpleSelector::Pseudo(Pseudo {
                        selector: Some(sel),
                        ..
                    }) = simple
                    {
                        return sel.contains_parent_selector();
                    }
                    false
                })
            } else {
                false
            }
        })
    }

    /// Whether this contains a parent selector that carries a suffix, such as
    /// the `&--x` in `&--x .y`.
    ///
    /// A bare `&` can stand on its own with no parent -- the browser resolves
    /// it as the CSS nesting selector -- but a suffix has nothing to attach
    /// itself to, so dart-sass reports the two cases differently. Searches
    /// inside a pseudo-selector's argument as well, because `:is(&--x)` is an
    /// error too.
    pub fn contains_suffixed_parent_selector(&self) -> bool {
        self.components.iter().any(|c| {
            if let ComplexSelectorComponent::Compound(compound) = c {
                compound.components.iter().any(|simple| match simple {
                    SimpleSelector::Parent(suffix) => suffix.is_some(),
                    SimpleSelector::Pseudo(Pseudo {
                        selector: Some(sel),
                        ..
                    }) => sel.contains_suffixed_parent_selector(),
                    _ => false,
                })
            } else {
                false
            }
        })
    }
}

/// One compound selector of a complex selector with the combinators written
/// after it.
///
/// This is the unit dart-sass's `ComplexSelectorComponent` holds, and the one
/// its superselector algorithm walks. This compiler stores compounds and
/// combinators interleaved instead, so [`ComplexSelector::is_super_selector`]
/// regroups them rather than rewriting the algorithm around the difference.
struct SuperselectorStep<'a> {
    compound: &'a CompoundSelector,
    combinators: Vec<Combinator>,
    /// The index of `compound` in the interleaved components, so a run of
    /// steps can be sliced back out of them.
    start: usize,
}

impl<'a> SuperselectorStep<'a> {
    /// Groups `components` into steps.
    ///
    /// Returns `None` when `components` begins with a combinator: dart-sass
    /// never treats a selector with leading combinators as a superselector or
    /// a subselector.
    fn split(components: &'a [ComplexSelectorComponent]) -> Option<Vec<Self>> {
        let mut steps: Vec<Self> = Vec::new();
        for (start, component) in components.iter().enumerate() {
            match component {
                ComplexSelectorComponent::Compound(compound) => steps.push(Self {
                    compound,
                    combinators: Vec::new(),
                    start,
                }),
                ComplexSelectorComponent::Combinator(combinator) => {
                    steps.last_mut()?.combinators.push(*combinator);
                }
            }
        }
        Some(steps)
    }
}

/// Whether a match that skipped the steps `parents` can follow a previous
/// match joined to it by `previous`.
///
/// The child and next-sibling combinators need the *immediately* following
/// compound to match, so nothing may be skipped after them. The following
/// sibling combinator allows skipped compounds, but only siblings.
fn compatible_with_previous_combinator(
    previous: Option<Combinator>,
    parents: &[SuperselectorStep<'_>],
) -> bool {
    if parents.is_empty() {
        return true;
    }
    match previous {
        None => true,
        Some(Combinator::FollowingSibling) => parents.iter().all(|component| {
            matches!(
                component.combinators.first(),
                Some(Combinator::FollowingSibling | Combinator::NextSibling)
            )
        }),
        Some(_) => false,
    }
}

/// Whether `combinator1` matches everything `combinator2` does, where `None`
/// is the descendant combinator.
fn is_supercombinator(combinator1: Option<Combinator>, combinator2: Option<Combinator>) -> bool {
    combinator1 == combinator2
        || (combinator1.is_none() && combinator2 == Some(Combinator::Child))
        || (combinator1 == Some(Combinator::FollowingSibling)
            && combinator2 == Some(Combinator::NextSibling))
}

#[derive(Clone, Debug, Eq, PartialEq, Copy, Hash)]
pub(crate) enum Combinator {
    /// Matches the right-hand selector if it's immediately adjacent to the
    /// left-hand selector in the DOM tree.
    ///
    /// `'+'`
    NextSibling,

    /// Matches the right-hand selector if it's a direct child of the left-hand
    /// selector in the DOM tree.
    ///
    /// `'>'`
    Child,

    /// Matches the right-hand selector if it comes after the left-hand selector
    /// in the DOM tree.
    ///
    /// `'~'`
    FollowingSibling,
}

impl Display for Combinator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(match self {
            Self::NextSibling => '+',
            Self::Child => '>',
            Self::FollowingSibling => '~',
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) enum ComplexSelectorComponent {
    Combinator(Combinator),
    Compound(CompoundSelector),
}

impl ComplexSelectorComponent {
    pub fn is_invisible(&self) -> bool {
        match self {
            Self::Combinator(..) => false,
            Self::Compound(c) => c.is_invisible(),
        }
    }

    pub fn is_compound(&self) -> bool {
        matches!(self, Self::Compound(..))
    }

    pub fn is_combinator(&self) -> bool {
        matches!(self, Self::Combinator(..))
    }

    pub fn resolve_parent_selectors(
        self,
        span: Span,
        parent: SelectorList,
    ) -> SassResult<Option<Vec<ComplexSelector>>> {
        match self {
            Self::Compound(c) => c.resolve_parent_selectors(span, parent),
            Self::Combinator(..) => todo!(),
        }
    }

    pub fn as_compound(&self) -> &CompoundSelector {
        match self {
            Self::Compound(c) => c,
            Self::Combinator(..) => unreachable!(),
        }
    }
}

impl Display for ComplexSelectorComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compound(c) => write!(f, "{}", c),
            Self::Combinator(c) => write!(f, "{}", c),
        }
    }
}
