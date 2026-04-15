//! Size constraint nodes.
//!
//! [`Margin`] wraps a single child in a runtime-configurable margin.
//!
//! [`Minimum`] wraps a single child with a configurable minimum size, allowing for the child to be
//! precisely aligned or given a fixed size. It can also be used without a child to create an empty,
//! fixed-size space.
//!
//! ## Alignment caveats with [`Margin`]
//!
//! When using [`Center`] on the child, the child will be aligned to the center of the space *after*
//! the restriction from the margin is applied. All other alignments hug the edges, giving the
//! correct appearances.
//!
//! To avoid the alignment shift while keeping centering, have symmetrical margins or use
//! [`Minimum`] with the intended area.
//! 
//! ## Negative margins
//! 
//! [`Margin`] supports negative values for all of its margins. A negative value causes the node to
//! grow its minimum size by the absolute value, but does no shrinking. This can be used to make a
//! node additively larger than its minimum size, rather than statically.
//! 
//! ```rust
//! use layuit::padding::{Margin, Spacer};
//! use layuit::stack::VStack;
//! use layuit::Rect;
//! 
//! let mut tree = layuit::ui!(
//!     %%,
//!     +|+ VStack::new() => [
//!         -|- Margin::new().with_equal(-4.0) => [
//!             +|+ Spacer::sized((10.0, 10.0))
//!         ],
//!         -|- Margin::new().with_equal(4.0) => [
//!             +|+ Spacer::sized((10.0, 10.0))
//!         ],
//!     ]
//! );
//! 
//! tree.calculate_layout(Rect::new(0.0, 0.0, 20.0, 40.0));
//! 
//! // Both margins have size 18.0, but only the first spacer grows to size 18.0
//! ```
//!
//! [`Center`]: Alignment::Center

use crate::{Alignment, NodeCache, NodeIndex, OwnedIndex, Rect, UiNode, UiTree};

/// Maintains a margin around a singular child.
///
/// Using a negative margin will expand the minimum size but not restrict the space of the child.
/// This can be used to make the child occupy more space than it would otherwise.
///
/// The space is restricted by the margin before alignment is applied, so if the child's alignment
/// is [`Center`] the child will be centered based on the restricted space, resulting in an
/// unexpected shift if `top != bottom` or `left != right`. All other alignments are unaffected.
///
/// Maintains the margin as a minimum size even if `child` is `None`, however, once a child is
/// assigned, it cannot be removed. If you intend to use the margin as a minimum size, you should
/// use [`Minimum`] with no child instead.
///
/// [`Center`]: Alignment::Center
pub struct Margin {
    /// The left margin amount.
    pub left: f32,
    /// The right margin amount.
    pub right: f32,
    /// The top margin amount.
    pub top: f32,
    /// The bottom margin amount.
    pub bottom: f32,

    align: (Alignment, Alignment),
    child: Option<OwnedIndex>,
}

impl Margin {
    /// Creates a new `Margin` with no child, no margin, and default alignment.
    pub fn new() -> Self {
        Self {
            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
            align: Default::default(),
            child: None,
        }
    }

    /// Bind a child node.
    ///
    /// # Panics
    /// If there is already a child node.
    pub fn with_child(mut self, index: OwnedIndex) -> Self {
        assert!(self.child.is_none());
        self.child = Some(index);
        self
    }

    /// Set the horizontal and vertical alignment.
    pub fn with_align(mut self, align: (Alignment, Alignment)) -> Self {
        self.align = align;
        self
    }

    /// Set the left, right, top, and bottom margins.
    pub fn with_margins(mut self, left: f32, right: f32, top: f32, bottom: f32) -> Self {
        self.left = left;
        self.right = right;
        self.top = top;
        self.bottom = bottom;
        self
    }

    /// Set the left and right margins to the same value.
    pub fn with_equal_x(mut self, lr: f32) -> Self {
        self.left = lr;
        self.right = lr;
        self
    }

    /// Set the top and bottom margins to the same value.
    pub fn with_equal_y(mut self, tb: f32) -> Self {
        self.top = tb;
        self.bottom = tb;
        self
    }

    /// Set both the left and right margins, and both the top and bottom margins.
    pub fn with_equal_xy(mut self, lr: f32, tb: f32) -> Self {
        self.left = lr;
        self.right = lr;
        self.top = tb;
        self.bottom = tb;
        self
    }

    /// Set the left, right, top, and bottom margins to the same value.
    pub fn with_equal(mut self, margin: f32) -> Self {
        self.left = margin;
        self.right = margin;
        self.top = margin;
        self.bottom = margin;
        self
    }

    /// Set the left margin.
    pub fn with_left(mut self, left: f32) -> Self {
        self.left = left;
        self
    }

    /// Set the right margin.
    pub fn with_right(mut self, right: f32) -> Self {
        self.right = right;
        self
    }

    /// Set the top margin.
    pub fn with_top(mut self, top: f32) -> Self {
        self.top = top;
        self
    }

    /// Set the bottom margin.
    pub fn with_bottom(mut self, bottom: f32) -> Self {
        self.bottom = bottom;
        self
    }

    /// Get the tree index of the child.
    pub fn get_child(&self) -> Option<NodeIndex> {
        self.child.as_ref().map(OwnedIndex::shareable)
    }
}

impl Default for Margin {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode for Margin {
    fn get_align(&self) -> (Alignment, Alignment) {
        self.align
    }

    fn get_align_mut(&mut self) -> (&mut Alignment, &mut Alignment) {
        (&mut self.align.0, &mut self.align.1)
    }

