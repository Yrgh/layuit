//! # A basic UI layout library.
//! 
//! Layuit provides many basic UI components and allows the user to build a UI tree with them.

#![warn(clippy::all)]
#![deny(clippy::unwrap_used)]

use std::collections::VecDeque;

use thunderdome::{Arena, Index as TdIndex};

pub mod containers;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
/// An alignment of any sort, for example determining node placement.
/// 
/// Begin refers to the left or top. End refers to the right or bottom.
pub enum Alignment {
    Begin,
    Center,
    End,
    #[default]
    Full
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
/// A rectangle in space, represented with `f32` coordinates.
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32
}

impl Rect {
    /// Create a new rectangle with the given dimensions and position.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height
        }
    }

    /// Shrink the width of the rectangle by the given amount toward the left.
    pub fn shrink_begin_x(mut self, amount: f32) -> Self {
        self.width -= amount;
        self
    }

    /// Shrink the width of the rectangle by the given amount toward the right.
    pub fn shrink_end_x(mut self, amount: f32) -> Self {
        self.width -= amount;
        self.x += amount;
        self
    }

    /// Shrink the width of the rectangle by the given amount toward the center.
    pub fn shrink_center_x(mut self, amount: f32) -> Self {
        self.width -= amount;
        self.x += amount * 0.5;
        self
    }

    /// Shrink the height of the rectangle by the given amount toward the top.
    pub fn shrink_begin_y(mut self, amount: f32) -> Self {
        self.height -= amount;
        self
    }

    /// Shrink the height of the rectangle by the given amount toward the bottom.
    pub fn shrink_end_y(mut self, amount: f32) -> Self {
        self.height -= amount;
        self.y += amount;
        self
    }

    /// Shrink the height of the rectangle by the given amount toward the middle.
    pub fn shrink_center_y(mut self, amount: f32) -> Self {
        self.height -= amount;
        self.y += amount * 0.5;
        self
    }

    /// Create a contained rectangle aligned within `self`.
    /// 
    /// Example:
    /// 
    /// ```rust
    /// let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
    /// 
    /// let contained_center = rect.align((Alignment::Center, Alignment::Center), 50.0, 50.0);
    /// assert_eq!(contained, Rect::new(25.0, 25.0, 50.0, 50.0));
    /// 
    /// let contained_top_right = rect.align((Alignment::End, Alignment::Begin), 50.0, 50.0);
    /// assert_eq!(contained, Rect::new(50.0, 0.0, 50.0, 50.0));
    /// 
    /// let contained_equal = rect.align((Alignment::Full, Alignment::Full), 50.0, 50.0);
    /// assert_eq!(contained, Rect::new(0.0, 0.0, 100.0, 100.0));
    /// ```
    pub fn align(mut self, align: (Alignment, Alignment), min_w: f32, min_h: f32) -> Self {
        self = match align.0 {
            Alignment::Begin => self.shrink_begin_x(self.width - min_w),
            Alignment::Center => self.shrink_center_x(self.width - min_w),
            Alignment::End => self.shrink_end_x(self.width - min_w),
            Alignment::Full => self
        };

        match align.1 {
            Alignment::Begin => self.shrink_begin_y(self.height - min_h),
            Alignment::Center => self.shrink_center_y(self.height - min_h),
            Alignment::End => self.shrink_end_y(self.height - min_h),
            Alignment::Full => self
        }
    }
}

/// Basic functionality for a UI node.
pub trait UiNode: std::any::Any {
    /// Get the alignment of the node.
    fn get_align(&self) -> (Alignment, Alignment);
    /// Get a mutable reference to the alignment of the node.
    fn get_align_mut(&mut self) -> &mut (Alignment, Alignment);

    /// Calculate the minimum size of the node.
    fn get_min_size(&self) -> (f32, f32);

    /// Get the computed position and size of the node. If it has not been calculated yet, returns
    /// [`Default::default`].
    fn get_rect(&self) -> Rect;

    /// Recalculate the position and size of the node *after* alignment has been applied.
    fn calculate_rect(&mut self, space: Rect);

    /// Get all children of the node, if applicable.
    fn get_children(&self) -> Vec<NodeIndex> {
        Vec::new()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeIndex(TdIndex);

/// A tree of UI nodes.
pub struct UiTree {
    root: TdIndex,
    arena: Arena<Box<dyn UiNode>>
}

impl UiTree {
    /// Create a new UI tree with the given root node.
    pub fn new(root: impl UiNode) -> Self {
        let mut arena = Arena::new();
        let index = arena.insert(Box::new(root) as Box<dyn UiNode>);
        Self {
            root: index,
            arena
        }
    }

    /// Add a node to the arena, although it may not be immediately exist in the tree.
    pub fn add_node(&mut self, node: impl UiNode) -> NodeIndex {
        let index = self.arena.insert(Box::new(node) as Box<dyn UiNode>);
        NodeIndex(index)
    }

    /// Add a pre-boxed node to the arena.
    pub fn add_boxed(&mut self, node: Box<dyn UiNode>) -> NodeIndex {
        let index = self.arena.insert(node);
        NodeIndex(index)
    }
    
    /// Remove a node and all of its descendants from the arena.
    pub fn remove_node(&mut self, index: NodeIndex) {
        let mut queue: VecDeque<_> = self.arena[index.0].get_children().into();
        while let Some(child) = queue.pop_front() {
            queue.extend(self.arena[child.0].get_children());
            self.arena.remove(child.0);
        }
        self.arena.remove(index.0);
    }

    /// Get a reference to a node.
    pub fn get_node(&self, index: NodeIndex) -> &dyn UiNode {
        &*self.arena[index.0]
    }

    /// Get a mutable reference to a node.
    pub fn get_node_mut(&mut self, index: NodeIndex) -> &mut dyn UiNode {
        &mut *self.arena[index.0]
    }

    /// Get the index of the root node.
    pub fn get_root(&self) -> NodeIndex {
        NodeIndex(self.root)
    }
}