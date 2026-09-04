use std::fmt::{self, Write};

use codemap::Span;

use crate::error::SassResult;

use super::{
    ComplexSelector, ComplexSelectorComponent, Namespace, Pseudo, SelectorList, SimpleSelector,
    Specificity,
};

/// A compound selector is composed of several
/// simple selectors
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct CompoundSelector {
    pub components: Vec<SimpleSelector>,
}

impl fmt::Display for CompoundSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut did_write = false;
        for simple in &self.components {
            if did_write {
                write!(f, "{}", simple)?;
            } else {
                let s = simple.to_string();
                if !s.is_empty() {
                    did_write = true;
                }
                write!(f, "{}", s)?;
            }
        }

        // If we emit an empty compound, it's because all of the components got
        // optimized out because they match all selectors, so we just emit the
        // universal selector.
        if !did_write {
            f.write_char('*')?;
        }

        Ok(())
    }
}

impl CompoundSelector {
    pub fn max_specificity(&self) -> i32 {
        self.specificity().max
    }

    pub fn min_specificity(&self) -> i32 {
        self.specificity().min
    }

    /// Returns tuple of (min, max) specificity
    pub fn specificity(&self) -> Specificity {
        let mut min = 0;
        let mut max = 0;
        for simple in &self.components {
            min += simple.min_specificity();
            max += simple.max_specificity();
        }
        Specificity::new(min, max)
    }

    pub fn is_invisible(&self) -> bool {
        self.components.iter().any(SimpleSelector::is_invisible)
    }

    pub fn is_super_selector(
        &self,
        other: &Self,
        parents: &Option<Vec<ComplexSelectorComponent>>,
    ) -> bool {
        for simple1 in &self.components {
            if let SimpleSelector::Pseudo(
                pseudo @ Pseudo {
                    selector: Some(..), ..
                },
            ) = simple1
            {
                if !pseudo.is_super_selector(other, parents.clone()) {
                    return false;
                }
            } else if !simple1.is_super_selector_of_compound(other) {
                return false;
            }
        }

        for simple2 in &other.components {
            if let SimpleSelector::Pseudo(Pseudo {
                is_class: false,
                selector: None,
                ..
            }) = simple2
            {
                if !simple2.is_super_selector_of_compound(self) {
                    return false;
                }
            }
        }

        true
    }

    /// Returns a new `CompoundSelector` based on `compound` with all
    /// `SimpleSelector::Parent`s replaced with `parent`.
    ///
    /// Returns `None` if `compound` doesn't contain any `SimpleSelector::Parent`s.
    pub fn resolve_parent_selectors(
        self,
        span: Span,
        parent: SelectorList,
    ) -> SassResult<Option<Vec<ComplexSelector>>> {
        let contains_selector_pseudo = self.components.iter().any(|simple| {
            if let SimpleSelector::Pseudo(Pseudo {
                selector: Some(sel),
                ..
            }) = simple
            {
                sel.contains_parent_selector()
            } else {
                false
            }
        });

        if !contains_selector_pseudo && !self.components[0].is_parent() {
            return Ok(None);
        }

        let resolved_members: Vec<SimpleSelector> = if contains_selector_pseudo {
            self.components
                .clone()
                .into_iter()
                .map(|simple| {
                    if let SimpleSelector::Pseudo(mut pseudo) = simple {
                        if let Some(sel) = pseudo.selector.clone() {
                            if !sel.contains_parent_selector() {
                                return Ok(SimpleSelector::Pseudo(pseudo));
                            }

                            pseudo.selector = Some(Box::new(sel.resolve_parent_selectors(
                                Some(parent.clone()),
                                false,
                                false,
                            )?));
                        }

                        Ok(SimpleSelector::Pseudo(pseudo))
                    } else {
                        Ok(simple)
                    }
                })
                .collect::<SassResult<Vec<SimpleSelector>>>()?
        } else {
            self.components.clone()
        };

        if let Some(SimpleSelector::Parent(suffix)) = self.components.first() {
            if self.components.len() == 1 && suffix.is_none() {
                return Ok(Some(parent.components));
            }
        } else {
            return Ok(Some(vec![ComplexSelector::new(
                vec![ComplexSelectorComponent::Compound(CompoundSelector {
                    components: resolved_members,
                })],
                false,
            )]));
        }

        let parent_span = parent.span;

        Ok(Some(
            parent
                .components
                .into_iter()
                .map(move |mut complex| {
                    let last_component = complex.components.last();
                    let last = if let Some(ComplexSelectorComponent::Compound(c)) = last_component {
                        c.clone()
                    } else {
                        return Err((
                            format!("Parent \"{}\" is incompatible with this selector.", complex),
                            span,
                        )
                            .into());
                    };

                    let mut components = last.components;

                    if let Some(SimpleSelector::Parent(Some(suffix))) = self.components.first() {
                        let mut end = components.pop().unwrap();
                        end.add_suffix(suffix, parent_span)?;
                        components.push(end);
                    }

                    components.extend(resolved_members.clone().into_iter().skip(1));

                    let last = CompoundSelector { components };

                    complex.components.pop();

                    let mut components = complex.components;
                    components.push(ComplexSelectorComponent::Compound(last));

                    Ok(ComplexSelector::new(components, complex.line_break))
                })
                .collect::<SassResult<Vec<ComplexSelector>>>()?,
        ))
    }

