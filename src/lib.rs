//! # A basic UI layout library.
//!
//! Layuit provides many basic UI components and allows the user to build a UI tree with them.

#![warn(clippy::all)]
#![deny(clippy::unwrap_used)]

use std::collections::{HashMap, VecDeque};

use thunderdome::{Arena, Index as TdIndex};

pub mod stacks;
pub mod padding;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
/// An alignment of any sort, for example determining node placement.
///
/// Begin refers to the left or top. End refers to the right or bottom.
pub enum Alignment {
    Begin,
    Center,
    End,
    #[default]
    Full,
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
/// A rectangle in space, represented with `f32` coordinates.
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Create a new rectangle with the given dimensions and position.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
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
    /// let contained_center = rect.align((Alignment::Center, Alignment::Center), (50.0, 50.0));
    /// assert_eq!(contained, Rect::new(25.0, 25.0, 50.0, 50.0));
    ///
    /// let contained_top_right = rect.align((Alignment::End, Alignment::Begin), (50.0, 50.0));
    /// assert_eq!(contained, Rect::new(50.0, 0.0, 50.0, 50.0));
    ///
    /// let contained_equal = rect.align((Alignment::Full, Alignment::Full), (50.0, 50.0));
    /// assert_eq!(contained, Rect::new(0.0, 0.0, 100.0, 100.0));
    /// ```
    pub fn align(mut self, align: (Alignment, Alignment), min: (f32, f32)) -> Self {
        self = match align.0 {
            Alignment::Begin => self.shrink_begin_x(self.width - min.0),
            Alignment::Center => self.shrink_center_x(self.width - min.0),
            Alignment::End => self.shrink_end_x(self.width - min.0),
            Alignment::Full => self,
        };

        match align.1 {
            Alignment::Begin => self.shrink_begin_y(self.height - min.1),
            Alignment::Center => self.shrink_center_y(self.height - min.1),
            Alignment::End => self.shrink_end_y(self.height - min.1),
            Alignment::Full => self,
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
    fn calculate_min_size(&self, tree: &UiTree) -> (f32, f32);

    /// Recalculate the position and size of child nodes, in the same order as [`get_children`].
    ///
    /// [`get_children`]: Self::get_children
    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect>;

    /// Get all children of the node, if applicable.
    fn get_children(&self) -> Vec<TdIndex> {
        Vec::new()
    }
}

#[derive(Copy, Clone, Debug, Default)]
/// Cached layout information for a node.
pub struct NodeCache {
    pub min_size: (f32, f32),
    pub rect: Rect,
}

/// A tree of UI nodes, stored as an arena.
pub struct UiTree {
    root: TdIndex,
    arena: Arena<Box<dyn UiNode>>,
    cache: HashMap<TdIndex, NodeCache>,
}

impl UiTree {
    /// Create a new UI tree with the given root node.
    pub fn new(root: impl UiNode) -> Self {
        let mut arena = Arena::new();
        let index = arena.insert(Box::new(root) as Box<dyn UiNode>);
        Self {
            root: index,
            arena,
            cache: HashMap::new(),
        }
    }

    /// Add a node to the arena.
    pub fn add_node(&mut self, node: impl UiNode) -> TdIndex {
        let index = self.arena.insert(Box::new(node) as Box<dyn UiNode>);
        self.cache.insert(index, Default::default());
        index
    }

    /// Remove a node and all of its children from the arena.
    /// 
    /// # Panics
    /// If the index is invalid or the tree is malformed. Removing the root node also panics.
    pub fn remove_node(&mut self, index: TdIndex) {
        if index == self.root {
            panic!("Cannot remove root node");
        }
        
        let mut queue: VecDeque<_> = self.arena[index].get_children().into();
        while let Some(child) = queue.pop_front() {
            queue.extend(self.arena[child].get_children());
            self.arena.remove(child);
            self.cache.remove(&index);
        }
        self.arena.remove(index);
        self.cache.remove(&index);
    }

    pub fn get_cache(&self, index: TdIndex) -> Option<&NodeCache> {
        self.cache.get(&index)
    }

    /// Get a mutable reference to the cached layout information for a node.
    pub fn get_cache_mut(&mut self, index: TdIndex) -> Option<&mut NodeCache> {
        self.cache.get_mut(&index)
    }

    /// Get a reference to a node.
    pub fn get_node(&self, index: TdIndex) -> Option<&dyn UiNode> {
        self.arena.get(index).map(|node| &**node)
    }

    /// Get a mutable reference to a node.
    pub fn get_node_mut(&mut self, index: TdIndex) -> Option<&mut dyn UiNode> {
        self.arena.get_mut(index).map(|node| &mut **node)
    }

    /// Get a reference to the root node.
    pub fn get_root(&self) -> &dyn UiNode {
        &**self.arena.get(self.root).expect("Root not valid")
    }

    /// Get a mutable reference to the root node.
    pub fn get_root_mut(&mut self) -> &mut dyn UiNode {
        &mut **self.arena.get_mut(self.root).expect("Root not valid")
    }

    /// Calculate the layout information for all nodes in the tree.
    /// 
    /// # Panics
    /// If the tree is malformed
    pub fn calculate_cache(&mut self) {
        // Queue to visit now
        let mut visit_stack = vec![self.root];
        // Queue to visit later
        let mut min_stack = Vec::new();
        while let Some(v) = visit_stack.pop() {
            min_stack.push(v);
            visit_stack.extend(self.arena[v].get_children());
        }

        for v in min_stack.iter().rev() {
            let min = self.arena[*v].calculate_min_size(self);
            self.cache.entry(*v).and_modify(|e| e.min_size = min);
        }

        for v in min_stack {
            let rects = self.arena[v].calculate_rects(&self.cache[&v], self);
            for (child, rect) in self.arena[v].get_children().iter().zip(rects) {
                self.cache.entry(*child).and_modify(|e| e.rect = rect);
            }
        }
    }
}
