//! 3-way splits
//!
//! Unlike [`HSplit`] and [`VSplit`] from [`proportion`], [`HSplit3`] and [`VSplit3`] use a node
//! as a separator, instead of a fixed space. This can be used to make a before/after image with
//! a handle in between.
//!
//! [`HSplit`]: crate::proportion::HSplit
//! [`VSplit`]: crate::proportion::VSplit
//! [`proportion`]: crate::proportion

use crate::{Alignment, NodeCache, NodeIndex, OwnedIndex, Rect, UiNode, UiTree};

/// Splits the space between 2 children, with a separator, rather than spacing, in between.
///
/// The separator is always shrunk horizontally, and every node receives at least its minimum size,
/// even if that contradicts the percentage.
///
/// The percentage excludes the space of the separator. A percentage of 0.0 will shrink the right
/// child and give the remaining space to the left. A percentage of 1.0 will shrink the left child
/// and give the remaining space to the right.
pub struct HSplit3 {
    percent: f32,

    align: (Alignment, Alignment),

    left: Option<OwnedIndex>,
    sep: Option<OwnedIndex>,
    right: Option<OwnedIndex>,
}

impl HSplit3 {
    /// Creates a new horizontal split with no children, 50/50 split, and default alignment.
    pub fn new() -> Self {
        Self {
            percent: 0.5,
            align: Default::default(),
            left: None,
            sep: None,
            right: None,
        }
    }

    /// Binds the left, separator, and right children to the node.
    ///
    /// # Panics
    /// If any slot is already set.
    pub fn with_children(mut self, left: OwnedIndex, sep: OwnedIndex, right: OwnedIndex) -> Self {
        assert!(self.left.is_none() || self.sep.is_none() || self.right.is_none());
        self.left = Some(left);
        self.sep = Some(sep);
        self.right = Some(right);
        self
    }

    /// Binds a child to the node. The left slot is set first, then the separator, then the right.
    ///
    /// # Panics
    /// If all slots are set.
    pub fn with_child(mut self, index: OwnedIndex) -> Self {
        self.add_child(index);
        self
    }

    /// Sets the alignment of the node.
    pub fn with_align(mut self, align: (Alignment, Alignment)) -> Self {
        self.align = align;
        self
    }

    /// Sets the percentage of the node.
    ///
    /// # Panics
    /// If the percentage is not between 0.0 and 1.0
    pub fn with_percent(mut self, percent: f32) -> Self {
        assert!(matches!(percent, 0.0..=1.0));
        self.percent = percent;
        self
    }

    /// Binds a child to the node. The left slot is set first, then the separator, then the right.
    ///
    /// # Panics
    /// If all slots are set.
    pub fn add_child(&mut self, index: OwnedIndex) {
        if self.left.is_none() {
            self.left = Some(index);
        } else if self.sep.is_none() {
            self.sep = Some(index);
        } else if self.right.is_none() {
            self.right = Some(index);
        } else {
            panic!("Cannot add child when all children are bound");
        }
    }

    /// Sets the percentage of the node.
    ///
    /// # Panics
    /// If the percentage is not between 0.0 and 1.0
    pub fn set_percent(&mut self, percent: f32) {
        assert!(matches!(percent, 0.0..=1.0));
        self.percent = percent;
    }

    /// Get the percentage of space to give to the first child
    pub fn get_percent(&self) -> f32 {
        self.percent
    }

    /// Get the index of the left child
    ///
    /// # Panics
    /// If the left node is not set
    pub fn get_left_index(&self) -> NodeIndex {
        self.left.as_ref().expect("Left slot not set").shareable()
    }

    /// Get the index of the right child
    ///
    /// # Panics
    /// If the right node is not set
    pub fn get_right_index(&self) -> NodeIndex {
        self.right.as_ref().expect("Right slot not set").shareable()
    }

    /// Get the index of the separator
    ///
    /// # Panics
    /// If the separator node is not set
    pub fn get_sep_index(&self) -> NodeIndex {
        self.sep.as_ref().expect("Sep slot not set").shareable()
    }
}

impl Default for HSplit3 {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode for HSplit3 {
    fn get_align(&self) -> (Alignment, Alignment) {
        self.align
    }

    fn get_align_mut(&mut self) -> (&mut Alignment, &mut Alignment) {
        (&mut self.align.0, &mut self.align.1)
    }

