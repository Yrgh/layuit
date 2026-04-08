//! Containers that distribute equal space to children.
//!
//! [`HEqual`] and [`VEqual`] work very similar to [`HStack`] and [`VStack`], but give every child
//! equal space and do not suffer from the [`Full`] alignment caveat.
//!
//! [`Grid`] arranges nodes in a grid, with each child getting equal width and height. Nodes fill
//! from left to right first, and then from top to bottom.
//!
//! [`HEqual`]: crate::grid::HEqual
//! [`VEqual`]: crate::grid::VEqual
//! [`HStack`]: crate::stacks::HStack
//! [`VStack`]: crate::stacks::VStack
//! [`Full`]: crate::Alignment::Full

use indexmap::IndexSet;
use std::num::NonZero;
use thunderdome::Index as TdIndex;

use crate::{Alignment, NodeCache, Rect, UiNode, UiTree};

/// Arranges children from left to right, similar to [`HStack`], but gives every child equal space
/// and does not suffer from the [`Full`] alignment caveat.
///
/// [`HStack`]: crate::stacks::HStack
/// [`Full`]: crate::Alignment::Full
pub struct HEqual {
    align: (Alignment, Alignment),
    children: IndexSet<TdIndex>,
}

impl HEqual {
    /// Creates a new `HEqual` with no children, 0 spacing, and ([`Begin`], [`Begin`]) alignment.
    ///
    /// [`Begin`]: Alignment::Begin
    pub fn new() -> Self {
        Self {
            align: (Alignment::Begin, Alignment::Begin),
            children: IndexSet::new(),
        }
    }

    /// Add a new child to the list.
    ///
    /// See [`add_child`].
    ///
    /// [`add_child`]: Self::add_child
    pub fn with_child(mut self, child: impl UiNode, tree: &mut UiTree) -> Self {
        self.add_child(child, tree);
        self
    }

    /// Set the horizontal and vertical alignment.
    pub fn with_align(mut self, align: (Alignment, Alignment)) -> Self {
        self.align = align;
        self
    }

    /// Add a child to the list. The child will appear at the end.
    pub fn add_child(&mut self, child: impl UiNode, tree: &mut UiTree) {
        let index = tree.add_node(child);
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

    /// Returns the tree index associated with a child at a given list index.
    pub fn get_child_index(&self, index: usize) -> Option<TdIndex> {
        self.children.get_index(index).copied()
    }
}

impl Default for HEqual {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode for HEqual {
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

        (w * (self.len() as f32), h)
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        if self.is_empty() {
            return vec![];
        }

        let mut child_rects = Vec::with_capacity(self.children.len());

        let w = cache.rect.w / (self.len() as f32);

        let mut x = cache.rect.x;
        for child in &self.children {
            let child_min = tree.get_cache(*child).expect("Child not in cache").min_size;
            let child = tree.get_node(*child).expect("Child not in arena");

            let space =
                Rect::new(x, cache.rect.y, w, cache.rect.h).align(child.get_align(), child_min);
            child_rects.push(space);
            x += w;
        }

        child_rects
    }

    fn get_children(&self) -> Vec<TdIndex> {
        self.children.iter().copied().collect()
    }
}

/// Arranges children from top to bottom, similar to [`VStack`], but gives every child equal space
/// and does not suffer from the [`Full`] alignment caveat.
///
/// [`VStack`]: crate::stacks::VStack
/// [`Full`]: crate::Alignment::Full
pub struct VEqual {
    align: (Alignment, Alignment),
    children: IndexSet<TdIndex>,
}

impl VEqual {
    /// Creates a new `VEqual` with no children, 0 spacing, and ([`Begin`], [`Begin`]) alignment.
    ///
    /// [`Begin`]: Alignment::Begin
    pub fn new() -> Self {
        Self {
            align: (Alignment::Begin, Alignment::Begin),
            children: IndexSet::new(),
        }
    }

    /// Add a new child to the list.
    ///
    /// See [`add_child`].
    ///
    /// [`add_child`]: Self::add_child
    pub fn with_child(mut self, child: impl UiNode, tree: &mut UiTree) -> Self {
        self.add_child(child, tree);
        self
    }

    /// Set the horizontal and vertical alignment.
    pub fn with_align(mut self, align: (Alignment, Alignment)) -> Self {
        self.align = align;
        self
    }

