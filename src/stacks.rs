use indexmap::IndexSet;
use thunderdome::Index as TdIndex;

use crate::{Alignment, NodeCache, Rect, UiNode, UiTree};

/// A horizontal arrangement of UI nodes with configurable spacing.
///
/// Nodes can be reordered and add/removed at any time.
///
/// Nodes always appear at their minimum size horizontally, regardless of alignment. Vertical
/// alignment still applies.
/// 
/// [`Full`]: Alignment::Full
/// [`Begin`]: Alignment::Begin
pub struct HStack {
    align: (Alignment, Alignment),
    children: IndexSet<TdIndex>,
    spacing: f32,
}

impl HStack {
    pub fn new() -> Self {
        Self {
            align: (Alignment::Full, Alignment::Full),
            children: IndexSet::new(),
            spacing: 0.0,
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
    /// The horizontal alignment cannot be [`Full`], as there would be extra space unassigned to
    /// children.
    ///
    /// # Panics
    /// If `align.0` is [`Full`].
    ///
    /// [`Full`]: Alignment::Full
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

    /// Set the horizontal and vertical alignment.
    ///
    /// See [`with_align`] for details.
    /// 
    /// # Panics
    /// If `align.0` is [`Full`].
    ///
    /// [`with_align`]: Self::with_align
    /// [`Full`]: Alignment::Full
    pub fn set_align(&mut self, align: (Alignment, Alignment)) {
        if align.0 == Alignment::Full {
            panic!("Alignment::Full would allow extra space that is unaccounted for");
        }
        self.align = align;
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
        tree.arena.remove(ti);
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

impl Default for HStack {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode for HStack {
    fn get_align(&self) -> (Alignment, Alignment) {
        self.align
    }

    fn get_align_mut(&mut self) -> &mut (Alignment, Alignment) {
        &mut self.align
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

        let (_, h) = cache.min_size;
        let mut x = cache.rect.x;
        for child in &self.children {
            let child_min = tree.get_cache(*child).expect("Child not in cache").min_size;
            let child = tree.arena.get(*child).expect("Child not in arena").as_ref();
            let space =
                Rect::new(x, cache.rect.y, child_min.0, h).align(child.get_align(), cache.min_size);
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
/// Nodes can be reordered and add/removed at any time.
///
/// Nodes always appear at their minimum size vertically, regardless of alignment. Vertical
/// alignment still applies.
/// 
/// If the vertical alignment is [`Full`], the appearance will be identical to that of [`Begin`].
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
    /// The vertical alignment cannot be [`Full`], as there would be extra space unassigned to
    /// children.
    ///
    /// # Panics
    /// If `align.1` is [`Full`].
    ///
    /// [`Full`]: Alignment::Full
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

    /// Set the horizontal and vertical alignment.
    ///
    /// See [`with_align`] for details.
    /// 
    /// # Panics
    /// If `align.1` is [`Full`].
    ///
    /// [`with_align`]: Self::with_align
    pub fn set_align(&mut self, align: (Alignment, Alignment)) {
        if align.1 == Alignment::Full {
            panic!("Alignment::Full would allow extra space that is unaccounted for");
        }
        self.align = align;
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
        tree.arena.remove(ti);
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

impl Default for VStack {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode for VStack {
    fn get_align(&self) -> (Alignment, Alignment) {
        self.align
    }

    fn get_align_mut(&mut self) -> &mut (Alignment, Alignment) {
        &mut self.align
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

        let (w, _) = cache.min_size;
        let mut y = cache.rect.y;
        for child in &self.children {
            let child_min = tree.get_cache(*child).expect("Child not in cache").min_size;
            let child = tree.get_node(*child).expect("Child not in arena");

            let space =
                Rect::new(cache.rect.x, y, w, child_min.1).align(child.get_align(), cache.min_size);
            child_rects.push(space);
            y += child_min.1 + self.spacing;
        }

        child_rects
    }

    fn get_children(&self) -> Vec<TdIndex> {
        self.children.iter().copied().collect()
    }
}