    fn calculate_min_size(&self, tree: &UiTree) -> (f32, f32) {
        let Some(left) = self.left.as_ref().map(OwnedIndex::shareable) else {
            return (0.0, 0.0);
        };

        let Some(right) = self.right.as_ref().map(OwnedIndex::shareable) else {
            return (0.0, 0.0);
        };

        let Some(sep) = self.sep.as_ref().map(OwnedIndex::shareable) else {
            return (0.0, 0.0);
        };

        let left_min = tree
            .get_cache(left)
            .expect("Left child not in cache")
            .min_size;
        let right_min = tree
            .get_cache(right)
            .expect("Right child not in cache")
            .min_size;
        let sep_min = tree
            .get_cache(sep)
            .expect("Sep child not in cache")
            .min_size;

        (
            left_min.0 + sep_min.0 + right_min.0,
            left_min.1.max(right_min.1).max(sep_min.1),
        )
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        let Some(left) = self.left.as_ref().map(OwnedIndex::shareable) else {
            return vec![];
        };

        let Some(right) = self.right.as_ref().map(OwnedIndex::shareable) else {
            return vec![];
        };

        let Some(sep) = self.sep.as_ref().map(OwnedIndex::shareable) else {
            return vec![];
        };

        let left_min = tree
            .get_cache(left)
            .expect("Left child not in cache")
            .min_size;
        let right_min = tree
            .get_cache(right)
            .expect("Right child not in cache")
            .min_size;
        let sep_min = tree
            .get_cache(sep)
            .expect("Sep child not in cache")
            .min_size;

        let left_node = tree.get_node(left).expect("Left child not in cache");
        let right_node = tree.get_node(right).expect("Right child not in cache");
        let sep_node = tree.get_node(sep).expect("Sep child not in cache");

        let width = cache.rect.w - sep_min.0;

        let div_left = (width * self.percent).clamp(left_min.0, width - right_min.0);
        let div_right = width - div_left;
        let x_sep = cache.rect.x + div_left;
        let x_right = x_sep + sep_min.0;

        let space_left = Rect::new(cache.rect.x, cache.rect.y, div_left, cache.rect.h)
            .align(left_node.get_align(), left_min);
        let space_right = Rect::new(x_right, cache.rect.y, div_right, cache.rect.h)
            .align(right_node.get_align(), right_min);
        let space_sep = Rect::new(x_sep, cache.rect.y, sep_min.0, cache.rect.h)
            .align(sep_node.get_align(), sep_min);

        vec![space_left, space_sep, space_right]
    }

    fn get_children(&self) -> Vec<NodeIndex> {
        let Some(left) = self.left.as_ref().map(OwnedIndex::shareable) else {
            return vec![];
        };
        let Some(sep) = self.sep.as_ref().map(OwnedIndex::shareable) else {
            return vec![left];
        };
        let Some(right) = self.right.as_ref().map(OwnedIndex::shareable) else {
            return vec![left, sep];
        };

        vec![left, sep, right]
    }
}

/// Splits the space between 2 children, with a separator, rather than spacing, in between.
///
/// The separator is always shrunk vertically, and every node receives at least its minimum size,
/// but if a node would not receive its minimum size, the percentage is bypassed.
///
/// The percentage excludes the space of the separator. A percentage of 0.0 will shrink the bottom
/// child to its minimum size and give the remaining space to the top, and a percentage of 1.0 will
/// shrink the top child to its minimum size and give the remaining space to the bottom.
pub struct VSplit3 {
    percent: f32,
    align: (Alignment, Alignment),

    top: Option<OwnedIndex>,
    sep: Option<OwnedIndex>,
    bot: Option<OwnedIndex>,
}

impl VSplit3 {
    /// Creates a new vertical split with no children, 50/50 split, and default alignment
    pub fn new() -> Self {
        Self {
            percent: 0.5,
            align: Default::default(),
            top: None,
            sep: None,
            bot: None,
        }
    }

    /// Binds the top, separator, and bottom children to the node.
    ///
    /// # Panics
    /// If any children are already bound.
    pub fn with_children(mut self, top: OwnedIndex, sep: OwnedIndex, bot: OwnedIndex) -> Self {
        assert!(self.top.is_none() || self.sep.is_none() || self.bot.is_none());
        self.top = Some(top);
        self.sep = Some(sep);
        self.bot = Some(bot);
        self
    }

    /// Binds a child to the node. The top slot is set first, then the separator, then the bottom.
    ///
    /// # Panics
    /// If all slots are set.
    pub fn with_child(mut self, index: OwnedIndex) -> Self {
        self.add_child(index);
        self
    }

    /// Sets the percentage of the node.
    ///
    /// # Panics
    /// If the percentage is not between 0.0 and 1.0
    pub fn with_percent(mut self, percent: f32) -> Self {
        self.percent = percent;
        self
    }

    /// Sets the alignment of the node.
    pub fn with_align(mut self, align: (Alignment, Alignment)) -> Self {
        self.align = align;
        self
    }

