//! # Layuit
//!
//! A renderer-agnostic UI layout system. Layuit handles computing the size and position of various
//! [`UiNode`]s in a [`UiTree`]. Layuit does not handle rendering, but provides tools for doing so.
//!
//! Layuit provides several organizational nodes such as [`HStack`] and [`Margin`], but allows users
//! to create their own nodes.
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
//! The majority of the layout process can be thought of as drawing boxes on a sheet of paper. Boxes
//! cannot normally cross, but are allowed to touch. Several nodes change that behavior:
//!
//! - [`Overlap`] intentionally allows the boxes of children to overlap each other, as long as they
//!   stay inside.
//!
//! - [`Clip`] allows a box to be bigger than that of its parent, but requires the box's viewable
//!   area to be constrained to that of the `Clip` by the renderer.
//!
//! - [`Hider`] allows a child to be completely excluded from the layout process, but requires the
//!   renderer to ignore it.
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
//! Probably the most important custom node is the `Label`:
//!
//! ```rust
//! use layuit::{Alignment, NodeCache, Rect, UiTree, UiNode};
//!
//! pub struct Label {
//!     text: String,
//!     cached_size: (f32, f32),
//!
//!     align: (Alignment, Alignment),
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
//!     fn calculate_min_size(&self, _tree: &UiTree) -> (f32, f32) {
//!         self.cached_size
//!     }
//!
//!     // calculate_rects and get_children are omitted for leaf nodes
//! }
//! ```
//!
//! However, you are not restricted to just leaf nodes. You can create containers.
//!
//! ## Using the `ui!` macro
//!
//! The [`ui!`] macro is a convenience for creating trees with a simple syntax, avoiding rewriting
//! `.with_align((...))` and `.with_child(...)` in every node, for every child. It does come with
//! the limitation that you cannot create your entire tree this way; your root node must be created
//! manually.
//!
//! Additionally, you can create variables outside the macro and assign the indices of nodes created
//! by the macro to them.
//!
//! ```rust
//! use layuit::UiTree;
//! use layuit::padding::{Spacer, Minimum};
//! use layuit::stacks::HStack;
//! use layuit::proportion::HSplit;
//! use thunderdome::Index;
//!
//! let spacer3;
//! let (node_index, mut tree) = layuit::ui!(
//!     %%,
//!     +|+ HStack::new() => [
//!         -|< Spacer::sized((10.0, 10.0)),
//!         -|- Minimum::new().with_min((20.0, 20.0)) => [
//!             -|- Spacer::sized((10.0, 10.0))
//!         ],
//!         spacer3 = -|> Spacer::sized((10.0, 10.0))
//!     ]
//! );
//!
//! tree
//!     .get_node_mut(spacer3)
//!     .unwrap()
//!     .downcast_mut::<Spacer>()
//!     .unwrap()
//!     .set_size((20.0, 20.0));
//!
//!
//! // HStack (Full, Full) = node_index / tree root
//! // ├─ Spacer (N/A, Begin)
//! // ├─ Minimum (N/A, Center)
//! // │  └─ Spacer (Center, Center)
//! // └─ Spacer (N/A, End) = spacer3
//! ```
//!
//! ## Provided nodes
//!
//! - [`HStack`] - Horizontal arrangement
//! - [`VStack`] - Vertical arrangement
//! - [`Overlap`] - Independent arrangement of children
//! - [`Margin`] - Adds padding to a child
//! - [`Minimum`] - Creates a minimum size constraint for precise control
//! - [`Spacer`] - A leaf node with configurable empty space
//! - [`Clip`] - Allows a child to outgrow the node with the assumption that the renderer will
//!   clip it, and enables a scroll offset to be applied if the child is larger.
//! - [`Hider`] - Allows a child's visibility to be controlled. An invisible node has no minimum
//!   size and should not be attempted to be rendered.
//! - [`Selector`] - Selects a single child node to be visible at a time.
//! - [`AspectRatio`] - Maintains a horizontal:vertical ratio.
//! - [`HSplit`] - Horizontal split between two children.
//! - [`VSplit`] - Vertical split between two children.
//! - [`Percent`] - Maintains a percentage of space for a child.
//! - [`HEqual`] - Horizontal arrangement with each child getting equal space.
//! - [`VEqual`] - Vertical arrangement with each child getting equal space.
//! - [`Grid`] - 2D grid of children.
//! - [`Clamp`] - Constrains a child to a maximum size.
//! - [`HSplit3`] - Horizontal split between two children, with a third child separating the two.
//! - [`VSplit3`] - Vertical split between two children, with a third child separating the two.
//!
//! [`calculate_min_size`]: UiNode::calculate_min_size
//! [`calculate_rects`]: UiNode::calculate_rects
//! [`HStack`]: stacks::HStack
//! [`VStack`]: stacks::VStack
//! [`Overlap`]: overlap::Overlap
//! [`Margin`]: padding::Margin
//! [`Minimum`]: padding::Minimum
//! [`Spacer`]: padding::Spacer
//! [`Clip`]: clip::Clip
//! [`Hider`]: visibility::Hider
//! [`Selector`]: visibility::Selector
//! [`AspectRatio`]: proportion::AspectRatio
//! [`HSplit`]: proportion::HSplit
//! [`VSplit`]: proportion::VSplit
//! [`Percent`]: proportion::Percent
//! [`HEqual`]: grid::HEqual
//! [`VEqual`]: grid::VEqual
//! [`Grid`]: grid::Grid
//! [`Clamp`]: limit::Clamp
//! [`HSplit3`]: split3::HSplit3
//! [`VSplit3`]: split3::VSplit3

