use std::{
    cell::{Ref, RefCell, RefMut},
    collections::BTreeMap,
};

use crate::ast::CssStmt;

#[derive(Debug, Clone)]
pub(super) struct CssTree {
    // None is tombstone
    stmts: Vec<RefCell<Option<CssStmt>>>,
    pub parent_to_child: BTreeMap<CssTreeIdx, Vec<CssTreeIdx>>,
    pub child_to_parent: BTreeMap<CssTreeIdx, CssTreeIdx>,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
#[repr(transparent)]
pub(super) struct CssTreeIdx(usize);

impl CssTree {
    pub const ROOT: CssTreeIdx = CssTreeIdx(0);

    pub fn new() -> Self {
        let mut tree = Self {
            stmts: Vec::new(),
            parent_to_child: BTreeMap::new(),
            child_to_parent: BTreeMap::new(),
        };

        tree.stmts.push(RefCell::new(None));

        tree
    }

    pub fn get(&self, idx: CssTreeIdx) -> Ref<'_, Option<CssStmt>> {
        self.stmts[idx.0].borrow()
    }

    /// The number of statements added so far, used to mark where a module's
    /// CSS begins.
    pub fn stmt_count(&self) -> usize {
        self.stmts.len()
    }

    /// The statements added since `start` whose parent was already in the
    /// tree at that point -- the roots of everything one module execution
    /// emitted, without the nested children those roots already own.
    pub fn top_level_stmts_since(&self, start: usize) -> Vec<CssTreeIdx> {
        (start..self.stmts.len())
            .map(CssTreeIdx)
            .filter(|idx| match self.child_to_parent.get(idx) {
                Some(parent) => parent.0 < start,
                None => true,
            })
            .collect()
    }

    pub fn get_mut(&self, idx: CssTreeIdx) -> RefMut<'_, Option<CssStmt>> {
        self.stmts[idx.0].borrow_mut()
    }

    pub fn finish(self) -> Vec<CssStmt> {
        let mut idx = 1;

        while idx < self.stmts.len() - 1 {
            if self.stmts[idx].borrow().is_none() || !self.has_children(CssTreeIdx(idx)) {
                idx += 1;
                continue;
            }

            self.apply_children(CssTreeIdx(idx));

            idx += 1;
        }

        self.stmts
            .into_iter()
            .filter_map(RefCell::into_inner)
            .collect()
    }

    fn apply_children(&self, parent: CssTreeIdx) {
        for &child in &self.parent_to_child[&parent] {
            if self.has_children(child) {
                self.apply_children(child);
            }

            match self.stmts[child.0].borrow_mut().take() {
                Some(child) => self.add_child_to_parent(child, parent),
                None => continue,
            };
        }
    }

    fn has_children(&self, parent: CssTreeIdx) -> bool {
        self.parent_to_child.contains_key(&parent)
    }

    fn add_child_to_parent(&self, child: CssStmt, parent_idx: CssTreeIdx) {
        RefMut::map(self.stmts[parent_idx.0].borrow_mut(), |parent| {
            match parent {
                Some(CssStmt::RuleSet { body, .. }) => body.push(child),
                Some(CssStmt::Style(..) | CssStmt::Comment(..) | CssStmt::Import(..)) | None => {
                    unreachable!()
                }
                Some(CssStmt::Media(media, ..)) => {
                    media.body.push(child);
                }
                Some(CssStmt::UnknownAtRule(at_rule, ..)) => {
                    at_rule.body.push(child);
                }
                Some(CssStmt::Supports(supports, ..)) => {
                    supports.body.push(child);
                }
                Some(CssStmt::KeyframesRuleSet(keyframes)) => {
                    keyframes.body.push(child);
                }
            };
            parent
        });
    }

    pub fn add_child(&mut self, child: CssStmt, parent_idx: CssTreeIdx) -> CssTreeIdx {
        let child_idx = self.add_stmt_inner(child);
        self.parent_to_child
            .entry(parent_idx)
            .or_default()
            .push(child_idx);
        self.child_to_parent.insert(child_idx, parent_idx);
        child_idx
    }

    /// Moves `child_idx` out of the parent it currently has and appends it to
    /// `parent_idx`.
    ///
    /// This is how a plain CSS rule that a module emitted at the top level gets
    /// nested back under the rule the module was loaded from, when its selector
    /// says it belongs there rather than merged into it.
    pub fn reparent(&mut self, child_idx: CssTreeIdx, parent_idx: CssTreeIdx) {
        if let Some(old_parent) = self.child_to_parent.get(&child_idx).copied()
            && let Some(siblings) = self.parent_to_child.get_mut(&old_parent)
        {
            siblings.retain(|&sibling| sibling != child_idx);
        }

        self.link_child_to_parent(child_idx, parent_idx);
    }

    pub fn link_child_to_parent(&mut self, child_idx: CssTreeIdx, parent_idx: CssTreeIdx) {
        self.parent_to_child
            .entry(parent_idx)
            .or_default()
            .push(child_idx);
        self.child_to_parent.insert(child_idx, parent_idx);
    }

    pub fn has_following_sibling(&self, child: CssTreeIdx) -> bool {
        if child == Self::ROOT {
            return false;
        }

        let parent_idx = self.child_to_parent.get(&child).unwrap();

        let parent_children = self.parent_to_child.get(parent_idx).unwrap();

        // todo: we shouldn't take into account children that are invisible
        parent_children.last() != Some(&child)
    }

    pub fn add_stmt(&mut self, child: CssStmt, parent: Option<CssTreeIdx>) -> CssTreeIdx {
        match parent {
            Some(parent) => self.add_child(child, parent),
            None => self.add_child(child, Self::ROOT),
        }
    }

    fn add_stmt_inner(&mut self, stmt: CssStmt) -> CssTreeIdx {
        let idx = CssTreeIdx(self.stmts.len());
        self.stmts.push(RefCell::new(Some(stmt)));

        idx
    }
}
