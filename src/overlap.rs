//! Containers of independent children
//! 
//! [`Overlap`] is a container that arranges its children independently on top of each other. Each
//! child aligns to the entire size of the container and no restriction is applied.

use indexmap::IndexSet;
use thunderdome::Index as TdIndex;

use crate::{Alignment, NodeCache, Rect, UiNode, UiTree};

/// A node that contains many children which are stacked on top of each other and do not interact.
pub struct Overlap {
    align: (Alignment, Alignment),
    children: IndexSet<TdIndex>,
}

impl Overlap {
    /// Create an empty `Overlap` with no child and alignment ([`Begin`], [`Begin`]).
    /// 
    /// [`Begin`]: Alignment::Begin
    pub fn new() -> Self {
        Self {
            align: (Alignment::Begin, Alignment::Begin),
            children: IndexSet::new(),
        }
    }

    /// Add a child to the stack, at the end.
    ///
    /// See [`add_child`].
    ///
    /// [`add_child`]: Self::add_child
    pub fn with_child(mut self, child: impl UiNode, tree: &mut UiTree) -> Self {
        self.add_child(child, tree);
        self
    }

    /// Set the horizontal and vertical alignment.
    ///
    /// A vertical alignment of [`Full`] will not function as expected, and will have the same
    /// appearance as [`Begin`].
    ///
    /// [`Full`]: Alignment::Full
    /// [`Begin`]: Alignment::Begin
    pub fn with_align(mut self, align: (Alignment, Alignment)) -> Self {
        self.align = align;
        self
    }

    /// Add a child to the stack. The child will appear at the end.
    pub fn add_child(&mut self, child: impl UiNode, tree: &mut UiTree) {
        let index = tree.add_node(child);
        self.children.insert(index);
    }

    /// Returns the number of children in the stack.
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Returns `true` if the stack is empty.
    /// 
    /// Equivalent to `len() == 0`.
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Remove a child from the stack.
    ///
    /// Returns `true` if the child was removed.
    pub fn remove_child(&mut self, index: usize, tree: &mut UiTree) -> bool {
        let Some(ti) = self.children.shift_remove_index(index) else {
            return false;
        };

        if tree.get_node(ti).is_none() {
            return false;
        }
        tree.remove_node(ti);

        true
    }

    /// Move a child to a new index.
    ///
    /// Returns `true` if the child was moved.
    pub fn set_child_position(&mut self, index: usize, position: usize) -> bool {
        self.children.move_index(index, position);
        true
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
            let child = tree.get_cache(*child).expect("Child not in cache");
            let (cw, ch) = child.min_size;
            w = w.max(cw);
            h = h.max(ch);
        }

        // Remove the extra spacing at the end
        (w, h)
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        let mut child_rects = Vec::with_capacity(self.children.len());

        for child in &self.children {
            let child_min = tree.get_cache(*child).expect("Child not in cache").min_size;
            let child = tree.get_node(*child).expect("Child not in arena");

            let space = cache.rect.align(child.get_align(), child_min);
            child_rects.push(space);
        }

        child_rects
    }
}