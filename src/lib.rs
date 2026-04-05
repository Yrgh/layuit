//! # Layuit
//! 
//! A renderer-agnostic UI layout system. Layuit handles computing the size and position of various
//! [`UiNode`]s in a [`UiTree`]. Layuit does not handle rendering, but provides tools for doing so.
//! 
//! Layuit provides several organizational nodes such as [`HStack`] and [`Margin`], but allows users
//! to create their own nodes.
//! 
//! Layuit uses the [`thunderdome`] crate for the tree structure. To access nodes from a tree, use
//! [`thunderdome::Index`].
//! 
//! ## Core concepts
//! 
//! - **[`UiTree`]**: Owns the [`UiNode`]s and layout information in an arena and handles
//!   computation and access.
//! - **[`UiNode`]**: A trait implemented by all UI nodes, containing alignment and any number of
//!   children.
//! - **[`NodeCache`]**: The cached layout information for a node, produced by
//!   [`UiTree::calculate_layout`].
//! - **[`Rect`]**: A rectangle in space, represented with `f32` coordinates.
//! - **[`Alignment`]**: An alignment primarily used for determining node placement.
//! - **[`NodeVisitor`]**: A trait implemented e.g. by renderers to process and/or manipulate nodes.
//! 
//! ## Layout process
//! 
//! Layout runs in two passes, when [`UiTree::calculate_layout`] is called:
//! 
//! 1. **Bottom-up: minimum size.** Children are visited before their parent. Each node computes its
//!    minimum size based on its children through [`calculate_min_size`] and stores it in its
//!    [`NodeCache::min_size`].
//! 
//! 2. **Top-down: rectangles.** Starting from the root, each node computes the position and size of
//!    its immediate children through [`calculate_rects`]. Each child then uses its restricted
//!    [`Rect`] to do the same for its children. The [`NodeCache::rect`] field is populated with
//!    the resulting [`Rect`]s.
//! 
//! ## Caveats
//! 
//! **The cache is stale before [`UiTree::calculate_layout`] is called**, and becomes stale if
//! children are added, removed, moved, or otherwise changed. The cache always produces valid
//! results, but they may be out of date or set to 0.
//! 
//! **Minimum size is a practice, not a requirement**. When implementing custom nodes, be wary of
//! ensuring each node's minimum size is enforced. This can easily become a problem if the space
//! required by the entire tree is smaller than the one provided to [`UiTree::calculate_layout`].
//! 
//! ## Implementing custom nodes
//! 
//! Custom nodes are essential to using Layuit. Without them, no meaningful UI can be rendered.
//! However, it is important to ensure you follow the rules:
//! 
//! 1. **Children must be accurately reported.** Failure to report children will result in them not
//!    being updated during [`UiTree::calculate_layout`] or removed during [`UiTree::remove_node`].
//! 
//! 2. **Minimum size must be correctly calculated.** Under-representing the minimum size can and
//!    often will result in nodes overflowing into each other.
//! 
//! 3. **Rectangles must be properly assigned.** Similar to #2, it is the responsibility of the
//!    *parent* node to ensure that each node get both enough space and not too much. Failing to do
//!    so will result in nodes overlapping.
//! 
//! One common custom node is the `Label`:
//! 
//! ```rust
//! use layuit::{Alignment, NodeCache, Rect, UiTree, UiNode};
//! 
//! pub struct Label {
//!     text: String,
//!     align: (Alignment, Alignment),
//!     cached_size: (f32, f32),
//! }
//! 
//! /* Label methods and constructors... */
//! 
//! impl UiNode for Label {
//!     fn get_align(&self) -> (Alignment, Alignment) { self.align }
//!     fn get_align_mut(&mut self) -> (&mut Alignment, &mut Alignment) {
//!         (&mut self.align.0, &mut self.align.1)
//!     }
//! 
//!     fn calculate_min_size(&mut self, _tree: &UiTree, _cache: &mut NodeCache) -> (f32, f32) {
//!         self.cached_size
//!     }
//! 
//!     // calculate_rects and get_children are omitted for leaf nodes
//! }
//! ```
//! 
//! ## Creating a tree
//! 
//! Every tree needs a root node, which cannot be removed. Good choices are [`Overlap`] and either
//! [`HStack`] or [`VStack`]. A custom node can also be used.
//! 
//! ```rust
//! use layuit::{UiTree, UiNode, NodeVisitor};
//! use layuit::stacks::HStack;
//! 
//! // The root node can be any UiNode, but must be specified.
//! let mut tree = UiTree::new(HStack::new().with_spacing(4.0));
//! 
//! // Create a label wrapped in a 4px margin
//! let padded_label = Margin::new()
//!     .with_margins(4.0, 4.0, 4.0, 4.0)
//!     .with_child(Label::new("Hello, world!"), &mut tree);
//! 
//! // Add the label to the root stack
//! tree.get_root_mut()
//!     .downcast_mut::<HStack>()
//!     .unwrap()
//!     .with_child(padded_label, &mut tree);
//! 
//! tree.calculate_layout(Rect::new(0.0, 0.0, 640.0, 480.0));
//! 
//! // Render the UI tree
//! 
//! struct Renderer {
//!     // ...
//! }
//! 
//! impl NodeVisitor for Renderer {
//!     fn visit(&mut self, node: &mut dyn UiNode, rect: layuit::Rect) {
//!         if let Some(label) = node.downcast_mut::<Label>() {
//!             // ...
//!         }
//!     }
//! }
//! 
//! let mut renderer = /* ... */;
//! tree.visit(&mut renderer);
//! ```
//! 
//! ## Provided nodes
//! 
//! Containers:
//! - [`HStack`] - Horizontal arrangement
//! - [`VStack`] - Vertical arrangement
//! - [`Overlap`] - Independent arrangement of children
//! - [`Margin`] - Adds padding to a child
//! - [`Minimum`] - Creates a minimum size constraint for precise control
//! 
//! [`calculate_min_size`]: UiNode::calculate_min_size
//! [`calculate_rects`]: UiNode::calculate_rects
//! [`HStack`]: stacks::HStack
//! [`VStack`]: stacks::VStack
//! [`Overlap`]: overlap::Overlap
//! [`Margin`]: padding::Margin
//! [`Minimum`]: padding::Minimum
//! 
//! [`thunderdome`]: https://crates.io/crates/thunderdome
//! [`thunderdome::Index`]: https://docs.rs/thunderdome/latest/thunderdome/struct.Index.html

