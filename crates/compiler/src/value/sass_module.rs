//! The first-class module value `meta.get-module` and `meta.load` return.

use std::{cell::RefCell, fmt, sync::Arc};

use crate::builtin::modules::Module;

/// A reference to a loaded module.
///
/// The value is the module itself, not a snapshot: `meta.module-variables`
/// on it reads the module's current state. Two references are equal when
/// they point at one module instance, which is why the built-in modules are
/// cached per compilation and why a file loaded twice is one module.
#[derive(Clone)]
pub struct SassModule(pub(crate) Arc<RefCell<Module>>);

impl SassModule {
    pub(crate) fn new(module: Arc<RefCell<Module>>) -> Self {
        Self(module)
    }

    /// The module this value refers to.
    pub(crate) fn inner(&self) -> &Arc<RefCell<Module>> {
        &self.0
    }

    /// The namespace `inspect` prints, when the module's URL has one.
    pub(crate) fn default_namespace(&self) -> Option<String> {
        (*self.0).borrow().default_namespace()
    }
}

impl PartialEq for SassModule {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for SassModule {}

impl fmt::Debug for SassModule {
    /// Prints the URL rather than the module's whole environment, which
    /// would recurse through every module it loaded.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SassModule")
            .field(&(*self.0).borrow().url())
            .finish()
    }
}
