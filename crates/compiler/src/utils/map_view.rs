use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
    fmt,
    sync::Arc,
};

use indexmap::IndexSet;

use crate::common::Identifier;

pub(crate) trait MapView: fmt::Debug {
    type Value;
    fn get(&self, name: Identifier) -> Option<Self::Value>;
    fn remove(&self, name: Identifier) -> Option<Self::Value>;
    fn insert(&self, name: Identifier, value: Self::Value) -> Option<Self::Value>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn contains_key(&self, k: Identifier) -> bool {
        self.get(k).is_some()
    }
    // todo: wildly ineffecient to return vec here, because of the arbitrary nesting of Self
    fn keys(&self) -> Vec<Identifier>;
    fn iter(&self) -> Vec<(Identifier, Self::Value)>;

    /// Identifies the map that actually holds `name`, seen through however many
    /// views wrap it.
    ///
    /// Two modules that forward the same declaration are not in conflict, so
    /// conflict checks compare identities rather than names. `None` means the
    /// name is absent.
    fn identity(&self, name: Identifier) -> Option<usize>;
}

impl<T> MapView for Arc<dyn MapView<Value = T>> {
    type Value = T;
    fn get(&self, name: Identifier) -> Option<Self::Value> {
        (**self).get(name)
    }
    fn remove(&self, name: Identifier) -> Option<Self::Value> {
        (**self).remove(name)
    }
    fn insert(&self, name: Identifier, value: Self::Value) -> Option<Self::Value> {
        (**self).insert(name, value)
    }
    fn len(&self) -> usize {
        (**self).len()
    }
    fn keys(&self) -> Vec<Identifier> {
        (**self).keys()
    }

    fn iter(&self) -> Vec<(Identifier, Self::Value)> {
        (**self).iter()
    }

    fn identity(&self, name: Identifier) -> Option<usize> {
        (**self).identity(name)
    }
}

#[derive(Debug)]
pub(crate) struct BaseMapView<T>(pub Arc<RefCell<BTreeMap<Identifier, T>>>);

impl<T> Clone for BaseMapView<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct UnprefixedMapView<V: fmt::Debug + Clone, T: MapView<Value = V> + Clone>(
    pub T,
    pub String,
);

#[derive(Debug, Clone)]
pub(crate) struct PrefixedMapView<V: fmt::Debug + Clone, T: MapView<Value = V> + Clone>(
    pub T,
    pub String,
);

impl<T: fmt::Debug + Clone> MapView for BaseMapView<T> {
    type Value = T;
    fn get(&self, name: Identifier) -> Option<Self::Value> {
        (*self.0).borrow().get(&name).cloned()
    }

    fn len(&self) -> usize {
        (*self.0).borrow().len()
    }

    fn remove(&self, name: Identifier) -> Option<Self::Value> {
        (*self.0).borrow_mut().remove(&name)
    }

    fn insert(&self, name: Identifier, value: Self::Value) -> Option<Self::Value> {
        (*self.0).borrow_mut().insert(name, value)
    }

    fn keys(&self) -> Vec<Identifier> {
        (*self.0).borrow().keys().copied().collect()
    }

    fn iter(&self) -> Vec<(Identifier, Self::Value)> {
        (*self.0).borrow().clone().into_iter().collect()
    }

    fn identity(&self, name: Identifier) -> Option<usize> {
        if !(*self.0).borrow().contains_key(&name) {
            return None;
        }

        Some(Arc::as_ptr(&self.0) as *const () as usize)
    }
}

impl<V: fmt::Debug + Clone, T: MapView<Value = V> + Clone> MapView for UnprefixedMapView<V, T> {
    type Value = V;
    fn get(&self, name: Identifier) -> Option<Self::Value> {
        let name = Identifier::from(format!("{}{}", self.1, name));
        self.0.get(name)
    }

    fn remove(&self, name: Identifier) -> Option<Self::Value> {
        let name = Identifier::from(format!("{}{}", self.1, name));
        self.0.remove(name)
    }

