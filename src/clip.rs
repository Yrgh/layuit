//! Container nodes that reduce the size a child occupies.
//!
//! [`Clip`] allows a child to outgrow its parent, and enables an offset to be applied if the child
//! does. This can be used to create a scrolling area, and is best used when wrapped in a
//! [`Minimum`].
//!
//! The layout system can normally be thought of as drawing boxes within boxes on a sheet of paper.
//! The lines can touch but not cross. A `Clip` instead represents a hole in the paper, with another
//! sheet beneath. The lines still don't touch, but only because the rest of the paper is hidden.
//! Thus, the hole can be as small as desired compared to the paper beneath.
//!
//! [`Minimum`]: crate::padding::Minimum

use thunderdome::Index as TdIndex;

use crate::{Alignment, NodeCache, Rect, UiNode, UiTree};

/// Allows a child to outgrow its parent and be clipped to the parent's bounds.
///
/// `Clip` is unaffected by its child's minimum size.
///
/// In each axis, if the child is larger than the parent, it is affected by the offset but not
/// alignment. If the child is smaller than the parent, it is affected by the alignment but not the
/// offset.
///
/// Once the child is added, it cannot be removed.
pub struct Clip {
    /// The offset to apply to the child if it exceeds this node's bounds.
    pub offset: (f32, f32),

    align: (Alignment, Alignment),
    child: Option<TdIndex>,
}

impl Clip {
    /// Creates a new `Clip` with no child, no alignment, and no offset.
    pub fn new() -> Self {
        Self {
            offset: (0.0, 0.0),
            align: (Alignment::Begin, Alignment::Begin),
            child: None,
        }
    }

    /// Create a child node and bind it to the node.
    ///
    /// # Panics
    /// If there is already a child node.
    pub fn with_child(mut self, child: impl UiNode, tree: &mut UiTree) -> Self {
        assert!(self.child.is_none());
        self.child = Some(tree.add_node(child));
        self
    }

    /// Set the horizontal and vertical alignment.
    ///
    /// Note that the alignment is only applied if the child is smaller than the parent in a given
    /// axis.
    pub fn with_align(mut self, align: (Alignment, Alignment)) -> Self {
        self.align = align;
        self
    }

    /// Set the horizontal and vertical offset.
    ///
    /// Note that the offset is only applied if the child is larger than the parent in a given axis.
    ///
    /// # Panics
    /// If the offset is negative
    pub fn with_offset(mut self, offset: (f32, f32)) -> Self {
        assert!(offset.0 >= 0.0 && offset.1 >= 0.0);
        self.offset = offset;
        self
    }

    /// Set the horizontal and vertical offset.
    ///
    /// Note that the offset is only applied if the child is larger than the parent in a given axis.
    ///
    /// # Panics
    /// If the offset is negative
    pub fn set_offset(&mut self, offset: (f32, f32)) {
        assert!(offset.0 >= 0.0 && offset.1 >= 0.0);
        self.offset = offset;
    }
}

impl Default for Clip {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode for Clip {
    fn get_align(&self) -> (Alignment, Alignment) {
        self.align
    }

    fn get_align_mut(&mut self) -> (&mut Alignment, &mut Alignment) {
        (&mut self.align.0, &mut self.align.1)
    }

    fn calculate_min_size(&self, _tree: &UiTree) -> (f32, f32) {
        (0.0, 0.0)
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        if let Some(child) = self.child {
            let child_min = tree.get_cache(child).expect("Child not in cache").min_size;
            let child = tree.get_node(child).expect("Child not in cache");
            let (cax, cay) = child.get_align();

            // If the child is larger than the parent, clip and offset it. If the child is smaller
            // than the parent, apply alignment

            let (x, width) = match cax {
                _ if child_min.0 > cache.rect.w => (cache.rect.x + self.offset.0, child_min.0),
                Alignment::Begin => (cache.rect.x, child_min.0),
                Alignment::Center => (
                    cache.rect.x + (cache.rect.w - child_min.0) * 0.5,
                    child_min.0,
                ),
                Alignment::End => (cache.rect.x + cache.rect.w - child_min.0, child_min.0),
                Alignment::Full => (cache.rect.x, cache.rect.w),
            };

            let (y, height) = match cay {
                _ if child_min.1 > cache.rect.h => (cache.rect.y + self.offset.1, child_min.1),
                Alignment::Begin => (cache.rect.y, child_min.1),
                Alignment::Center => (
                    cache.rect.y + (cache.rect.h - child_min.1) * 0.5,
                    child_min.1,
                ),
                Alignment::End => (cache.rect.y + cache.rect.h - child_min.1, child_min.1),
                Alignment::Full => (cache.rect.y, cache.rect.h),
            };

            vec![Rect::new(x, y, width, height)]
        } else {
            vec![]
        }
    }

    fn get_children(&self) -> Vec<TdIndex> {
        self.child.into_iter().collect()
    }
}