    /// Returns a `CompoundSelector` that matches only elements that are matched by
    /// both `compound1` and `compound2`.
    ///
    /// If no such selector can be produced, returns `None`.
    pub fn unify(self, other: Self) -> Option<Self> {
        // A pseudo-element and the pseudo-classes written after it belong
        // together at the end of the compound, so they are set aside and
        // merged separately: unifying `.x` with `.y::scrollbar:horizontal`
        // gives `.x.y::scrollbar:horizontal`, not `.x.y:horizontal::scrollbar`.
        let (head_one, tail_one) = Self::split_at_pseudo_element(self.components);
        let (head_two, tail_two) = Self::split_at_pseudo_element(other.components);

        // The receiver's simple selectors keep their positions and the other
        // compound's are merged into them, so `.c` unified with `.e` is `.c.e`
        // rather than `.e.c`.
        let mut components = head_one;
        for simple in head_two {
            components = if components.is_empty() {
                vec![simple]
            } else {
                simple.unify(std::mem::take(&mut components))?
            };
        }

        components.extend(Self::unify_pseudo_element_tails(tail_one, tail_two)?);

        Some(Self { components })
    }

    /// Splits a compound into the part before its first pseudo-element and that
    /// pseudo-element together with everything after it.
    fn split_at_pseudo_element(
        components: Vec<SimpleSelector>,
    ) -> (Vec<SimpleSelector>, Vec<SimpleSelector>) {
        match components
            .iter()
            .position(SimpleSelector::is_pseudo_element)
        {
            Some(idx) => {
                let mut components = components;
                let tail = components.split_off(idx);
                (components, tail)
            }
            None => (components, Vec::new()),
        }
    }

    /// Merges two pseudo-element tails.
    ///
    /// A compound may carry only one pseudo-element, so two tails unify only
    /// when they name the same one; the pseudo-classes that follow are then
    /// concatenated.
    fn unify_pseudo_element_tails(
        one: Vec<SimpleSelector>,
        two: Vec<SimpleSelector>,
    ) -> Option<Vec<SimpleSelector>> {
        if one.is_empty() {
            return Some(two);
        }

        if two.is_empty() {
            return Some(one);
        }

        if one[0] != two[0] {
            return None;
        }

        let mut result = one;

        for simple in two.into_iter().skip(1) {
            if !result.contains(&simple) {
                result.push(simple);
            }
        }

        Some(result)
    }

    /// Adds a `SimpleSelector::Parent` to the beginning of `compound`, or returns `None` if
    /// that wouldn't produce a valid selector.
    pub fn prepend_parent(mut self) -> Option<Self> {
        Some(match self.components.first()? {
            SimpleSelector::Universal(..) => return None,
            SimpleSelector::Type(name) => {
                if name.namespace != Namespace::None {
                    return None;
                }
                let mut components = vec![SimpleSelector::Parent(Some(name.ident.clone()))];
                components.extend(self.components.into_iter().skip(1));

                Self { components }
            }
            _ => {
                let mut components = vec![SimpleSelector::Parent(None)];
                components.append(&mut self.components);
                Self { components }
            }
        })
    }
}
