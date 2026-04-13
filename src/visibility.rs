//! Container nodes that control the visibility of their child.
//!
//! An invisible child has 0 minimum size and is not visited by [`UiTree::walk_node`] with
//! `use_visible` set to `true`.
//!
//! [`Hider`] allows a child to be hidden manually.

use indexmap::IndexSet;

use thunderdome::Index as NodeIndex;

use crate::{Alignment, NodeCache, Rect, UiNode, UiTree};

/// A node that optionally hides its child.
///
/// Hiding the child makes both the child have a width and height of 0 and makes the hider have 0
/// minimum size. It also prevents the child from being visited by [`UiTree::walk_node`] with
/// `use_visible` set to `true`, so that renderers don't attempt to render it, even if it has 0
/// size.
pub struct Hider {
    pub hidden: bool,

    align: (Alignment, Alignment),
    child: Option<NodeIndex>,
}

impl Hider {
    /// Creates a new `Hider` with no child, no alignment, and shown.
    pub fn new() -> Self {
        Self {
            hidden: false,
            align: (Alignment::Begin, Alignment::Begin),
            child: None,
        }
    }

    /// Set the horizontal and vertical alignment.
    pub fn with_align(mut self, align: (Alignment, Alignment)) -> Self {
        self.align = align;
        self
    }

    /// Bind a child node to the hider.
    ///
    /// # Panics
    /// If there is already a child node.
    pub fn with_child(mut self, index: NodeIndex) -> Self {
        assert!(self.child.is_none());
        self.child = Some(index);
        self
    }

    /// Set whether the child is hidden.
    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    /// Bind a child node to the node.
    ///
    /// # Panics
    /// If there is already a child node.
    pub fn add_child(&mut self, index: NodeIndex) {
        assert!(self.child.is_none());
        self.child = Some(index);
    }
}

impl Default for Hider {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode for Hider {
    fn get_align(&self) -> (Alignment, Alignment) {
        self.align
    }

    fn get_align_mut(&mut self) -> (&mut Alignment, &mut Alignment) {
        (&mut self.align.0, &mut self.align.1)
    }

    fn calculate_min_size(&self, tree: &UiTree) -> (f32, f32) {
        if !self.hidden
            && let Some(child) = self.child
        {
            tree.get_cache(child).expect("Child not in cache").min_size
        } else {
            (0.0, 0.0)
        }
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        if !self.hidden
            && let Some(child) = self.child
        {
            let child_min = tree.get_cache(child).expect("Child not in cache").min_size;
            let child = tree.get_node(child).expect("Child not in cache");
            let space = cache.rect.align(child.get_align(), child_min);
            vec![space]
        } else {
            vec![]
        }
    }

    fn get_children(&self) -> Vec<NodeIndex> {
        self.child.into_iter().collect()
    }

    fn get_visible_children(&self) -> Vec<NodeIndex> {
        if !self.hidden
            && let Some(child) = self.child
        {
            vec![child]
        } else {
            vec![]
        }
    }
}

/// Shows at most **one** of its children.
///
/// You can change which node is selected as visible, or set no child to be visible. If you wish
/// to have multiple children visible at once, wrap all of them in [`Hider`]s and your choice of
/// container.
///
/// The minimum size of the selector is the minimum size of the selected child, or 0 if none is
/// selected.
pub struct Selector {
    selected: Option<NodeIndex>,

    align: (Alignment, Alignment),
    children: IndexSet<NodeIndex>,
}

impl Selector {
    /// Create an empty `Selector` with no children, no selection, and alignment ([`Begin`],
    /// [`Begin`]).
    ///
    /// [`Begin`]: Alignment::Begin
    pub fn new() -> Self {
        Self {
            selected: None,
            align: (Alignment::Begin, Alignment::Begin),
            children: IndexSet::new(),
        }
    }

    /// Add a new child to the list. If there are no other children, the new child will be selected.
    pub fn with_child(mut self, index: NodeIndex) -> Self {
        if self.selected.is_none() {
            self.selected = Some(index);
        }
        self.children.insert(index);
        self
    }

    /// Set the horizontal and vertical alignment of the container, not the children.
    pub fn with_align(mut self, align: (Alignment, Alignment)) -> Self {
        self.align = align;
        self
    }

    /// Add a child to the list.
    pub fn add_child(&mut self, index: NodeIndex) {
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

    /// Remove a child from the list. If the child is currently selected, the selection is removed.
    ///
    /// Returns `true` if the child was removed.
    pub fn remove_child(&mut self, index: usize, tree: &mut UiTree) -> bool {
        let Some(ti) = self.children.shift_remove_index(index) else {
            return false;
        };

        if let Some(selected) = self.selected
            && ti == selected
        {
            self.selected = None;
        }

        if tree.get_node(ti).is_none() {
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
        self.children.get_index(index).copied()
    }

    /// Returns the tree index of the currently-selected node.
    pub fn get_selected(&self) -> Option<NodeIndex> {
        self.selected
    }

    /// Returns the list index of the currently-selected node.
    pub fn get_selected_index(&self) -> Option<usize> {
        self.children.get_index_of(&self.selected?)
    }

    /// Sets the selected node to the given tree index.
    pub fn set_selected(&mut self, child: NodeIndex) {
        self.selected = Some(child)
    }

    /// Sets the selected node to the given list index.
    pub fn set_selected_index(&mut self, index: usize) {
        self.selected = self.children.get_index(index).copied()
    }

    /// Removes the selection.
    pub fn unselect(&mut self) {
        self.selected = None;
    }
}

impl Default for Selector {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode for Selector {
    fn get_align(&self) -> (Alignment, Alignment) {
        self.align
    }

    fn get_align_mut(&mut self) -> (&mut Alignment, &mut Alignment) {
        (&mut self.align.0, &mut self.align.1)
    }

    fn calculate_min_size(&self, tree: &UiTree) -> (f32, f32) {
        if let Some(index) = self.selected {
            tree.get_cache(index).expect("Child not in cache").min_size
        } else {
            (0.0, 0.0)
        }
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        if let Some(index) = self.selected {
            let child_min = tree.get_cache(index).expect("Child not in cache").min_size;
            let child = tree.get_node(index).expect("Child not in cache");

            vec![cache.rect.align(child.get_align(), child_min)]
        } else {
            vec![]
        }
    }

    fn get_children(&self) -> Vec<NodeIndex> {
        self.children.iter().copied().collect()
    }

    fn get_visible_children(&self) -> Vec<NodeIndex> {
        self.selected.iter().copied().collect()
    }
}