    fn calculate_min_size(&self, tree: &UiTree) -> (f32, f32) {
        let left = self.left.abs();
        let right = self.right.abs();
        let top = self.top.abs();
        let bottom = self.bottom.abs();
        if let Some(child) = &self.child {
            let child = tree
                .get_cache(child.shareable())
                .expect("Child not in cache");
            let (w, h) = child.min_size;
            (w + left + right, h + top + bottom)
        } else {
            (left + right, top + bottom)
        }
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        let Some(child_idx) = self.child.as_ref() else {
            return vec![];
        };

        let child_idx = child_idx.shareable();

        let left = self.left.max(0.0);
        let right = self.right.max(0.0);
        let top = self.top.max(0.0);
        let bottom = self.bottom.max(0.0);

        let child_min = tree
            .get_cache(child_idx)
            .expect("Child not in cache")
            .min_size;
        let child = tree.get_node(child_idx).expect("Child not in cache");

        let space = Rect::new(
            cache.rect.x + left,
            cache.rect.y + top,
            cache.rect.w - left - right,
            cache.rect.h - top - bottom,
        )
        .align(child.get_align(), child_min);

        vec![space]
    }

    fn get_children(&self) -> Vec<NodeIndex> {
        self.child.iter().map(OwnedIndex::shareable).collect()
    }
}

/// Maintains an additional constraint to minimum size.
///
/// Maintains a minimum size even if `child` is `None`, however, once a child is
/// assigned, it cannot be removed.
pub struct Minimum {
    /// The minimum size to maintain.
    pub min_override: (f32, f32),

    child: Option<OwnedIndex>,
    align: (Alignment, Alignment),
}

impl Minimum {
    /// Creates a new `Minimum` with no child, no minimum override, and default alignment.
    pub fn new() -> Self {
        Self {
            min_override: (0.0, 0.0),
            child: None,
            align: Default::default(),
        }
    }

    /// Bind a child node to the node.
    ///
    /// # Panics
    /// If there is already a child node.
    pub fn with_child(mut self, index: OwnedIndex) -> Self {
        assert!(self.child.is_none());
        self.child = Some(index);
        self
    }

    /// Set the horizontal and vertical alignment.
    pub fn with_align(mut self, align: (Alignment, Alignment)) -> Self {
        self.align = align;
        self
    }

    /// Set the minimum size.
    pub fn with_min(mut self, min: (f32, f32)) -> Self {
        self.min_override = min;
        self
    }

    /// Bind a child node to the node.
    ///
    /// # Panics
    /// If there is already a child node.
    pub fn add_child(&mut self, index: OwnedIndex) {
        assert!(self.child.is_none());
        self.child = Some(index);
    }

    /// Get the tree index of the child.
    pub fn get_child(&self) -> Option<NodeIndex> {
        self.child.as_ref().map(OwnedIndex::shareable)
    }
}

impl Default for Minimum {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode for Minimum {
    fn get_align(&self) -> (Alignment, Alignment) {
        self.align
    }

    fn get_align_mut(&mut self) -> (&mut Alignment, &mut Alignment) {
        (&mut self.align.0, &mut self.align.1)
    }

    fn calculate_min_size(&self, tree: &UiTree) -> (f32, f32) {
        if let Some(child) = self.child.as_ref() {
            let child = tree
                .get_cache(child.shareable())
                .expect("Child not in cache");
            let (w, h) = child.min_size;
            (w.max(self.min_override.0), h.max(self.min_override.1))
        } else {
            self.min_override
        }
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        if let Some(child) = &self.child {
            let child = child.shareable();

            let child_min = tree.get_cache(child).expect("Child not in cache").min_size;
            let child = tree.get_node(child).expect("Child not in cache");
            let space = cache.rect.align(child.get_align(), child_min);
            vec![space]
        } else {
            vec![]
        }
    }

    fn get_children(&self) -> Vec<NodeIndex> {
        self.child.iter().map(OwnedIndex::shareable).collect()
    }
}

/// Behaves identically to a childless [`Minimum`]. It has a fixed size.
pub struct Spacer {
    size: (f32, f32),

    align: (Alignment, Alignment),
}

impl Spacer {
    /// Creates a new `Spacer` with no size and default alignment.
    pub fn new() -> Self {
        Self {
            size: (0.0, 0.0),
            align: Default::default(),
        }
    }

    /// Creates a new `Spacer` with the given size and default alignment.
    ///
    /// Equivalent to `Spacer::new().with_size(size)`.
    pub fn sized(size: (f32, f32)) -> Self {
        Self {
            size,
            align: Default::default(),
        }
    }

    /// Set the horizontal and vertical alignment.
    pub fn with_align(mut self, align: (Alignment, Alignment)) -> Self {
        self.align = align;
        self
    }

    /// Set the size.
    ///
    /// # Panics
    /// If `size` is negative
    pub fn with_size(mut self, size: (f32, f32)) -> Self {
        assert!(size.0 >= 0.0 && size.1 >= 0.0);
        self.size = size;
        self
    }

    /// Set the size.
    ///
    /// # Panics
    /// If `size` is negative
    pub fn set_size(&mut self, size: (f32, f32)) {
        assert!(size.0 >= 0.0 && size.1 >= 0.0);
        self.size = size;
    }

    /// Get the size.
    pub fn get_size(&self) -> (f32, f32) {
        self.size
    }
}

impl Default for Spacer {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode for Spacer {
    fn get_align(&self) -> (Alignment, Alignment) {
        self.align
    }

    fn get_align_mut(&mut self) -> (&mut Alignment, &mut Alignment) {
        (&mut self.align.0, &mut self.align.1)
    }

    fn calculate_min_size(&self, _tree: &UiTree) -> (f32, f32) {
        self.size
    }
}