#![warn(clippy::all)]
#![deny(clippy::unwrap_used)]

use std::collections::{HashMap, VecDeque};

use thunderdome::Arena;

pub mod clip;
pub mod grid;
pub mod limit;
pub mod overlap;
pub mod padding;
pub mod prelude;
pub mod proportion;
pub mod split3;
pub mod stacks;
pub mod visibility;

pub use thunderdome::Index as NodeIndex;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
/// An alignment of any sort, for example determining node placement.
///
/// `Begin`, `Center`, and `End` cause a node to shrink to its minimum size in to that position.
/// `Full` causes a node to occupy all space it is given. For example, to shrink to the left-middle,
/// use (`Begin`, `Center`). To fill horizontally and shrink down, use (`Full`, `End`).
pub enum Alignment {
    #[default]
    /// The left or top.
    Begin,
    Center,
    End,
    Full,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
/// Horizontal or vertical anchoring. While very similar to [`Alignment`], `Anchor` represents
/// shrinking only, and has no `Full` variant.
///
/// It is also noteworthy that the default is `Center`, not `Alignment::Begin`.
pub enum Anchor {
    Begin,
    #[default]
    Center,
    End,
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
/// A rectangle in space, represented with `f32` coordinates.
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    /// Create a new rectangle with the given dimensions and position.
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// Identical to [`new`], but returns `None` if either the width or height is negative.
    ///
    /// [`new`]: Self::new
    pub fn new_checked(x: f32, y: f32, w: f32, h: f32) -> Option<Self> {
        if w >= 0.0 && h >= 0.0 {
            Some(Self::new(x, y, w, h))
        } else {
            None
        }
    }

    /// Returns `false` if either the width or height is negative. Otherwise, returns `true`.
    pub fn is_valid(&self) -> bool {
        self.w >= 0.0 && self.h >= 0.0
    }

    /// Returns `true` if the rectangle has no width *or* no height.
    pub fn is_empty(&self) -> bool {
        self.w == 0.0 || self.h == 0.0
    }

    /// Returns `true` if the rectangle has no width *and* no height.
    pub fn is_zero(&self) -> bool {
        self.w == 0.0 && self.h == 0.0
    }

    /// Shrink the width of the rectangle by the given amount toward the left.
    pub fn shrink_begin_x(mut self, amount: f32) -> Self {
        self.w -= amount;
        self
    }

    /// Shrink the width of the rectangle by the given amount toward the right.
    pub fn shrink_end_x(mut self, amount: f32) -> Self {
        self.w -= amount;
        self.x += amount;
        self
    }

    /// Shrink the width of the rectangle by the given amount toward the center.
    pub fn shrink_center_x(mut self, amount: f32) -> Self {
        self.w -= amount;
        self.x += amount * 0.5;
        self
    }

    /// Shrink the height of the rectangle by the given amount toward the top.
    pub fn shrink_begin_y(mut self, amount: f32) -> Self {
        self.h -= amount;
        self
    }

    /// Shrink the height of the rectangle by the given amount toward the bottom.
    pub fn shrink_end_y(mut self, amount: f32) -> Self {
        self.h -= amount;
        self.y += amount;
        self
    }

    /// Shrink the height of the rectangle by the given amount toward the middle.
    pub fn shrink_center_y(mut self, amount: f32) -> Self {
        self.h -= amount;
        self.y += amount * 0.5;
        self
    }