#![warn(clippy::all)]
#![deny(clippy::unwrap_used)]

use std::collections::{HashMap, VecDeque};

use thunderdome::{Arena, Index as TdIndex};

pub mod stacks;
pub mod padding;
pub mod overlap;

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
    /// assert_eq!(contained_center, Rect::new(25.0, 25.0, 50.0, 50.0));
    ///
    /// let contained_top_right = rect.align((Alignment::End, Alignment::Begin), (50.0, 50.0));
    /// assert_eq!(contained_top_right, Rect::new(50.0, 0.0, 50.0, 50.0));
    ///
    /// let contained_equal = rect.align((Alignment::Full, Alignment::Full), (50.0, 50.0));
    /// assert_eq!(contained_equal, Rect::new(0.0, 0.0, 100.0, 100.0));
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
    fn get_align_mut(&mut self) -> (&mut Alignment, &mut Alignment);

    /// Calculate the minimum size of the node.
    fn calculate_min_size(&self, tree: &UiTree) -> (f32, f32);

    /// Recalculate the position and size of child nodes, in the same order as [`get_children`].
    /// 
    /// This is optional, and should only be implemented if the node has a child/children.
    ///
    /// [`get_children`]: Self::get_children
    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        let _ = (cache, tree);
        vec![]
    }

    /// Get all children of the node, if applicable.
    fn get_children(&self) -> Vec<TdIndex> {
        vec![]
    }
}

impl dyn UiNode {
    /// Downcast a reference to a specific type. See [`Any::downcast_ref`].
    /// 
    /// [`Any::downcast_ref`]: https://doc.rust-lang.org/std/any/trait.Any.html#method.downcast_ref
    pub fn downcast_ref<T: UiNode>(&self) -> Option<&T> {
        (self as &dyn std::any::Any).downcast_ref()
    }

    /// Downcast a mutable reference to a specific type. See [`Any::downcast_mut`].
    /// 
    /// [`Any::downcast_mut`]: https://doc.rust-lang.org/std/any/trait.Any.html#method.downcast_mut
    pub fn downcast_mut<T: UiNode>(&mut self) -> Option<&mut T> {
        (self as &mut dyn std::any::Any).downcast_mut()
    }
}

#[derive(Copy, Clone, Debug, Default)]
/// Cached layout information for a node.
pub struct NodeCache {
    pub min_size: (f32, f32),
    pub rect: Rect,
}

/// A visitor for UI nodes.
pub trait NodeVisitor {
    /// Visit a node.
    fn visit(&mut self, node: &mut dyn UiNode, rect: Rect);
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
    /// If the index is invalid or the tree is malformed.
    /// 
    /// Also panics if the root node is removed.
    pub fn remove_node(&mut self, index: TdIndex) {
        assert_ne!(index, self.root, "Root node cannot be removed");
        
        let mut queue: VecDeque<_> = self.arena[index].get_children().into();
        while let Some(child) = queue.pop_front() {
            queue.extend(self.arena[child].get_children());
            self.arena.remove(child);
            self.cache.remove(&child);
        }
        self.arena.remove(index);
        self.cache.remove(&index);
    }

    /// Get a reference to the cached layout information for a node.
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
    /// Returns `true` if `root_rect` preserves minimum size requirements. If the given space is too
    /// small, `false` is returned, but the cache will be updated with potentially incorrect
    /// results.
    /// 
    /// # Panics
    /// If the tree is malformed
    pub fn calculate_layout(&mut self, root_rect: Rect) -> bool {
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

        let is_good = root_rect.width >= self.cache[&self.root].min_size.0
            && root_rect.height >= self.cache[&self.root].min_size.1;

        self.cache
            .entry(self.root)
            .and_modify(|e| e.rect = root_rect);

        for v in min_stack {
            let rects = self.arena[v].calculate_rects(&self.cache[&v], self);
            for (child, rect) in self.arena[v].get_children().iter().zip(rects) {
                self.cache.entry(*child).and_modify(|e| e.rect = rect);
            }
        }

        is_good
    }

    /// Visit the nodes in the tree, in descending order, calling the visitor for each node.
    /// 
    /// Nodes are visited in the order they appear in [`get_children`], depth-first.
    /// 
    /// [`get_children`]: UiNode::get_children
    pub fn visit(&mut self, visitor: &mut impl NodeVisitor) {
        let mut queue = VecDeque::from([self.root]);
        while let Some(idx) = queue.pop_front() {
            let node = &mut *self.arena[idx];
            let rect = self.cache[&idx].rect;
            visitor.visit(node, rect);
            queue.extend(node.get_children());
        }
    }
}
