use std::{
    cell::RefCell,
    collections::{HashSet, hash_set::IntoIter},
    hash::{Hash, Hasher},
    ops::Deref,
    ptr,
    rc::Rc,
};

use crate::selector::{Selector, SelectorList};

#[derive(Debug, Clone)]
pub(crate) struct ExtendedSelector(Rc<RefCell<SelectorList>>);

/// Two `ExtendedSelector`s are the same only when they are the same rule.
///
/// Equality is by identity, not by value, because a stylesheet may hold
/// several rules whose selectors are equal -- `\.foo` and `\2E foo`
/// normalize to the same list -- and each has to receive its own `@extend`.
/// It also has to agree with `Hash` below, which hashes the pointer.
impl PartialEq for ExtendedSelector {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ExtendedSelector {}

impl Hash for ExtendedSelector {
    /// Hashes the pointer, which is what `PartialEq` above compares.
    ///
    /// Hashing the value instead would be both slower and wrong: two rules
    /// with equal selectors are different rules, and the `HashSet` in
    /// `SelectorHashSet` has to keep them apart.
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(&*self.0, state);
    }
}

impl ExtendedSelector {
    pub fn new(selector: SelectorList) -> Self {
        Self(Rc::new(RefCell::new(selector)))
    }

    pub fn is_invisible(&self) -> bool {
        (*self.0).borrow().is_invisible()
    }

    pub fn into_selector(self) -> Selector {
        Selector(match Rc::try_unwrap(self.0) {
            Ok(v) => v.into_inner(),
            Err(v) => v.borrow().clone(),
        })
    }

    pub fn as_selector_list(&self) -> impl Deref<Target = SelectorList> + '_ {
        self.0.borrow()
    }

    pub fn set_inner(&mut self, selector: SelectorList) {
        self.0.replace(selector);
    }
}

/// There is the potential for danger here by modifying the hash
/// through `RefCell`, but I haven't come up with a good solution
/// for this yet (we can't just use a `Vec` because linear insert)
/// is too big of a penalty
///
/// In practice, I have yet to find a test case that can demonstrate
/// an issue with storing a `RefCell`.
#[derive(Clone, Debug)]
pub(crate) struct SelectorHashSet(HashSet<ExtendedSelector>);

impl SelectorHashSet {
    pub fn new() -> Self {
        Self(HashSet::new())
    }

    pub fn insert(&mut self, selector: ExtendedSelector) {
        self.0.insert(selector);
    }
}

impl IntoIterator for SelectorHashSet {
    type Item = ExtendedSelector;
    type IntoIter = IntoIter<ExtendedSelector>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