    fn insert(&self, name: Identifier, value: Self::Value) -> Option<Self::Value> {
        let name = Identifier::from(format!("{}{}", self.1, name));
        self.0.insert(name, value)
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn keys(&self) -> Vec<Identifier> {
        self.0
            .keys()
            .into_iter()
            .filter(|key| key.as_str().starts_with(&self.1))
            .map(|key| Identifier::from(key.as_str().strip_prefix(&self.1).unwrap()))
            .collect()
    }

    fn iter(&self) -> Vec<(Identifier, Self::Value)> {
        self.0
            .iter()
            .into_iter()
            .filter_map(|(key, value)| {
                let stripped = key.as_str().strip_prefix(&*self.1)?;

                Some((Identifier::from(stripped), value))
            })
            .collect()
    }

    fn identity(&self, name: Identifier) -> Option<usize> {
        self.0
            .identity(Identifier::from(format!("{}{}", self.1, name)))
    }
}

impl<V: fmt::Debug + Clone, T: MapView<Value = V> + Clone> MapView for PrefixedMapView<V, T> {
    type Value = V;
    fn get(&self, name: Identifier) -> Option<Self::Value> {
        if !name.as_str().starts_with(&self.1) {
            return None;
        }

        let name = Identifier::from(name.as_str().strip_prefix(&self.1).unwrap());

        self.0.get(name)
    }

    fn remove(&self, name: Identifier) -> Option<Self::Value> {
        if !name.as_str().starts_with(&self.1) {
            return None;
        }

        let name = Identifier::from(name.as_str().strip_prefix(&self.1).unwrap());

        self.0.remove(name)
    }

    fn insert(&self, name: Identifier, value: Self::Value) -> Option<Self::Value> {
        if !name.as_str().starts_with(&self.1) {
            return None;
        }

        let name = Identifier::from(name.as_str().strip_prefix(&self.1).unwrap());

        self.0.insert(name, value)
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn keys(&self) -> Vec<Identifier> {
        // Every key of the underlying map is visible through this view; it is
        // the view's names that carry the prefix, not the map's.
        self.0
            .keys()
            .into_iter()
            .map(|key| Identifier::from(format!("{}{}", self.1, key)))
            .collect()
    }

    fn iter(&self) -> Vec<(Identifier, Self::Value)> {
        self.0
            .iter()
            .into_iter()
            .map(|(key, value)| (Identifier::from(format!("{}{}", self.1, key)), value))
            .collect()
    }

    fn identity(&self, name: Identifier) -> Option<usize> {
        let name = name.as_str().strip_prefix(&*self.1)?;

        self.0.identity(Identifier::from(name))
    }
}

/// A mostly-unmodifiable view of a map that only allows certain keys to be
/// accessed.
///
/// Whether or not the underlying map contains keys that aren't allowed, this
/// view will behave as though it doesn't contain them.
///
/// The underlying map's values may change independently of this view, but its
/// set of keys may not.
///
/// This is unmodifiable *except for the [remove] method*, which is used for
/// `@used with` to mark configured variables as used.
#[derive(Debug, Clone)]
pub(crate) struct LimitedMapView<V: fmt::Debug + Clone, T: MapView<Value = V> + Clone>(
    pub T,
    // Ordered, so that the members this view exposes come out in the same order
    // every run -- `meta.module-variables` and friends return them.
    pub IndexSet<Identifier>,
);

impl<V: fmt::Debug + Clone, T: MapView<Value = V> + Clone> LimitedMapView<V, T> {
    pub fn safelist(map: T, keys: &HashSet<Identifier>) -> Self {
        // Iterating the underlying map rather than the safelist keeps the
        // map's own order.
        let allowed = map
            .keys()
            .into_iter()
            .filter(|key| keys.contains(key))
            .collect();

        Self(map, allowed)
    }

    pub fn blocklist(map: T, blocklist: &HashSet<Identifier>) -> Self {
        let keys = map
            .keys()
            .into_iter()
            .filter(|key| !blocklist.contains(key))
            .collect();

        Self(map, keys)
    }
}

impl<V: fmt::Debug + Clone, T: MapView<Value = V> + Clone> MapView for LimitedMapView<V, T> {
    type Value = V;
    fn get(&self, name: Identifier) -> Option<Self::Value> {
        if !self.1.contains(&name) {
            return None;
        }

        self.0.get(name)
    }