    /// Add a child to the list. The child will appear at the end.
    pub fn add_child(&mut self, child: impl UiNode, tree: &mut UiTree) {
        let index = tree.add_node(child);
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

    /// Returns the tree index associated with a child at a given list index.
    pub fn get_child_index(&self, index: usize) -> Option<TdIndex> {
        self.children.get_index(index).copied()
    }
}

impl Default for VEqual {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode for VEqual {
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

        (w, h * (self.len() as f32))
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        if self.is_empty() {
            return vec![];
        }

        let mut child_rects = Vec::with_capacity(self.children.len());

        let h = cache.rect.h / (self.len() as f32);

        let mut y = cache.rect.y;
        for child in &self.children {
            let child_min = tree.get_cache(*child).expect("Child not in cache").min_size;
            let child = tree.get_node(*child).expect("Child not in arena");

            let space =
                Rect::new(cache.rect.x, y, cache.rect.w, h).align(child.get_align(), child_min);
            child_rects.push(space);
            y += h;
        }

        child_rects
    }

    fn get_children(&self) -> Vec<TdIndex> {
        self.children.iter().copied().collect()
    }
}

/// Arranges children in a grid of equally-sized cells, from left to right and then top to bottom.
pub struct Grid {
    pub num_cols: NonZero<usize>,

    align: (Alignment, Alignment),
    children: IndexSet<TdIndex>,
}

impl Grid {
    /// Create a new grid with the specified number of columns.
    pub fn new(num_cols: NonZero<usize>) -> Self {
        Self {
            num_cols,
            align: (Alignment::Full, Alignment::Full),
            children: IndexSet::new(),
        }
    }

    /// Add a new child to the grid.
    ///
    /// See [`add_child`].
    ///
    /// [`add_child`]: Self::add_child
    pub fn with_child(mut self, child: impl UiNode, tree: &mut UiTree) -> Self {
        self.add_child(child, tree);
        self
    }

    /// Set the horizontal and vertical alignment.
    pub fn with_align(mut self, align: (Alignment, Alignment)) -> Self {
        self.align = align;
        self
    }

    /// Add a child to the grid. The child will appear at the end.
    pub fn add_child(&mut self, child: impl UiNode, tree: &mut UiTree) {
        let index = tree.add_node(child);
        self.children.insert(index);
    }

    /// Returns the number of children in the grid.
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Returns `true` if the grid is empty.
    ///
    /// Equivalent to `len() == 0`.
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Remove a child from the grid.
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

    /// Returns the tree index associated with a child at a given list index.
    pub fn get_child_index(&self, index: usize) -> Option<TdIndex> {
        self.children.get_index(index).copied()
    }
}

impl UiNode for Grid {
    fn get_align(&self) -> (Alignment, Alignment) {
        self.align
    }

    fn get_align_mut(&mut self) -> (&mut Alignment, &mut Alignment) {
        (&mut self.align.0, &mut self.align.1)
    }

    fn calculate_min_size(&self, tree: &UiTree) -> (f32, f32) {
        let mut w = 0.0f32;
        let mut h = 0.0f32;
        for child in &self.children {
            let (cw, ch) = tree.get_cache(*child).expect("Child not in cache").min_size;
            w = w.max(cw);
            h = h.max(ch);
        }

        let num_rows = self.len().div_ceil(self.num_cols.get());

        (w * self.num_cols.get() as f32, h * num_rows as f32)
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        let mut child_rects = Vec::with_capacity(self.children.len());

        let num_rows = self.len().div_ceil(self.num_cols.get());
        let dx = cache.rect.w / (self.num_cols.get() as f32);
        let dy = cache.rect.h / (num_rows as f32);

        let mut col = 0;
        let mut x = cache.rect.x;
        let mut y = cache.rect.y;
        for child in &self.children {
            let child_min = tree.get_cache(*child).expect("Child not in cache").min_size;
            let child = tree.get_node(*child).expect("Child not in arena");

            let space = Rect::new(x, y, dx, dy).align(child.get_align(), child_min);
            child_rects.push(space);

            col += 1;
            if col >= self.num_cols.get() {
                col = 0;
                x = cache.rect.x;
                y += dy;
            } else {
                x += dx;
            }
        }

        child_rects
    }

    fn get_children(&self) -> Vec<TdIndex> {
        self.children.iter().copied().collect()
    }
}
