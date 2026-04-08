//! Horizontal and vertical stacks of UI nodes.
//!
//! [`HStack`] arranges children from left to right. [`VStack`] arranges children from top to
//! bottom. Both support inserting spacing between children and changing the order that children
//! appear in.
//!
//! ## Alignment caveats
//!
//! - The alignment of children in the stack axis (horizontal for [`HStack`], vertical for
//!   [`VStack`]) is ignored. All children are compressed to their minimum size in the stack axis.
//!
//! - The alignment of the `Stack` in the stack axis cannot be [`Full`]. The stack must compress as
//!   well as the children. `Full` creates extra space that the stack does not handle. If `Full` is
//!   used, it will have the same appearance as [`Begin`].
//!
//! The cross axis alignment of children and stacks works as expected.
//!
//! [`Full`]: Alignment::Full
//! [`Begin`]: Alignment::Begin

use indexmap::IndexSet;
use thunderdome::Index as TdIndex;

use crate::{Alignment, NodeCache, Rect, UiNode, UiTree};

/// A horizontal arrangement of UI nodes with configurable spacing.
///
/// Nodes can be reordered and added/removed at any time.
///
/// Nodes always appear at their minimum size horizontally, regardless of alignment. Vertical
/// alignment still applies.
///
/// [`Full`]: Alignment::Full
/// [`Begin`]: Alignment::Begin
pub struct HStack {
    align: (Alignment, Alignment),
    children: IndexSet<TdIndex>,
    pub spacing: f32,
}

impl HStack {
    pub fn new() -> Self {
        Self {
            align: (Alignment::Full, Alignment::Full),
            children: IndexSet::new(),
            spacing: 0.0,
        }
    }

    /// Add a new child to the stack.
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
    /// A horizontal alignment of [`Full`] will not function as expected, and will have the same
    /// appearance as [`Begin`].
    ///
    /// [`Full`]: Alignment::Full
    /// [`Begin`]: Alignment::Begin
    pub fn with_align(mut self, align: (Alignment, Alignment)) -> Self {
        self.align = align;
        self
    }

    /// Set the spacing between children. Defaults to 0.
    ///
    /// No spacing appears before the first child or after the last child.
    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
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

    /// Returns the tree index associated with a child at a given stack index.
    pub fn get_child_index(&self, index: usize) -> Option<TdIndex> {
        self.children.get_index(index).copied()
    }
}

impl Default for HStack {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode for HStack {
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
            w += cw + self.spacing;
            h = h.max(ch);
        }

        // Remove the extra spacing at the end
        (w - self.spacing, h)
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        let mut child_rects = Vec::with_capacity(self.children.len());

        let mut x = cache.rect.x;
        for child in &self.children {
            let child_min = tree.get_cache(*child).expect("Child not in cache").min_size;
            let child = tree.get_node(*child).expect("Child not in arena");

            let space = Rect::new(x, cache.rect.y, child_min.0, cache.rect.h)
                .align(child.get_align(), child_min);
            child_rects.push(space);
            x += child_min.0 + self.spacing;
        }

        child_rects
    }

    fn get_children(&self) -> Vec<TdIndex> {
        self.children.iter().copied().collect()
    }
}

/// A vertical arrangement of UI nodes with configurable spacing.
///
/// Nodes can be reordered and added/removed at any time.
///
/// Nodes always appear at their minimum size vertically, regardless of alignment. Horizontal
/// alignment still applies.
///
/// [`Full`]: Alignment::Full
/// [`Begin`]: Alignment::Begin
pub struct VStack {
    align: (Alignment, Alignment),
    children: IndexSet<TdIndex>,
    spacing: f32,
}

impl VStack {
    pub fn new() -> Self {
        Self {
            align: (Alignment::Full, Alignment::Full),
            children: IndexSet::new(),
            spacing: 0.0,
        }
    }

    /// Add a new child to the stack.
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

    /// Set the spacing between children. Defaults to 0.
    ///
    /// No spacing appears before the first child or after the last child.
    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set the spacing between children.
    ///
    /// See [`with_spacing`] for details.
    ///
    /// [`with_spacing`]: Self::with_spacing
    pub fn set_spacing(&mut self, spacing: f32) {
        self.spacing = spacing;
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

    /// Returns the tree index associated with a child at a given stack index.
    pub fn get_child_index(&self, index: usize) -> Option<TdIndex> {
        self.children.get_index(index).copied()
    }
}

impl Default for VStack {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode for VStack {
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
            h += ch + self.spacing;
        }

        // Remove the extra spacing at the end
        (w, h - self.spacing)
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        let mut child_rects = Vec::with_capacity(self.children.len());

        let mut y = cache.rect.y;
        for child in &self.children {
            let child_min = tree.get_cache(*child).expect("Child not in cache").min_size;
            let child = tree.get_node(*child).expect("Child not in arena");

            let space = Rect::new(cache.rect.x, y, cache.rect.w, child_min.1)
                .align(child.get_align(), child_min);
            child_rects.push(space);
            y += child_min.1 + self.spacing;
        }

        child_rects
    }

    fn get_children(&self) -> Vec<TdIndex> {
        self.children.iter().copied().collect()
    }
}