    /// Binds a child to the node. The top slot is set first, then the separator, then the bottom.
    ///
    /// # Panics
    /// If all slots are set.
    pub fn add_child(&mut self, index: OwnedIndex) {
        if self.top.is_none() {
            self.top = Some(index);
        } else if self.sep.is_none() {
            self.sep = Some(index);
        } else if self.bot.is_none() {
            self.bot = Some(index);
        } else {
            panic!("Cannot add child when all children are bound");
        }
    }

    /// Sets the percentage of the node.
    ///
    /// # Panics
    /// If the percentage is not between 0.0 and 1.0
    pub fn set_percent(&mut self, percent: f32) {
        assert!(matches!(percent, 0.0..=1.0));
        self.percent = percent;
    }

    /// Gets the percentage of the node
    pub fn get_percent(&self) -> f32 {
        self.percent
    }

    /// Returns the tree index of the top node.
    ///
    /// # Panics
    /// If the top node is not set.
    pub fn get_top_index(&self) -> NodeIndex {
        self.top.as_ref().expect("Top child not bound").shareable()
    }

    /// Returns the tree index of the bottom node.
    ///
    /// # Panics
    /// If the bottom node is not set.
    pub fn get_bot_index(&self) -> NodeIndex {
        self.bot
            .as_ref()
            .expect("Bottom child not bound")
            .shareable()
    }

    /// Returns the tree index of the separator node.
    ///
    /// # Panics
    /// If the separator node is not set.
    pub fn get_sep_index(&self) -> NodeIndex {
        self.sep.as_ref().expect("Sep child not bound").shareable()
    }
}

impl Default for VSplit3 {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode for VSplit3 {
    fn get_align(&self) -> (Alignment, Alignment) {
        self.align
    }

    fn get_align_mut(&mut self) -> (&mut Alignment, &mut Alignment) {
        (&mut self.align.0, &mut self.align.1)
    }

    fn calculate_min_size(&self, tree: &UiTree) -> (f32, f32) {
        let Some(top) = self.top.as_ref().map(OwnedIndex::shareable) else {
            return (0.0, 0.0);
        };

        let Some(sep) = self.sep.as_ref().map(OwnedIndex::shareable) else {
            return (0.0, 0.0);
        };

        let Some(bot) = self.bot.as_ref().map(OwnedIndex::shareable) else {
            return (0.0, 0.0);
        };

        let top_min = tree
            .get_cache(top)
            .expect("Top child not in cache")
            .min_size;
        let sep_min = tree
            .get_cache(sep)
            .expect("Sep child not in cache")
            .min_size;
        let bot_min = tree
            .get_cache(bot)
            .expect("Bottom child not in cache")
            .min_size;

        (
            top_min.0.max(sep_min.0).max(bot_min.0),
            top_min.1 + sep_min.1 + bot_min.1,
        )
    }

    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        let Some(top) = self.top.as_ref().map(OwnedIndex::shareable) else {
            return vec![];
        };
        let Some(sep) = self.sep.as_ref().map(OwnedIndex::shareable) else {
            return vec![];
        };
        let Some(bot) = self.bot.as_ref().map(OwnedIndex::shareable) else {
            return vec![];
        };

        let top_min = tree
            .get_cache(top)
            .expect("Top child not in cache")
            .min_size;
        let sep_min = tree
            .get_cache(sep)
            .expect("Sep child not in cache")
            .min_size;
        let bot_min = tree
            .get_cache(bot)
            .expect("Bottom child not in cache")
            .min_size;

        let top_node = tree.get_node(top).expect("Top child not in cache");
        let sep_node = tree.get_node(sep).expect("Sep child not in cache");
        let bot_node = tree.get_node(bot).expect("Bottom child not in cache");

        let height = cache.rect.h - sep_min.1;
        let div_top = (height * self.percent).clamp(top_min.1, height - bot_min.1);
        let div_bot = height - div_top;
        let y_sep = cache.rect.y + div_top;
        let y_bot = y_sep + sep_min.1;

        let top_space = Rect::new(cache.rect.x, cache.rect.y, cache.rect.w, div_top)
            .align(top_node.get_align(), top_min);
        let sep_space = Rect::new(cache.rect.x, y_sep, cache.rect.w, sep_min.1)
            .align(sep_node.get_align(), sep_min);
        let bot_space = Rect::new(cache.rect.x, y_bot, cache.rect.w, div_bot)
            .align(bot_node.get_align(), bot_min);

        vec![top_space, sep_space, bot_space]
    }

    fn get_children(&self) -> Vec<NodeIndex> {
        let Some(top) = self.top.as_ref().map(OwnedIndex::shareable) else {
            return vec![];
        };
        let Some(sep) = self.sep.as_ref().map(OwnedIndex::shareable) else {
            return vec![top];
        };
        let Some(bot) = self.bot.as_ref().map(OwnedIndex::shareable) else {
            return vec![top, sep];
        };

        vec![top, sep, bot]
    }
}
