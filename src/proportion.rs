//! Containers that use ratios, but maintain minimum size requirements.
//!
//! [`AspectRatio`] maintains a runtime-configurable horizontal:vertical ratio.
//!
//! [`HSplit`] and [`VSplit`] distribute a percentage of the horizontal or vertical space to the
//! left or top child, and give the rest to the other child.
//!
//! [`Percent`] gives a child a percentage of the available space. It can be configured to extend
//! the minimum size to ensure the percentage is always maintained.

use thunderdome::Index as TdIndex;

use crate::{Alignment, NodeCache, Rect, UiNode, UiTree};

/// Shrinks the horizontal or vertical dimensions of a child to maintain an aspect ratio.
///
/// Once the child is added, it cannot be removed.
pub struct AspectRatio {
    ratio: f32,

    align: (Alignment, Alignment),
    child: Option<TdIndex>,
}

impl AspectRatio {
    /// Creates a new `AspectRatio` with no child, no alignment, and a 1:1 ratio.
    pub fn new() -> Self {
        Self {
            ratio: 1.0,
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
    pub fn with_align(mut self, align: (Alignment, Alignment)) -> Self {
        self.align = align;
        self
    }

    /// Set the horizontal:vertical ratio to be maintained.
    ///
    /// # Panics
    /// If `ratio <= 0.0`.
    pub fn with_ratio(mut self, ratio: f32) -> Self {
        assert!(ratio > 0.0);
        self.ratio = ratio;
        self
    }

    /// Returns the current ratio.
    pub fn get_ratio(&self) -> f32 {
        self.ratio
    }

    /// Set the horizontal:vertical ratio to be maintained.
    ///
    /// # Panics
    /// If `ratio <= 0.0`.
    pub fn set_ratio(&mut self, ratio: f32) {
        assert!(ratio > 0.0);
        self.ratio = ratio;
    }
}

impl Default for AspectRatio {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode for AspectRatio {
    fn get_align(&self) -> (Alignment, Alignment) {
        self.align
    }

    fn get_align_mut(&mut self) -> (&mut Alignment, &mut Alignment) {
        (&mut self.align.0, &mut self.align.1)
    }

    fn calculate_min_size(&self, tree: &UiTree) -> (f32, f32) {
        if let Some(child) = self.child {
            let child_min = tree.get_cache(child).expect("Child not in cache").min_size;

            // Prevent division by zero
            if child_min.0 == 0.0 || child_min.1 == 0.0 {
                return (0.0, 0.0);
            }

            let child_ratio = child_min.0 / child_min.1;
            if child_ratio > self.ratio {
                (child_min.0, child_min.0 / self.ratio)
            } else {
                (child_min.1 * self.ratio, child_min.1)
            }
        } else {
            (0.0, 0.0)
        }
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        if let Some(child) = self.child {
            let child_min = tree.get_cache(child).expect("Child not in cache").min_size;

            // Prevent division by zero
            if child_min.0 == 0.0 || child_min.1 == 0.0 {
                return vec![Rect::new(cache.rect.x, cache.rect.y, 0.0, 0.0)];
            }

            let child_ratio = child_min.0 / child_min.1;

            // If the child is wider than it should be, compress the width. If it is taller,
            // compress the height
            let size = if child_ratio > self.ratio {
                (child_min.1 * self.ratio, child_min.1)
            } else {
                (child_min.0, child_min.0 / self.ratio)
            };

            let space = cache.rect.align(self.align, size);
            vec![space]
        } else {
            vec![]
        }
    }

    fn get_children(&self) -> Vec<TdIndex> {
        self.child.into_iter().collect()
    }
}

/// Divides the space between two children horizontally, giving the left child a proportion of the
/// space.
///
/// Both nodes recieve their minimum size. If a child would not recieve its minimum size, the
/// percentage is bypassed.
pub struct HSplit {
    /// The space between the two children.
    pub spacing: f32,

    percent: f32,

    align: (Alignment, Alignment),
    children: Option<(TdIndex, TdIndex)>,
}

impl HSplit {
    /// Creates a new `HSplit` with no children, 0 spacing, 50/50 split, and ([`Begin`], [`Begin`])
    /// alignment.
    ///
    /// [`Begin`]: Alignment::Begin
    pub fn new() -> Self {
        Self {
            spacing: 0.0,
            percent: 0.5,
            align: (Alignment::Begin, Alignment::Begin),
            children: None,
        }
    }

    /// Create two child nodes and bind them to the node.
    ///
    /// # Panics
    /// If there are already child nodes.
    pub fn with_children(
        mut self,
        left: impl UiNode,
        right: impl UiNode,
        tree: &mut UiTree,
    ) -> Self {
        assert!(self.children.is_none());
        self.children = Some((tree.add_node(left), tree.add_node(right)));
        self
    }

    /// Set the horizontal and vertical alignment.
    pub fn with_align(mut self, align: (Alignment, Alignment)) -> Self {
        self.align = align;
        self
    }

    /// Set the spacing between the two children
    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set the percentage of space to give to the first child
    ///
    /// # Panics
    /// If the percentage is not between 0.0 and 1.0
    pub fn with_percent(mut self, percent: f32) -> Self {
        assert!(matches!(percent, 0.0..=1.0));
        self.percent = percent;
        self
    }

    /// Get the percentage of space to give to the first child
    pub fn get_percent(&self) -> f32 {
        self.percent
    }

    /// Set the percentage of space to give to the first child
    ///
    /// # Panics
    /// If the percentage is not between 0.0 and 1.0
    pub fn set_percent(&mut self, percent: f32) {
        assert!(matches!(percent, 0.0..=1.0));
        self.percent = percent;
    }
}

impl Default for HSplit {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode for HSplit {
    fn get_align(&self) -> (Alignment, Alignment) {
        self.align
    }

    fn get_align_mut(&mut self) -> (&mut Alignment, &mut Alignment) {
        (&mut self.align.0, &mut self.align.1)
    }

    fn calculate_min_size(&self, tree: &UiTree) -> (f32, f32) {
        if let Some((left, right)) = self.children {
            let left_min = tree
                .get_cache(left)
                .expect("Left child not in cache")
                .min_size;
            let right_min = tree
                .get_cache(right)
                .expect("Right child not in cache")
                .min_size;
            (
                left_min.0 + right_min.0 + self.spacing,
                left_min.1.max(right_min.1),
            )
        } else {
            (0.0, 0.0)
        }
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        if let Some((left, right)) = self.children {
            let left_min = tree
                .get_cache(left)
                .expect("Left child not in cache")
                .min_size;
            let right_min = tree
                .get_cache(right)
                .expect("Right child not in cache")
                .min_size;

            let div_left = (cache.rect.w - self.spacing) * self.percent;

            // If there is not enough space for the left child, give it enough space and give the
            // right child the rest.

            if div_left < left_min.0 {
                let div_right = cache.rect.w - left_min.0 - self.spacing;
                let x_right = cache.rect.x + left_min.0 + self.spacing;

                let left_space = Rect::new(cache.rect.x, cache.rect.y, left_min.0, cache.rect.h)
                    .align(self.align, left_min);
                let right_space = Rect::new(x_right, cache.rect.y, div_right, cache.rect.h)
                    .align(self.align, right_min);
                return vec![left_space, right_space];
            }

            let div_right = cache.rect.w - div_left - self.spacing;

            // If there is not enough space for the right child, give it enough space and give the
            // left child the rest.

            if div_right < right_min.0 {
                let div_left = cache.rect.w - right_min.0 - self.spacing;
                let x_right = cache.rect.x + div_left + self.spacing;

                let left_space = Rect::new(cache.rect.x, cache.rect.y, div_left, cache.rect.h)
                    .align(self.align, left_min);
                let right_space = Rect::new(x_right, cache.rect.y, right_min.0, cache.rect.h)
                    .align(self.align, right_min);
                return vec![left_space, right_space];
            }

            let x_right = cache.rect.x + div_left + self.spacing;

            // If there is enough space for both children, use the percentage to divide the space.

            let left_space = Rect::new(cache.rect.x, cache.rect.y, div_left, cache.rect.h)
                .align(self.align, left_min);
            let right_space = Rect::new(x_right, cache.rect.y, div_right, cache.rect.h)
                .align(self.align, right_min);
            vec![left_space, right_space]
        } else {
            vec![]
        }
    }

    fn get_children(&self) -> Vec<TdIndex> {
        if let Some((left, right)) = self.children {
            vec![left, right]
        } else {
            vec![]
        }
    }
}

/// Divides the space between two children vertically, giving the top child a proportion of the
/// space.
///
/// Both nodes recieve their minimum size. If a child would not recieve its minimum size, the
/// percentage is bypassed.
pub struct VSplit {
    /// The space between the two children.
    pub spacing: f32,

    percent: f32,

    align: (Alignment, Alignment),
    children: Option<(TdIndex, TdIndex)>,
}

impl VSplit {
    /// Creates a new `VSplit` with no children, 0 spacing, 50/50 split, and ([`Begin`], [`Begin`])
    /// alignment.
    ///
    /// [`Begin`]: Alignment::Begin
    pub fn new() -> Self {
        Self {
            spacing: 0.0,
            percent: 0.5,
            align: (Alignment::Begin, Alignment::Begin),
            children: None,
        }
    }

    /// Create two child nodes and bind them to the node.
    ///
    /// # Panics
    /// If there are already child nodes.
    pub fn with_children(
        mut self,
        left: impl UiNode,
        right: impl UiNode,
        tree: &mut UiTree,
    ) -> Self {
        assert!(self.children.is_none());
        self.children = Some((tree.add_node(left), tree.add_node(right)));
        self
    }

    /// Set the horizontal and vertical alignment.
    pub fn with_align(mut self, align: (Alignment, Alignment)) -> Self {
        self.align = align;
        self
    }

    /// Set the spacing between the two children.
    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set the percentage of space to give to the first child
    ///
    /// # Panics
    /// If the percentage is not between 0.0 and 1.0
    pub fn with_percent(mut self, percent: f32) -> Self {
        assert!(matches!(percent, 0.0..=1.0));
        self.percent = percent;
        self
    }

    /// Get the percentage of space to give to the first child.
    pub fn get_percent(&self) -> f32 {
        self.percent
    }

    /// Set the percentage of space to give to the first child
    ///
    /// # Panics
    /// If the percentage is not between 0.0 and 1.0
    pub fn set_percent(&mut self, percent: f32) {
        assert!(matches!(percent, 0.0..=1.0));
        self.percent = percent;
    }
}

impl Default for VSplit {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode for VSplit {
    fn get_align(&self) -> (Alignment, Alignment) {
        self.align
    }

    fn get_align_mut(&mut self) -> (&mut Alignment, &mut Alignment) {
        (&mut self.align.0, &mut self.align.1)
    }

    fn calculate_min_size(&self, tree: &UiTree) -> (f32, f32) {
        if let Some((top, bot)) = self.children {
            let top_min = tree
                .get_cache(top)
                .expect("Top child not in cache")
                .min_size;
            let bot_min = tree
                .get_cache(bot)
                .expect("Bottom child not in cache")
                .min_size;
            (
                top_min.0.max(bot_min.0),
                top_min.1 + bot_min.1 + self.spacing,
            )
        } else {
            (0.0, 0.0)
        }
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        if let Some((top, bot)) = self.children {
            let top_min = tree
                .get_cache(top)
                .expect("Top child not in cache")
                .min_size;
            let bot_min = tree
                .get_cache(bot)
                .expect("Bottom child not in cache")
                .min_size;

            let div_top = (cache.rect.h - self.spacing) * self.percent;

            if div_top < top_min.1 {
                let div_bot = cache.rect.h - top_min.1 - self.spacing;
                let y_bot = cache.rect.y + top_min.1 + self.spacing;

                let top_space = Rect::new(cache.rect.x, cache.rect.y, cache.rect.w, top_min.1)
                    .align(self.align, top_min);
                let bot_space = Rect::new(cache.rect.x, y_bot, cache.rect.w, div_bot)
                    .align(self.align, bot_min);
                return vec![top_space, bot_space];
            }

            let div_bot = cache.rect.h - div_top - self.spacing;

            if div_bot < bot_min.1 {
                let div_top = cache.rect.h - bot_min.1 - self.spacing;
                let y_bot = cache.rect.y + div_top + self.spacing;

                let top_space = Rect::new(cache.rect.x, cache.rect.y, cache.rect.w, div_top)
                    .align(self.align, top_min);
                let bot_space = Rect::new(cache.rect.x, y_bot, cache.rect.w, bot_min.1)
                    .align(self.align, bot_min);
                return vec![top_space, bot_space];
            }

            let y_bot = cache.rect.y + div_top + self.spacing;

            let top_space = Rect::new(cache.rect.x, cache.rect.y, cache.rect.w, div_top)
                .align(self.align, top_min);
            let bot_space =
                Rect::new(cache.rect.x, y_bot, cache.rect.w, div_bot).align(self.align, bot_min);
            vec![top_space, bot_space]
        } else {
            vec![]
        }
    }

    fn get_children(&self) -> Vec<TdIndex> {
        if let Some((top, bot)) = self.children {
            vec![top, bot]
        } else {
            vec![]
        }
    }
}

/// Assigns a percentage of the available space to the child.
///
/// If `strict` is not enabled, and there is not enough space to maintain the percentage, it will be
/// bypassed. If `strict` is enabled, the minimum size will grow to ensure the percentage is always
/// maintained.
///
/// Once the child is added, it cannot be removed.
pub struct Percent {
    /// If `true`, the minimum size grows to ensure the percentage is always maintained.
    pub strict: bool,
    percent: (f32, f32),

    align: (Alignment, Alignment),
    child: Option<TdIndex>,
}

impl Percent {
    /// Creates a new `Percent` with no child, no alignment, a (100%, 100%) percent, `strict`
    /// disabled, and ([`Begin`], [`Begin`]) alignment.
    ///
    /// [`Begin`]: Alignment::Begin
    pub fn new() -> Self {
        Self {
            strict: false,
            percent: (1.0, 1.0),
            align: (Alignment::Begin, Alignment::Begin),
            child: None,
        }
    }

    /// Set the child of the `Percent`.
    ///
    /// # Panics
    /// If the child is already set
    pub fn with_child(mut self, child: impl UiNode, tree: &mut UiTree) -> Self {
        assert!(self.child.is_none());
        self.child = Some(tree.add_node(child));
        self
    }

    /// Set the horizontal and vertical alignment.
    pub fn with_align(mut self, align: (Alignment, Alignment)) -> Self {
        self.align = align;
        self
    }

    /// Set the percentage of space to give to the first child
    ///
    /// # Panics
    /// If the percentage is not between 0.0 and 1.0 or if `strict` is enabled and the percent is 0
    pub fn with_percent(mut self, percent: (f32, f32)) -> Self {
        assert!(matches!(percent, (0.0..=1.0, 0.0..=1.0)));
        if self.strict {
            assert!(
                self.percent.0 > 0.0 && self.percent.1 > 0.0,
                "Percent must be greater than 0 when strict is enabled"
            );
        }
        self.percent = percent;
        self
    }

    /// Set whether the minimum size grows to ensure the percentage is always maintained.
    ///
    /// # Panics
    /// If `strict` is enabled and the percent is 0
    pub fn with_strict(mut self, strict: bool) -> Self {
        if strict {
            assert!(
                self.percent.0 > 0.0 && self.percent.1 > 0.0,
                "Percent must be greater than 0 when strict is enabled"
            );
        }
        self.strict = strict;
        self
    }

    /// Set the percentage of space to give to the first child
    ///
    /// # Panics
    /// If the percentage is not between 0.0 and 1.0 or if `strict` is enabled and the percent is 0
    pub fn set_percent(&mut self, percent: (f32, f32)) {
        assert!(matches!(percent, (0.0..=1.0, 0.0..=1.0)));
        if self.strict {
            assert!(
                self.percent.0 > 0.0 && self.percent.1 > 0.0,
                "Percent must be greater than 0 when strict is enabled"
            );
        }
        self.percent = percent;
    }

    /// Set whether the minimum size grows to ensure the percentage is always maintained.
    ///
    /// # Panics
    /// If `strict` is enabled and the percent is 0
    pub fn set_strict(&mut self, strict: bool) {
        if strict {
            assert!(
                self.percent.0 > 0.0 && self.percent.1 > 0.0,
                "Percent must be greater than 0 when strict is enabled"
            );
        }
        self.strict = strict;
    }

    /// Get the percentage of space to give to the first child
    pub fn get_percent(&self) -> (f32, f32) {
        self.percent
    }
}

impl Default for Percent {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode for Percent {
    fn get_align(&self) -> (Alignment, Alignment) {
        self.align
    }

    fn get_align_mut(&mut self) -> (&mut Alignment, &mut Alignment) {
        (&mut self.align.0, &mut self.align.1)
    }

    fn calculate_min_size(&self, tree: &UiTree) -> (f32, f32) {
        if let Some(child) = self.child {
            let child_min = tree.get_cache(child).expect("Child not in cache").min_size;
            if self.strict {
                // If percent is 50%, the minimum size doubles.
                (child_min.0 / self.percent.0, child_min.1 / self.percent.1)
            } else {
                child_min
            }
        } else {
            (0.0, 0.0)
        }
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        if let Some(child) = self.child {
            let child_min = tree.get_cache(child).expect("Child not in cache").min_size;
            // Child gets enough space but can get up to the percent.
            let w = child_min.0.max(cache.rect.w * self.percent.0);
            let h = child_min.1.max(cache.rect.h * self.percent.1);
            let space = cache.rect.align(self.align, (w, h));
            vec![space]
        } else {
            vec![]
        }
    }

    fn get_children(&self) -> Vec<TdIndex> {
        if let Some(child) = self.child {
            vec![child]
        } else {
            vec![]
        }
    }
}