    /// Create a contained rectangle aligned within `self`.
    ///
    /// Example:
    ///
    /// ```rust
    /// use layuit::{Rect, Alignment};
    ///
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
            Alignment::Begin => self.shrink_begin_x(self.w - min.0),
            Alignment::Center => self.shrink_center_x(self.w - min.0),
            Alignment::End => self.shrink_end_x(self.w - min.0),
            Alignment::Full => self,
        };

        match align.1 {
            Alignment::Begin => self.shrink_begin_y(self.h - min.1),
            Alignment::Center => self.shrink_center_y(self.h - min.1),
            Alignment::End => self.shrink_end_y(self.h - min.1),
            Alignment::Full => self,
        }
    }

    /// Similar to [`align`], but based on [`Anchor`] instead of [`Alignment`].
    ///
    /// [`align`]: Self::align
    pub fn anchor(mut self, anchor: (Anchor, Anchor), size: (f32, f32)) -> Self {
        self = match anchor.0 {
            Anchor::Begin => self.shrink_begin_x(self.w - size.0),
            Anchor::Center => self.shrink_center_x(self.w - size.0),
            Anchor::End => self.shrink_end_x(self.w - size.0),
        };

        match anchor.1 {
            Anchor::Begin => self.shrink_begin_y(self.h - size.1),
            Anchor::Center => self.shrink_center_y(self.h - size.1),
            Anchor::End => self.shrink_end_y(self.h - size.1),
        }
    }

    /// Returns the area included by both `self` and `other`.
    pub fn intersect(self, other: Self) -> Self {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);

        let x2 = (self.x + self.w).min(other.x + other.w);
        let y2 = (self.y + self.h).min(other.y + other.h);

        Self::new(x1, y1, (x2 - x1).max(0.0), (y2 - y1).max(0.0))
    }
}

#[derive(Copy, Clone, Debug, Default)]
/// Cached layout information for a node.
pub struct NodeCache {
    pub min_size: (f32, f32),
    pub rect: Rect,
}

/// Basic functionality for a UI node.
pub trait UiNode: std::any::Any {
    /// Get the alignment of the node.
    fn get_align(&self) -> (Alignment, Alignment);
    /// Get a mutable reference to the alignment of the node.
    fn get_align_mut(&mut self) -> (&mut Alignment, &mut Alignment);

    /// Calculate the minimum size of the node.
    ///
    /// It is the parent's responsibility to ensure the minimum size of all children is met.
    fn calculate_min_size(&self, tree: &UiTree) -> (f32, f32);

    /// Recalculate the position and size of child nodes, in the same order and count as
    /// [`get_visible_children`].
    ///
    /// Parents control the space allocated to their children. It is not the child's responsibility
    /// to manage its own rects. The parent **must** apply the correct alignment to all children,
    /// no matter what kind of container they are.
    ///
    /// A good mental model is to draw boxes on a piece of paper. Each node is a box. A box must be
    /// entirely contained within another. With exceptions, no line can cross through any other.
    /// They may touch and run parallel, but not cross.
    ///
    /// This is optional, and should only be implemented if the node has a child/children.
    ///
    /// [`get_visible_children`]: Self::get_visible_children
    fn calculate_rects(&self, cache: &NodeCache, tree: &UiTree) -> Vec<Rect> {
        let _ = (cache, tree);
        vec![]
    }

    /// Get all children of the node, if applicable.
    fn get_children(&self) -> Vec<NodeIndex> {
        vec![]
    }