    fn remove(&self, name: Identifier) -> Option<Self::Value> {
        if !self.1.contains(&name) {
            return None;
        }

        self.0.remove(name)
    }

    fn insert(&self, name: Identifier, value: Self::Value) -> Option<Self::Value> {
        if !self.1.contains(&name) {
            return None;
        }

        self.0.insert(name, value)
    }

    fn len(&self) -> usize {
        self.1.len()
    }

    fn keys(&self) -> Vec<Identifier> {
        self.1.iter().copied().collect()
    }

    fn iter(&self) -> Vec<(Identifier, Self::Value)> {
        self.1
            .iter()
            .filter_map(|key| Some((*key, self.0.get(*key)?)))
            .collect()
    }

    fn identity(&self, name: Identifier) -> Option<usize> {
        if !self.1.contains(&name) {
            return None;
        }

        self.0.identity(name)
    }
}

#[derive(Debug)]
pub(crate) struct MergedMapView<V: fmt::Debug + Clone>(
    pub Vec<Arc<dyn MapView<Value = V>>>,
    // An ordered set, not a hash set: `meta.module-variables` and friends
    // return these keys, and a map whose order changes from run to run is not
    // something a stylesheet can rely on.
    IndexSet<Identifier>,
);

impl<V: fmt::Debug + Clone> MergedMapView<V> {
    pub fn new(maps: Vec<Arc<dyn MapView<Value = V>>>) -> Self {
        let unique_keys: IndexSet<Identifier> =
            maps.iter().fold(IndexSet::new(), |mut keys, map| {
                keys.extend(map.keys());
                keys
            });

        Self(maps, unique_keys)
    }
}

impl<V: fmt::Debug + Clone> MapView for MergedMapView<V> {
    type Value = V;
    fn get(&self, name: Identifier) -> Option<Self::Value> {
        self.0.iter().rev().find_map(|map| (*map).get(name))
    }

    fn remove(&self, _name: Identifier) -> Option<Self::Value> {
        unimplemented!()
    }

    fn len(&self) -> usize {
        self.1.len()
    }

    fn insert(&self, name: Identifier, value: Self::Value) -> Option<Self::Value> {
        for map in self.0.iter().rev() {
            if map.contains_key(name) {
                return map.insert(name, value);
            }
        }

        unreachable!("New entries may not be added to MergedMapView")
    }

    fn keys(&self) -> Vec<Identifier> {
        self.1.iter().copied().collect()
    }

    fn iter(&self) -> Vec<(Identifier, Self::Value)> {
        self.1
            .iter()
            .copied()
            .map(|name| (name, self.get(name).unwrap()))
            .collect()
    }

    fn identity(&self, name: Identifier) -> Option<usize> {
        self.0.iter().rev().find_map(|map| map.identity(name))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PublicMemberMapView<V: fmt::Debug + Clone, T: MapView<Value = V> + Clone>(pub T);

impl<V: fmt::Debug + Clone, T: MapView<Value = V> + Clone> MapView for PublicMemberMapView<V, T> {
    type Value = V;
    fn get(&self, name: Identifier) -> Option<Self::Value> {
        if !name.is_public() {
            return None;
        }

        self.0.get(name)
    }

    fn remove(&self, name: Identifier) -> Option<Self::Value> {
        if !name.is_public() {
            return None;
        }

        self.0.remove(name)
    }

    fn insert(&self, name: Identifier, value: Self::Value) -> Option<Self::Value> {
        if !name.is_public() {
            return None;
        }

        self.0.insert(name, value)
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn keys(&self) -> Vec<Identifier> {
        self.0
            .keys()
            .iter()
            .copied()
            .filter(Identifier::is_public)
            .collect()
    }

    fn iter(&self) -> Vec<(Identifier, Self::Value)> {
        self.0
            .iter()
            .into_iter()
            .filter(|(name, _)| Identifier::is_public(name))
            .collect()
    }

    fn identity(&self, name: Identifier) -> Option<usize> {
        if !name.is_public() {
            return None;
        }

        self.0.identity(name)
    }
}
