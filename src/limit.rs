//! Containers that create upper bounds for nodes.

use thunderdome::Index as NodeIndex;

use crate::{Alignment, Anchor, NodeCache, Rect, UiNode, UiTree};

/// Limits a node's maximum size.
///
/// If the node's minimum size exceeds the maximum, the minimum takes precedence.
pub struct Clamp {
    /// The maximum size to maintain.
    pub max_override: (f32, f32),

    /// The position to place the shrunken space. The child is then aligned within the new space.
    pub anchor: (Anchor, Anchor),

    child: Option<NodeIndex>,
    align: (Alignment, Alignment),
}

impl Clamp {
    /// Creates a new `Clamp` with no child, a size limit of 0, default anchoring, and ([`Begin`],
    /// [`Begin`]) alignment.
    ///
    /// The default maximum has no effect, as the maximum is immediately overridden by the child's
    /// min size.
    ///
    /// [`Begin`]: Alignment::Begin
    pub fn new() -> Self {
        Self {
            max_override: (0.0, 0.0),
            child: None,
            anchor: (Anchor::Center, Anchor::Center),
            align: (Alignment::Begin, Alignment::Begin),
        }
    }

    /// bind a child node.
    ///
    /// # Panics
    /// If there is already a child node.
    pub fn with_child(mut self, index: NodeIndex) -> Self {
        assert!(self.child.is_none());
        self.child = Some(index);
        self
    }

    /// Set the horizontal and vertical alignment.
    pub fn with_align(mut self, align: (Alignment, Alignment)) -> Self {
        self.align = align;
        self
    }

    /// Set the horizontal and vertical anchor mode for the shrunken space.
    pub fn with_anchor(mut self, anchor: (Anchor, Anchor)) -> Self {
        self.anchor = anchor;
        self
    }

    /// Set the maximum size.
    ///
    /// If the maximum size is smaller than the minimum size, the minimum takes precedence.
    pub fn with_max(mut self, max: (f32, f32)) -> Self {
        self.max_override = max;
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

    /// Get the tree index of the child.
    pub fn get_child(&self) -> Option<NodeIndex> {
        self.child
    }
}

impl Default for Clamp {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode for Clamp {
    fn get_align(&self) -> (Alignment, Alignment) {
        self.align
    }

    fn get_align_mut(&mut self) -> (&mut Alignment, &mut Alignment) {
        (&mut self.align.0, &mut self.align.1)
    }

    fn calculate_min_size(&self, tree: &UiTree) -> (f32, f32) {
        if let Some(index) = self.child {
            tree.get_cache(index).expect("Child not in cache").min_size
        } else {
            (0.0, 0.0)
        }
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        if let Some(index) = self.child {
            let child_min = tree.get_cache(index).expect("Child not in cache").min_size;
            let child = tree.get_node(index).expect("Child not in cache");

            // The size must be at most the given, at least the min, and defaults to the max
            let w = cache.rect.w.clamp(child_min.0, self.max_override.0);
            let h = cache.rect.h.clamp(child_min.1, self.max_override.1);

            let shrunk = cache.rect.anchor(self.anchor, (w, h));
            let space = shrunk.align(child.get_align(), child_min);
            vec![space]
        } else {
            vec![]
        }
    }

    fn get_children(&self) -> Vec<NodeIndex> {
        self.child.into_iter().collect()
    }
}
