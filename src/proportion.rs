//! Containers that use ratios, but maintain minimum size requirements.
//!
//! [`AspectRatio`] maintains a runtime-configurable horizontal:vertical ratio.
//!
//! [`HSplit`] and [`VSplit`] distribute a percentage of the horizontal or vertical space to the
//! left or top child, and give the rest to the other child.
//!
//! [`Percent`] gives a child a percentage of the available space. It can be configured to extend
//! the minimum size to ensure the percentage is always maintained.
//!
//! ## Strict vs non-strict [`Percent`]
//!
//! `Percent` features a field called `strict`. While normally disabled, when enabled, the minimum
//! size of the `Percent` will grow to ensure that the child's minimum is exactly the percentage of
//! the `Percent`'s minimum. When disabled, the minimum size will be the child's minimum size, and
//! the `Percent` will not ensure the percentage is always maintained if it is too small.
//!
//! ```rust
//! use layuit::Rect;
//! use layuit::proportion::Percent;
//! use layuit::stack::HStack;
//! use layuit::padding::Spacer;
//!
//! let mut tree = layuit::ui!(
//!     %%,
//!     +|+ HStack::new() => [
//!         -|- Percent::new()
//!             .with_percent((0.5, 0.5))
//!             .with_strict(true)
//!         => [
//!             -|- Spacer::sized((10.0, 10.0))
//!         ],
//!         -|- Percent::new()
//!             .with_percent((0.5, 0.5))
//!             .with_strict(false)
//!         => [
//!             -|- Spacer::sized((10.0, 10.0))
//!         ],
//!     ]
//! );
//!
//! tree.calculate_layout(Rect::new(0.0, 0.0, 30.0, 20.0));
//!
//! // Final results:
//!
//! // strict has a size of 20x20, since its minimum grew to ensure the percentage is always upheld.
//!
//! // non_strict has a size of 10x10, since its minimum remained the same as the spacer, and the
//! // percentage was not upheld.
//! ```

use crate::{Alignment, Anchor, NodeCache, NodeIndex, OwnedIndex, Rect, UiNode, UiTree};

/// Expands the horizontal or vertical dimensions of a child to maintain an aspect ratio.
///
/// An anchor must be specified to determine where the child should be placed after the aspect ratio
/// is applied.
///
/// Once the child is added, it cannot be removed.
pub struct AspectRatio {
    ratio: f32,

    /// The position to place the shrunken space. The child is then aligned within the new space.
    pub anchor: (Anchor, Anchor),

    align: (Alignment, Alignment),
    child: Option<OwnedIndex>,
}

impl AspectRatio {
    /// Creates a new `AspectRatio` with no child, default anchoring, a 1:1 ratio, and default
    /// alignment.
    pub fn new() -> Self {
        Self {
            ratio: 1.0,
            anchor: (Anchor::Center, Anchor::Center),
            align: Default::default(),
            child: None,
        }
    }

    /// Create a child node and bind it to the node.
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

    /// Set the horizontal and vertical anchor.
    pub fn with_anchor(mut self, anchor: (Anchor, Anchor)) -> Self {
        self.anchor = anchor;
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

    /// Bind a child node.
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
        if let Some(child) = &self.child {
            let child_min = tree
                .get_cache(child.shareable())
                .expect("Child not in cache")
                .min_size;

            // Prevent division by zero
            if child_min.0 == 0.0 || child_min.1 == 0.0 {
                return (0.0, 0.0);
            }

            let child_ratio = child_min.0 / child_min.1;
            if child_ratio > self.ratio {
                // Keep width, increase height
                (child_min.0, child_min.0 / self.ratio)
            } else {
                // Keep height, increase width
                (child_min.1 * self.ratio, child_min.1)
            }
        } else {
            (0.0, 0.0)
        }
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        if let Some(child) = &self.child {
            let child_min = tree
                .get_cache(child.shareable())
                .expect("Child not in cache")
                .min_size;

            // Prevent division by zero
            if child_min.0 == 0.0 || child_min.1 == 0.0 {
                return vec![Rect::new(cache.rect.x, cache.rect.y, 0.0, 0.0)];
            }

            let child_ratio = child_min.0 / child_min.1;

            let (w, h) = if child_ratio > self.ratio {
                // Keep width, increase height
                (child_min.0, child_min.0 / self.ratio)
            } else {
                // Keep height, increase width
                (child_min.1 * self.ratio, child_min.1)
            };

            let w = w.max(child_min.0);
            let h = h.max(child_min.1);

            let shrunk = cache.rect.anchor(self.anchor, (w, h));
            let space = shrunk.align(self.align, child_min);
            vec![space]
        } else {
            vec![]
        }
    }