    /// Get all **visible** children of the node, if applicable.
    ///
    /// If a node has children, but does not control visibility, this defaults to the result of
    /// [`get_children`].
    ///
    /// This does not have to have the same order as [`get_children`], but it must be a subset.
    ///
    /// [`get_children`]: Self::get_children
    fn get_visible_children(&self) -> Vec<NodeIndex> {
        self.get_children()
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

/// A walker for a UI tree.
pub trait UiWalker {
    /// Called when a node is visited, before its children.
    fn enter(&mut self, node: &mut dyn UiNode, rect: Rect, index: NodeIndex);

    /// Called after all children of a node have been visited, including if it has no
    /// children.
    fn leave(&mut self, node: &mut dyn UiNode, rect: Rect, index: NodeIndex);
}

#[derive(Default)]
pub struct CalculateLayoutConfig {
    /// If `true`, applies the root node's alignment instead of ignoring it.
    pub align_root: bool,
    /// If `true`, ensures that the minimum size of the root node is met, even if that would exceed
    /// the provided rect.
    pub force_good: bool
}

/// A tree of UI nodes, stored as an arena.
pub struct UiTree {
    root: NodeIndex,
    arena: Arena<Box<dyn UiNode>>,
    cache: HashMap<NodeIndex, NodeCache>,
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
    pub fn add_node(&mut self, node: impl UiNode) -> NodeIndex {
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
    pub fn remove_node(&mut self, index: NodeIndex) {
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
    pub fn get_cache(&self, index: NodeIndex) -> Option<&NodeCache> {
        self.cache.get(&index)
    }

    /// Get a reference to a node.
    pub fn get_node(&self, index: NodeIndex) -> Option<&dyn UiNode> {
        self.arena.get(index).map(|node| &**node)
    }

    /// Get a mutable reference to a node.
    pub fn get_node_mut(&mut self, index: NodeIndex) -> Option<&mut dyn UiNode> {
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

    /// Calculate the layout information for all nodes in the tree. This is equivalent to calling
    /// [`calculate_layout_ex`] with the default configuration.
    ///
    /// Returns `true` if `root_rect` preserves minimum size requirements. If the given
    /// space is too small, `false` is returned, but the cache will still be updated. The root node
    /// will be treated as having ([`Full`], [`Full`]) alignment.
    ///
    /// Nodes that are not visible will be given a minimum size of `(0, 0)`.
    ///
    /// # Panics
    /// If the tree is malformed
    /// 
    /// [`calculate_layout_ex`]: Self::calculate_layout_ex
    /// [`Full`]: Alignment::Full
    pub fn calculate_layout(&mut self, root_rect: Rect) -> bool {
        self.calculate_layout_ex(root_rect, Default::default())
    }

    /// Calculate the layout information for all nodes in the tree, with additional control of the
    /// process.
    /// 
    /// If `force_good` is `true`, ensures that the minimum size of the root node is met, even if
    /// that would exceed the provided rect. If `false`, returns whether the minimum size of the
    /// root node is met.
    /// 
    /// If `align_root` is `true`, applies the root node's alignment instead of ignoring it.
    /// 
    /// # Panics
    /// If the tree is malformed
    pub fn calculate_layout_ex(&mut self, root_rect: Rect, config: CalculateLayoutConfig) -> bool {
        // Clear cache
        for v in self.cache.values_mut() {
            *v = Default::default();
        }

        // Queue to visit now
        let mut visit_stack = vec![self.root];
        // Queue to visit later
        let mut min_stack = Vec::new();
        while let Some(v) = visit_stack.pop() {
            min_stack.push(v);
            visit_stack.extend(self.arena[v].get_visible_children());
        }

        for v in min_stack.iter().rev() {
            let min = self.arena[*v].calculate_min_size(self);
            self.cache.entry(*v).or_default().min_size = min;
        }

        let (is_good, root_rect) = if config.force_good {
            let root_min = self.cache[&self.root].min_size;
            (true, Rect::new(0.0, 0.0, root_min.0.max(root_rect.w), root_min.1.max(root_rect.h)))
        } else {
            let is_good = root_rect.w >= self.cache[&self.root].min_size.0
                && root_rect.h >= self.cache[&self.root].min_size.1;

            (is_good, root_rect)
        };

        if config.align_root {
            let align = self.arena[self.root].get_align();
            let min = self.cache[&self.root].min_size;
            self.cache
                .entry(self.root)
                .and_modify(|e| e.rect = root_rect.align(align, min));
        } else {
            self.cache
                .entry(self.root)
                .and_modify(|e| e.rect = root_rect);
        }

        for v in min_stack {
            let rects = self.arena[v].calculate_rects(&self.cache[&v], self);
            for (child, rect) in self.arena[v].get_children().iter().zip(rects) {
                self.cache.entry(*child).or_default().rect = rect;
            }
        }

        is_good
    }

    /// Walks the entire tree, starting from the root, with the given walker. See
    /// [`walk_node`].
    ///
    /// If `use_visible` is `true`, only nodes that are visible will be visited.
    ///
    /// [`walk_node`]: Self::walk_node
    pub fn walk_root(&mut self, walker: &mut impl UiWalker, use_visible: bool) {
        self.walk_node(self.root, walker, use_visible);
    }

    /// Walks a single node and its children, with the given walker.
    ///
    /// First, the walker receives [`enter`] with the node and its cached rect and index.
    /// Then, any and all children are walked in the order returned by
    #[doc = concat!("[`", stringify!(UiNode), "::get_visible_children`]")]
    /// . Finally, the walker receives [`leave`].
    ///
    /// Parents are always visited before their children.
    ///
    /// *Every* call to [`enter`] **will** be matched with a call to [`leave`].
    ///
    /// If `use_visible` is `true`, only nodes that are visible will be visited.
    ///
    /// [`enter`]: UiWalker::enter
    /// [`leave`]: UiWalker::leave
    pub fn walk_node(
        &mut self,
        index: NodeIndex,
        walker: &mut impl UiWalker,
        use_visible: bool
    ) {
        let rect = self.cache[&index].rect;
        walker.enter(self.arena[index].as_mut(), rect, index);

        let children = if use_visible {
            self.arena[index].get_visible_children()
        } else {
            self.arena[index].get_children()
        };
        for child in children {
            self.walk_node(child, walker, use_visible);
        }

        walker.leave(self.arena[index].as_mut(), rect, index);
    }
}

/// A [`UiTree`] that does not have a full tree structure assigned.
/// 
/// The API for this struct only contains a method to add a node to the tree and a method to
/// complete the tree. This comes at the benefit of not requiring a root node at
/// construction.
/// 
/// It is used internally by the [`ui!`] macro, but can also be used manually.
pub struct PartialTree {
    arena: Arena<Box<dyn UiNode>>,
}

impl PartialTree {
    /// Constructs a new empty tree.
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
        }
    }

    /// Adds a node to the tree and returns its index.
    pub fn add_node(&mut self, node: impl UiNode) -> NodeIndex {
        self.arena.insert(Box::new(node) as Box<dyn UiNode>)
    }

    /// Completes a partial tree into a full UI tree.
    ///
    /// Returns a new UI tree with the given root node and the arena and cache from `self`.
    /// 
    /// Indices are preserved.
    ///
    /// # Panics
    /// If the tree is malformed, or if the root node is not present in the arena.
    pub fn complete(self, root: NodeIndex) -> UiTree {
        let cache = self.arena.iter().map(|(id, _)| (id, Default::default())).collect();
        UiTree {
            arena: self.arena,
            root,
            cache,
        }
    }
}

impl Default for PartialTree {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
/// A macro for making the process of creating a UI subtree easier.
///
/// The macro takes a mutable reference to the tree, or `%%`, signifying to create one, and a base
/// node, and returns the index of the base node in the tree. If a tree is created, the index is
/// stored as the first element of a tuple and the tree is the second.
///
/// Each node is represented by two alignment symbols, separated by a `|`, a constructor for the
/// node, and an optional list of children, contained in square brackets after `=>`.
///
/// The alignment characters are orderer horizontal then vertical, with the following allowed:
///
/// `+` - [`Full`]
///
/// `-` - [`Center`]
///
/// `<` - [`Begin`]
///
/// `>` - [`End`]
///
/// Additionally, before a node's alignment, you may write `name =` to assign the index of the node
/// to a variable.
///
/// ## Example usage
/// ```rust
/// use layuit::padding::{Spacer, Minimum};
/// use layuit::stacks::HStack;
/// 
/// let minimum;
/// let (_, tree) = layuit::ui!(
///     %%,
///     +|+ HStack::new() => [
///         -|< Spacer::sized((10.0, 10.0)),
///         minimum = -|- Minimum::new().with_min((20.0, 20.0)) => [
///             -|- Spacer::sized((10.0, 10.0))
///         ],
///         -|> Spacer::sized((10.0, 10.0))
///     ]
/// );
///
/// // Resulting tree:
/// // HStack (Full, Full)
/// // ├─ Spacer (Center, Begin)
/// // ├─ Minimum (Center, Center) = minimum
/// // │  └─ Spacer (Center, Center)
/// // └─ Spacer (Center, End)
/// ```
///
/// [`Begin`]: crate::Alignment::Begin
/// [`Center`]: crate::Alignment::Center
/// [`End`]: crate::Alignment::End
/// [`Full`]: crate::Alignment::Full
macro_rules! ui {
    ($tree:expr, $($node:tt)*) => {
        $crate::ui!(@@_node $tree;; $($node)*)
    };

    (%%, $($node:tt)*) => {
        {
            let mut tree = $crate::PartialTree::new();
            let root = $crate::ui!(@@_node &mut tree;; $($node)*);
            (root, tree.complete(root))
        }
    };

    ($($random:tt)*, $($node:tt)*) => {
        compile_error!(concat!(
            "Expected either a mutable reference to a tree or `%%`, found `",
            stringify!($($random)*),
            "` where the tree should have been."
        ));
    };

    // With binding, no children
    (@@_node $tree:expr;; $binding:ident = $ha:tt | $va:tt $node:expr) => {
        {
            let __node_idx = $tree.add_node($crate::ui!(@@_align $ha | $va $node));
            $binding = __node_idx;
            __node_idx
        }
    };

    // With binding, with children
    (@@_node $tree:expr;; $binding:ident = $ha:tt | $va:tt $node:expr => [ $($child:tt)* ]) => {
        {
            let __ui_node = $crate::ui!(
                @@_child
                $tree;;
                $crate::ui!(@@_align $ha | $va $node)
                => [ $($child)* ]
            );
            let __node_idx = $tree.add_node(__ui_node);
            $binding = __node_idx;
            __node_idx
        }
    };

    // Without binding, without children
    (@@_node $tree:expr;; $ha:tt | $va:tt $node:expr) => {
        {
            let __node_idx = $tree.add_node($crate::ui!(@@_align $ha | $va $node));
            __node_idx
        }
    };

    // Without binding, with children
    (@@_node $tree:expr;; $ha:tt | $va:tt $node:expr => [ $($child:tt)* ]) => {
        {
            let __ui_node = $crate::ui!(
                @@_child
                $tree;;
                $crate::ui!(@@_align $ha | $va $node)
                => [ $($child)* ]
            );
            let __node_idx = $tree.add_node(__ui_node);
            __node_idx
        }
    };

    // No children
    (@@_child $tree:expr;; $node:expr => [ ]) => {
        $node
    };

    // Children, with binding
    (
        @@_child
        $tree:expr;;
        $node:expr
        => [
            $binding:ident =
            $ha:tt |
            $va:tt
            $child:expr
            $(=> [ $($grand:tt)* ])?
            $(, $($rest:tt)*)?
        ]
    ) => {
        {
            let __node_idx = $crate::ui!(
                @@_node
                $tree;;
                $binding =
                $ha |
                $va
                $child
                $(=> [ $($grand)* ])?
            );

            $crate::ui!(
                @@_child
                $tree;;
                $node.with_child(__node_idx)
                => [ $($($rest)*)? ]
            )
        }
    };

    // Children, no binding
    (
        @@_child
        $tree:expr;;
        $node:expr
        => [
            $ha:tt |
            $va:tt
            $child:expr
            $(=> [ $($grand:tt)* ])?
            $(, $($rest:tt)*)?
        ]
    ) => {
        {
            let __node_idx = $crate::ui!(
                @@_node
                $tree;;
                $ha |
                $va
                $child
                $(=> [ $($grand)* ])?
            );

            $crate::ui!(
                @@_child
                $tree;;
                $node.with_child(__node_idx)
                => [ $($($rest)*)? ]
            )
        }
    };

    (@@_align $ha:tt | $va: tt $node:expr) => {
        $node.with_align(($crate::ui!(@@_align $ha), $crate::ui!(@@_align $va)))
    };

    (@@_align <) => {
        $crate::Alignment::Begin
    };

    (@@_align -) => {
        $crate::Alignment::Center
    };

    (@@_align >) => {
        $crate::Alignment::End
    };

    (@@_align +) => {
        $crate::Alignment::Full
    };

    (@@_align $a:tt) => {
        compile_error!("Invalid alignment. Only `<`, `-`, `>`, and `+` are allowed.")
    };

    (@@_node $tree:expr;; $($tt:tt)*) => {
        compile_error!(concat!(
            "Invalid node syntax. A node should have an optional assignment, two alignment \
            characters, separated by a pipe, and a node expression, optionally followed by \
            a fat arrow and children in square brackets.\nExample:\n\
            `target = -|< Node::new() => [ ]`\nFound: `",
            $(stringify!($tt),)*
            "`."
        ))
    };

    (@@_child $tree:expr;; $node:expr => [ $($tt:tt)* ]) => {
        compile_error!(
            "Invalid child node syntax. A node should have an optional assignment, two alignment \
            characters, separated by a pipe, and a node expression, optionally followed by \
            a fat arrow and children in square brackets."
        )
    };

    ($($tt:tt)*) => {
        compile_error!(concat!(
            "Expected a tree followed by a `,`, followed by a node, found `",
            stringify!($($tt)*),
            "`."
        ))
    };
}