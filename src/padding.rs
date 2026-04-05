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
//! [`Center`]: Alignment::Center

use thunderdome::Index as TdIndex;

use crate::{Alignment, NodeCache, Rect, UiNode, UiTree};

/// Maintains a margin around a singular child.
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
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,

    align: (Alignment, Alignment),
    child: Option<TdIndex>,
}

impl Margin {
    /// Creates a new `Margin` with no child, no margin, and ([`Begin`], [`Begin`]) alignment.
    /// 
    /// [`Begin`]: Alignment::Begin
    pub fn new() -> Self {
        Self {
            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
            align: (Alignment::Begin, Alignment::Begin),
            child: None,
        }
    }

    /// Create a child node and bind it to the margin.
    pub fn with_child(mut self, child: impl UiNode, tree: &mut UiTree) -> Self {
        self.child = Some(tree.add_node(child));
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
        if let Some(child) = self.child {
            let child = tree.get_cache(child).expect("Child not in cache");
            let (w, h) = child.min_size;
            (w + self.left + self.right, h + self.top + self.bottom)
        } else {
            (self.left + self.right, self.top + self.bottom)
        }
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        let Some(child_idx) = self.child else { return vec![]; };

        let child_min = tree.get_cache(child_idx).expect("Child not in cache").min_size;
        let child = tree.get_node(child_idx).expect("Child not in cache");

        let space = Rect::new(
            cache.rect.x + self.left,
            cache.rect.y + self.top,
            cache.rect.width - self.left - self.right,
            cache.rect.height - self.top - self.bottom,
        )
        .align(child.get_align(), child_min);

        vec![space]
    }

    fn get_children(&self) -> Vec<TdIndex> {
        self.child.into_iter().collect()
    }
}

/// Maintains an additional constraint to minimum size.
/// 
/// Maintains a minimum size even if `child` is `None`, however, once a child is
/// assigned, it cannot be removed.
pub struct Minimum {
    /// The minimum size to maintain.
    pub min_override: (f32, f32),

    child: Option<TdIndex>,
    align: (Alignment, Alignment),
}

impl Minimum {
    /// Creates a new `Minimum` with no child, no minimum override, and ([`Begin`], [`Begin`])
    /// alignment.
    /// 
    /// [`Begin`]: Alignment::Begin
    pub fn new() -> Self {
        Self {
            min_override: (0.0, 0.0),
            child: None,
            align: (Alignment::Begin, Alignment::Begin),
        }
    }

    /// Create a child node and bind it to the node.
    pub fn with_child(mut self, child: impl UiNode, tree: &mut UiTree) -> Self {
        self.child = Some(tree.add_node(child));
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
        if let Some(child) = self.child {
            let child = tree.get_cache(child).expect("Child not in cache");
            let (w, h) = child.min_size;
            (w.max(self.min_override.0), h.max(self.min_override.1))
        } else {
            self.min_override
        }
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        if let Some(child) = self.child {
            let child_min = tree.get_cache(child).expect("Child not in cache").min_size;
            let child = tree.get_node(child).expect("Child not in cache");
            let space = cache.rect.align(child.get_align(), child_min);
            vec![space]
        } else {
            vec![]
        }
    }

    fn get_children(&self) -> Vec<TdIndex> {
        self.child.into_iter().collect()
    }
}