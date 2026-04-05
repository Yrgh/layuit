use thunderdome::Index as TdIndex;

use crate::{Alignment, NodeCache, Rect, UiNode, UiTree};

/// Maintains a padding border around a singular child.
/// 
/// Padding is applied before alignment, so [`Alignment::Center`] may have a shifted appearance if
/// `top != bottom` or `left != right`.
/// 
/// Maintains padding even if `child` is `None`, however, once a child is assigned, it cannot be
/// removed.
pub struct Margin {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,

    align: (Alignment, Alignment),
    child: Option<TdIndex>,
}

impl Margin {
    /// Creates a new `Margin` with no child, no padding, and ([`Begin`], [`Begin`]) alignment.
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

    /// Set the left, right, top, and bottom padding.
    pub fn with_margins(mut self, left: f32, right: f32, top: f32, bottom: f32) -> Self {
        self.left = left;
        self.right = right;
        self.top = top;
        self.bottom = bottom;
        self
    }

    pub fn with_left(mut self, left: f32) -> Self {
        self.left = left;
        self
    }

    pub fn with_right(mut self, right: f32) -> Self {
        self.right = right;
        self
    }

    pub fn with_top(mut self, top: f32) -> Self {
        self.top = top;
        self
    }

    pub fn with_bottom(mut self, bottom: f32) -> Self {
        self.bottom = bottom;
        self
    }

    pub fn set_align(&mut self, align: (Alignment, Alignment)) {
        self.align = align;
    }

    pub fn set_margins(&mut self, left: f32, right: f32, top: f32, bottom: f32) {
        self.left = left;
        self.right = right;
        self.top = top;
        self.bottom = bottom;
    }

    pub fn set_left(&mut self, left: f32) {
        self.left = left;
    }

    pub fn set_right(&mut self, right: f32) {
        self.right = right;
    }

    pub fn set_top(&mut self, top: f32) {
        self.top = top;
    }

    pub fn set_bottom(&mut self, bottom: f32) {
        self.bottom = bottom;
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

    fn get_align_mut(&mut self) -> &mut (Alignment, Alignment) {
        &mut self.align
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

        let child = tree.get_node(child_idx).expect("Child not in cache");

        let space = Rect::new(
            cache.rect.x + self.left,
            cache.rect.y + self.top,
            cache.rect.width - self.left - self.right,
            cache.rect.height - self.top - self.bottom,
        )
        .align(child.get_align(), cache.min_size);

        vec![space]
    }

    fn get_children(&self) -> Vec<TdIndex> {
        self.child.into_iter().collect()
    }
}