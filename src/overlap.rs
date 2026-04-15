//! Containers of independent children
//!
//! [`Overlap`] is a container that arranges its children independently on top of each other. Each
//! child is aligned to the space of the entire container and no restrictions are applied.

use crate::{Alignment, NodeCache, NodeIndex, OwnedIndex, Rect, UiNode, UiTree};
use indexmap::IndexSet;

/// A node that contains many children which are stacked on top of each other and do not interact.
pub struct Overlap {
    align: (Alignment, Alignment),
    children: IndexSet<OwnedIndex>,
}

impl Overlap {
    /// Create an empty `Overlap` with no child and default alignment.
    pub fn new() -> Self {
        Self {
            align: Default::default(),
            children: IndexSet::new(),
        }
    }

    /// Add a new child to the list.
    pub fn with_child(mut self, index: OwnedIndex) -> Self {
        self.children.insert(index);
        self
    }

    /// Set the horizontal and vertical alignment of the container, not the children.
    pub fn with_align(mut self, align: (Alignment, Alignment)) -> Self {
        self.align = align;
        self
    }

    /// Add a child to the list. The child will appear on top (last-visited).
    pub fn add_child(&mut self, index: OwnedIndex) {
        self.children.insert(index);
    }

    /// Returns the number of children in the list.
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Returns `true` if the list is empty.
    ///
    /// Equivalent to `len() == 0`.
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Remove a child from the list.
    ///
    /// Returns `true` if the child was removed.
    pub fn remove_child(&mut self, index: usize, tree: &mut UiTree) -> bool {
        let Some(ti) = self.children.shift_remove_index(index) else {
            return false;
        };

        if tree.get_node(ti.shareable()).is_none() {
            return false;
        }
        tree.remove_node(ti);

        true
    }

    /// Move a child to a new index. Lower indices are visited first.
    ///
    /// Returns `true` if the child was moved.
    pub fn set_child_position(&mut self, index: usize, position: usize) -> bool {
        self.children.move_index(index, position);
        true
    }

    /// Returns the tree index associated with a child at a given list index.
    pub fn get_child_index(&self, index: usize) -> Option<NodeIndex> {
        self.children.get_index(index).map(OwnedIndex::shareable)
    }
}

impl Default for Overlap {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode for Overlap {
    fn get_align(&self) -> (Alignment, Alignment) {
        self.align
    }

    fn get_align_mut(&mut self) -> (&mut Alignment, &mut Alignment) {
        (&mut self.align.0, &mut self.align.1)
    }

    fn calculate_min_size(&self, tree: &UiTree) -> (f32, f32) {
        if self.children.is_empty() {
            return (0.0, 0.0);
        }

        let mut w = 0.0f32;
        let mut h = 0.0f32;
        for child in &self.children {
            let child = tree
                .get_cache(child.shareable())
                .expect("Child not in cache");
            let (cw, ch) = child.min_size;
            w = w.max(cw);
            h = h.max(ch);
        }

        (w, h)
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        let mut child_rects = Vec::with_capacity(self.children.len());

        for child in &self.children {
            let child = child.shareable();

            let child_min = tree.get_cache(child).expect("Child not in cache").min_size;
            let child = tree.get_node(child).expect("Child not in arena");

            let space = cache.rect.align(child.get_align(), child_min);
            child_rects.push(space);
        }

        child_rects
    }
}