    fn get_children(&self) -> Vec<NodeIndex> {
        self.child.iter().map(OwnedIndex::shareable).collect()
    }
}

/// Divides the space between two children horizontally, giving the left child a proportion of the
/// space.
///
/// Both nodes receive their minimum size. If a child would not receive its minimum size, the
/// percentage is bypassed.
pub struct HSplit {
    /// The space between the two children.
    pub spacing: f32,

    percent: f32,

    align: (Alignment, Alignment),
    left: Option<OwnedIndex>,
    right: Option<OwnedIndex>,
}

impl HSplit {
    /// Creates a new `HSplit` with no children, 0 spacing, 50/50 split, and default alignment.
    pub fn new() -> Self {
        Self {
            spacing: 0.0,
            percent: 0.5,
            align: Default::default(),
            left: None,
            right: None,
        }
    }

    /// Create two child nodes and bind them to the node.
    ///
    /// # Panics
    /// If there are already child nodes.
    pub fn with_children(mut self, left: OwnedIndex, right: OwnedIndex) -> Self {
        assert!(self.left.is_none() && self.right.is_none());
        self.left = Some(left);
        self.right = Some(right);
        self
    }

    /// Creates and binds a child node to the left slot, or the right, if the left is occupied.
    ///
    /// # Panics
    /// If both left and right are set.
    pub fn with_child(mut self, index: OwnedIndex) -> Self {
        self.add_child(index);
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

    /// Get the index of the left child
    ///
    /// # Panics
    /// If the left slot is not set
    pub fn get_left_index(&self) -> NodeIndex {
        self.left.as_ref().expect("Left slot not set").shareable()
    }

    /// Get the index of the right child
    ///
    /// # Panics
    /// If the right slot is not set
    pub fn get_right_index(&self) -> NodeIndex {
        self.right.as_ref().expect("Right slot not set").shareable()
    }

    /// Binds a child node to the left slot, or the right, if the left is occupied.
    ///
    /// # Panics
    /// If both left and right are set.
    pub fn add_child(&mut self, index: OwnedIndex) {
        if self.left.is_none() {
            self.left = Some(index);
        } else if self.right.is_none() {
            self.right = Some(index);
        }
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
        if let Some((left, right)) = self.left.as_ref().zip(self.right.as_ref()) {
            let left = left.shareable();
            let right = right.shareable();

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
        if let Some((left, right)) = self.left.as_ref().zip(self.right.as_ref()) {
            let left = left.shareable();
            let right = right.shareable();

            let left_min = tree
                .get_cache(left)
                .expect("Left child not in cache")
                .min_size;
            let right_min = tree
                .get_cache(right)
                .expect("Right child not in cache")
                .min_size;

            let left_node = tree.get_node(left).expect("Left child not in cache");
            let right_node = tree.get_node(right).expect("Right child not in cache");

            let width = cache.rect.w - self.spacing;
            let div_left = (width * self.percent).clamp(left_min.0, width - right_min.0);
            let div_right = width - div_left;
            let x_right = cache.rect.x + div_left + self.spacing;

            let left_space = Rect::new(cache.rect.x, cache.rect.y, div_left, cache.rect.h)
                .align(left_node.get_align(), left_min);
            let right_space = Rect::new(x_right, cache.rect.y, div_right, cache.rect.h)
                .align(right_node.get_align(), right_min);

            vec![left_space, right_space]
        } else {
            vec![]
        }
    }

    fn get_children(&self) -> Vec<NodeIndex> {
        if let Some((left, right)) = self.left.as_ref().zip(self.right.as_ref()) {
            vec![left.shareable(), right.shareable()]
        } else {
            vec![]
        }
    }
}

/// Divides the space between two children vertically, giving the top child a proportion of the
/// space.
///
/// Both nodes receive their minimum size. If a child would not receive its minimum size, the
/// percentage is bypassed.
pub struct VSplit {
    /// The space between the two children.
    pub spacing: f32,

    percent: f32,

    align: (Alignment, Alignment),
    top: Option<OwnedIndex>,
    bot: Option<OwnedIndex>,
}

impl VSplit {
    /// Creates a new `VSplit` with no children, 0 spacing, 50/50 split, and default alignment.
    pub fn new() -> Self {
        Self {
            spacing: 0.0,
            percent: 0.5,
            align: Default::default(),
            top: None,
            bot: None,
        }
    }

    /// Binds two children to the node.
    ///
    /// # Panics
    /// If there are already child nodes.
    pub fn with_children(mut self, top: OwnedIndex, bottom: OwnedIndex) -> Self {
        assert!(self.top.is_none() && self.bot.is_none());
        self.top = Some(top);
        self.bot = Some(bottom);
        self
    }

    /// Binds a child node to the top slot, or the bottom, if the top is occupied.
    ///
    /// # Panics
    /// If both left and right are set.
    pub fn with_child(mut self, index: OwnedIndex) -> Self {
        self.add_child(index);
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

    /// Returns the tree index of the top node.
    ///
    /// # Panics
    /// If the top node is not set.
    pub fn get_top_index(&self) -> NodeIndex {
        self.top.as_ref().expect("Top slot not set").shareable()
    }

    /// Returns the tree index of the bottom node.
    ///
    /// # Panics
    /// If the top node is not set.
    pub fn get_bottom_index(&self) -> NodeIndex {
        self.bot.as_ref().expect("Bottom slot not set").shareable()
    }

    /// Binds a child node to the top slot, or the bottom, if the top is occupied.
    ///
    /// # Panics
    /// If both left and right are set.
    pub fn add_child(&mut self, index: OwnedIndex) {
        if self.top.is_none() {
            self.top = Some(index);
        } else if self.bot.is_none() {
            self.bot = Some(index);
        } else {
            panic!("Cannot add child when both children are bound");
        }
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
        if let Some((top, bot)) = self.top.as_ref().zip(self.bot.as_ref()) {
            let top = top.shareable();
            let bot = bot.shareable();

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
        if let Some((top, bot)) = self.top.as_ref().zip(self.bot.as_ref()) {
            let top = top.shareable();
            let bot = bot.shareable();

            let top_min = tree
                .get_cache(top)
                .expect("Top child not in cache")
                .min_size;
            let bot_min = tree
                .get_cache(bot)
                .expect("Bottom child not in cache")
                .min_size;

            let top_node = tree.get_node(top).expect("Top child not in cache");
            let bot_node = tree.get_node(bot).expect("Bottom child not in cache");

            let height = cache.rect.h - self.spacing;
            let div_top = (height * self.percent).clamp(top_min.1, height - bot_min.1);
            let div_bot = height - div_top;
            let y_bot = cache.rect.y + div_top + self.spacing;

            let top_space = Rect::new(cache.rect.x, cache.rect.y, cache.rect.w, div_top)
                .align(top_node.get_align(), top_min);
            let bot_space = Rect::new(cache.rect.x, y_bot, cache.rect.w, div_bot)
                .align(bot_node.get_align(), bot_min);

            vec![top_space, bot_space]
        } else {
            vec![]
        }
    }

    fn get_children(&self) -> Vec<NodeIndex> {
        if let Some((top, bot)) = self.top.as_ref().zip(self.bot.as_ref()) {
            vec![top.shareable(), bot.shareable()]
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
    /// The anchor of the child within the shrunk space.
    pub anchor: (Anchor, Anchor),

    align: (Alignment, Alignment),
    child: Option<OwnedIndex>,
}

impl Percent {
    /// Creates a new `Percent` with no child, no alignment, default anchoring, a (100%, 100%)
    /// percent, `strict` disabled, and default alignment.
    pub fn new() -> Self {
        Self {
            strict: false,
            percent: (1.0, 1.0),
            anchor: (Anchor::Begin, Anchor::Begin),
            align: Default::default(),
            child: None,
        }
    }

    /// Set the child of the `Percent`.
    ///
    /// # Panics
    /// If the child is already set
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

    /// Set the horizontal and vertical anchor.
    pub fn with_anchor(mut self, anchor: (Anchor, Anchor)) -> Self {
        self.anchor = anchor;
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
                percent.0 > 0.0 && percent.1 > 0.0,
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
                percent.0 > 0.0 && percent.1 > 0.0,
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

    /// Bind a child node.
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
        if let Some(child) = &self.child {
            let child_min = tree
                .get_cache(child.shareable())
                .expect("Child not in cache")
                .min_size;
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
        if let Some(child) = &self.child {
            let child_min = tree
                .get_cache(child.shareable())
                .expect("Child not in cache")
                .min_size;
            // Child gets enough space but can get up to the percent.
            let w = child_min.0.max(cache.rect.w * self.percent.0);
            let h = child_min.1.max(cache.rect.h * self.percent.1);

            let shrunk = cache.rect.anchor(self.anchor, (w, h));
            let space = shrunk.align(self.align, child_min);
            vec![space]
        } else {
            vec![]
        }
    }

    fn get_children(&self) -> Vec<NodeIndex> {
        if let Some(child) = &self.child {
            vec![child.shareable()]
        } else {
            vec![]
        }
    }
}
