//! # Layuit
//!
//! A renderer-agnostic UI layout system. Layuit handles computing the size and position of various
//! [`UiNode`]s in a [`UiTree`]. Layuit does not handle rendering, but provides tools to define a
//! renderer.
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
//! - **[`Rect`]**: A rectangle in space, represented with `f32` coordinates.
//! - **[`Alignment`]**: An alignment primarily used for determining node placement.
//! - **[`NodeIndex`]**: A tree index, used to access the [`UiNode`]s in a [`UiTree`], but does not
//!   have "ownership".
//! - **[`OwnedIndex`]**: A non-duplicable tree index, used to refer to children, with "ownership".
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
//! **No threads or async.** Due to using `dyn UiNode` and [`UiNode`] not requiring `Send + Sync`,
//! [`UiTree`] and [`PartialTree`] are `!Send + !Sync`.
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
//! use layuit::{Alignment, UiTree, UiNode};
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
//! use layuit::padding::{Spacer, Minimum};
//! use layuit::stack::HStack;
//!
//! let spacer3;
//! let mut tree = layuit::ui!(
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
//!     .get_cast_mut::<Spacer>(spacer3)
//!     .unwrap()
//!     .set_size((20.0, 20.0));
//!
//!
//! // HStack (Full, Full)
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
//! [`HStack`]: stack::HStack
//! [`VStack`]: stack::VStack
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
//!
//! [`mpsc`]: https://doc.rust-lang.org/nightly/std/sync/mpsc/index.html

#![warn(clippy::all, missing_docs)]
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
pub mod stack;
pub mod visibility;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
/// An index to a node in a tree.
pub struct NodeIndex(thunderdome::Index);

#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
/// An index to a node in a tree, with the caveat that it represents a node not already in the
/// hierarchy.
///
/// To prevent tree malformation, this index is produced by [`UiTree::add_node`] and cannot be
/// duplicated.
pub struct OwnedIndex(thunderdome::Index);

impl OwnedIndex {
    /// Clones the index as a [`NodeIndex`].
    pub fn shareable(&self) -> NodeIndex {
        NodeIndex(self.0)
    }
}

impl From<OwnedIndex> for NodeIndex {
    fn from(value: OwnedIndex) -> Self {
        NodeIndex(value.0)
    }
}

impl From<&OwnedIndex> for NodeIndex {
    fn from(value: &OwnedIndex) -> Self {
        NodeIndex(value.0)
    }
}

impl std::borrow::Borrow<NodeIndex> for OwnedIndex {
    fn borrow(&self) -> &NodeIndex {
        // # Safety
        // repr(transparent) used, and same definition as NodeIndex
        unsafe { std::mem::transmute(self) }
    }
}

impl From<thunderdome::Index> for NodeIndex {
    fn from(value: thunderdome::Index) -> Self {
        NodeIndex(value)
    }
}

impl From<NodeIndex> for thunderdome::Index {
    fn from(value: NodeIndex) -> thunderdome::Index {
        value.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
/// An alignment of a node within its parent.
///
/// `Begin`, `Center`, and `End` cause a node to shrink to its minimum size in to that position.
/// `Full` causes a node to occupy all space it is given. For example, to shrink to the left-middle,
/// use (`Begin`, `Center`). To fill horizontally and shrink down, use (`Full`, `End`).
///
/// The default is `Full`.
pub enum Alignment {
    /// Shrink to the left or top.
    Begin,
    /// Shrink to the center.
    Center,
    /// Shrink to the right or bottom.
    End,
    #[default]
    /// Grow to the entire available space.
    Full,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
/// Shrinking direction of a constrained rectangle.
///
/// While similar to [`Alignment`], `Anchor` does not support [`Full`], since that would be growing
/// instead of shrinking.
///
/// It is also noteworthy that the default is [`Center`], not [`Alignment::Begin`].
///
/// [`Alignment`]: Alignment
/// [`Center`]: Anchor::Center
/// [`Full`]: Alignment::Full
pub enum Anchor {
    /// Top left
    Begin,
    #[default]
    /// Center
    Center,
    /// Bottom right
    End,
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
/// A rectangle in space, represented with `f32` coordinates, representing unitless position and
/// size.
///
/// A rectangle is "valid" if *both* its width and height are non-negative. 0 is valid. A rectangle
/// is "empty" if *either* its width or height is 0. An invalid rectangle may not be empty. A
/// rectangle is "zero" if *both* its width and height are 0.
pub struct Rect {
    /// The x position of the top left corner.
    pub x: f32,
    /// The y position of the top left corner.
    pub y: f32,
    /// The width of the rectangle.
    pub w: f32,
    /// The height of the rectangle.
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

    /// Returns `false` if *either* the width or height is negative. Otherwise, returns `true`.
    pub fn is_valid(&self) -> bool {
        self.w >= 0.0 && self.h >= 0.0
    }

    /// Returns `true` if the rectangle has zero width *or* zero height.
    pub fn is_empty(&self) -> bool {
        self.w == 0.0 || self.h == 0.0
    }

    /// Returns `true` if the rectangle has zero width *and* zero height.
    pub fn is_zero(&self) -> bool {
        self.w == 0.0 && self.h == 0.0
    }

    /// Returns the width and height of the rectangle.
    pub fn get_size(&self) -> (f32, f32) {
        (self.w, self.h)
    }

    /// Set the width and height of the rectangle.
    pub fn with_size(mut self, w: f32, h: f32) -> Self {
        self.w = w;
        self.h = h;
        self
    }

    /// Shrink the width of the rectangle by the given amount toward the **left**.
    pub fn shrink_begin_x(mut self, amount: f32) -> Self {
        self.w -= amount;
        self
    }

    /// Shrink the width of the rectangle by the given amount toward the **right**.
    pub fn shrink_end_x(mut self, amount: f32) -> Self {
        self.w -= amount;
        self.x += amount;
        self
    }

    /// Shrink the **width** of the rectangle by the given amount toward the **center**.
    pub fn shrink_center_x(mut self, amount: f32) -> Self {
        self.w -= amount;
        self.x += amount * 0.5;
        self
    }

    /// Shrink the height of the rectangle by the given amount toward the **top**.
    pub fn shrink_begin_y(mut self, amount: f32) -> Self {
        self.h -= amount;
        self
    }

    /// Shrink the height of the rectangle by the given amount toward the **bottom**.
    pub fn shrink_end_y(mut self, amount: f32) -> Self {
        self.h -= amount;
        self.y += amount;
        self
    }

    /// Shrink the **height** of the rectangle by the given amount toward the **middle**.
    pub fn shrink_center_y(mut self, amount: f32) -> Self {
        self.h -= amount;
        self.y += amount * 0.5;
        self
    }

    /// Create a contained rectangle aligned within `self`.
    ///
    /// See [`Alignment`] for what each alignment does.
    ///
    /// If self` rectangle is smaller than `min` and a shrinking mode is used, the contained
    /// rectangle will *grow* in the opposite direction it would have (eg. End would grow to the
    /// top-left).
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

    /// Create a contained rectangle aligned within `self`, always shrinking to the given size.
    ///
    /// Nearly identical behavior to [`align`], but uses [`Anchor`] instead of [`Alignment`],
    /// preventing use of [`Alignment::Full`].
    ///
    /// If self` rectangle is smaller than `min`, the contained rectangle will *grow* in the
    /// opposite direction it would have (eg. End would grow to the top-left).
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

    /// Returns `true` if `point` is contained within `self`. It is inclusive of the begin edges,
    /// but exclusive of the end edges.
    pub fn contains(&self, point: (f32, f32)) -> bool {
        point.0 >= self.x
            && point.0 < self.x + self.w
            && point.1 >= self.y
            && point.1 < self.y + self.h
    }

    /// Returns `true` if `rect` is contained entirely within `self`.
    pub fn contains_rect(&self, rect: Rect) -> bool {
        rect.x >= self.x
            && rect.x + rect.w <= self.x + self.w
            && rect.y >= self.y
            && rect.y + rect.h <= self.y + self.h
    }

    /// Returns the area included by both `self` and `other`.
    ///
    /// Always returns a valid rectangle (both >=0), but it may be empty (either =0) if the two
    /// rectangles do not overlap.
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
    /// The stored minimum size of the node.
    pub min_size: (f32, f32),
    /// The stored rect of the node.
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

/// Walks a UI tree, but only in cases where a point is within the node's rect.
pub trait PointTester {
    /// Called on every visited node. Should return `true` to cancel the walk.
    fn accept(&self, p: (f32, f32), node: &dyn UiNode, rect: Rect, index: NodeIndex) -> bool;
}

#[derive(Default)]
/// Settings for [`calculate_layout_ex`]
///
/// [`calculate_layout_ex`]: UiTree::calculate_layout_ex
pub struct LayoutConfig {
    /// If `true`, applies the root node's alignment instead of ignoring it.
    pub align_root: bool,
    /// If `true`, ensures that the minimum size of the root node is met, even if that would exceed
    /// the provided rect.
    pub force_good: bool,
}

/// A tree of UI nodes, stored as an arena.
pub struct UiTree {
    root: thunderdome::Index,
    arena: Arena<Box<dyn UiNode>>,
    cache: HashMap<thunderdome::Index, NodeCache>,
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
    ///
    /// This produces an [`OwnedIndex`] that is used to ensure exactly one node holds parental
    /// "ownership" of the node in the hierarchy.
    pub fn add_node(&mut self, node: impl UiNode) -> OwnedIndex {
        let index = self.arena.insert(Box::new(node) as Box<dyn UiNode>);
        self.cache.insert(index, Default::default());
        OwnedIndex(index)
    }

    /// Remove a node and all of its children from the arena.
    ///
    /// # Panics
    /// If the root node is removed.
    pub fn remove_node(&mut self, index: OwnedIndex) {
        assert_ne!(index.0, self.root, "Root node cannot be removed");

        let mut queue: VecDeque<_> = self.arena[index.0].get_children().into();
        while let Some(child) = queue.pop_front() {
            queue.extend(self.arena[child.0].get_children());
            self.arena.remove(child.0);
            self.cache.remove(&child.0);
        }
        self.arena.remove(index.0);
        self.cache.remove(&index.0);
    }

    /// Get a reference to the cached layout information for a node.
    pub fn get_cache(&self, index: NodeIndex) -> Option<&NodeCache> {
        self.cache.get(&index.0)
    }

    /// Get a reference to a node.
    pub fn get_node(&self, index: NodeIndex) -> Option<&dyn UiNode> {
        self.arena.get(index.0).map(|node| &**node)
    }

    /// Get a mutable reference to a node.
    pub fn get_node_mut(&mut self, index: NodeIndex) -> Option<&mut dyn UiNode> {
        self.arena.get_mut(index.0).map(|node| &mut **node)
    }

    /// Get a reference to a node, performing a downcast at the same time.
    pub fn get_cast<T: UiNode>(&self, index: NodeIndex) -> Option<&T> {
        self.get_node(index).and_then(|node| node.downcast_ref())
    }

    /// Get a mutable reference to a node, performing a downcast at the same time.
    pub fn get_cast_mut<T: UiNode>(&mut self, index: NodeIndex) -> Option<&mut T> {
        self.get_node_mut(index).and_then(|node| node.downcast_mut())
    }

    /// Return the index of the root node.
    pub fn root_index(&self) -> NodeIndex {
        NodeIndex(self.root)
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
    pub fn calculate_layout_ex(&mut self, root_rect: Rect, config: LayoutConfig) -> bool {
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
            visit_stack.extend(
                self.arena[v]
                    .get_visible_children()
                    .into_iter()
                    .map(|v| v.0),
            );
        }

        for v in min_stack.iter().rev() {
            let min = self.arena[*v].calculate_min_size(self);
            self.cache.entry(*v).or_default().min_size = min;
        }

        let (is_good, root_rect) = if config.force_good {
            let root_min = self.cache[&self.root].min_size;
            (
                true,
                Rect::new(
                    0.0,
                    0.0,
                    root_min.0.max(root_rect.w),
                    root_min.1.max(root_rect.h),
                ),
            )
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
                self.cache.entry(child.0).or_default().rect = rect;
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
    pub fn walk_tree(&mut self, walker: &mut impl UiWalker, use_visible: bool) {
        self.walk_node(NodeIndex(self.root), walker, use_visible);
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
    pub fn walk_node(&mut self, index: NodeIndex, walker: &mut impl UiWalker, use_visible: bool) {
        let rect = self.cache[&index.0].rect;
        walker.enter(self.arena[index.0].as_mut(), rect, index);

        let children = if use_visible {
            self.arena[index.0].get_visible_children()
        } else {
            self.arena[index.0].get_children()
        };
        for child in children {
            self.walk_node(child, walker, use_visible);
        }

        walker.leave(self.arena[index.0].as_mut(), rect, index);
    }

    /// Walks the tree from the root, whitelisting nodes where `point` is contained within the
    /// node's rect.
    ///
    /// Only visible nodes will be visited. Nodes that do not contain `point` will not be visited.
    pub fn point_test(&self, point: (f32, f32), tester: &mut impl PointTester) {
        self.point_test_int(self.root, point, tester);
    }

    /// Walks a single node and its children, whitelisting nodes where `point` is contained within
    /// the node's rect.
    ///
    /// Only visible nodes will be visited. Nodes that do not contain `point` will not be visited.
    pub fn point_test_node(
        &self,
        index: NodeIndex,
        point: (f32, f32),
        tester: &mut impl PointTester,
    ) {
        self.point_test_int(index.0, point, tester);
    }

    fn point_test_int(
        &self,
        index: thunderdome::Index,
        point: (f32, f32),
        tester: &mut impl PointTester,
    ) -> bool {
        let rect = self.cache[&index].rect;
        if !rect.contains(point) {
            return false;
        }

        if tester.accept(point, self.arena[index].as_ref(), rect, NodeIndex(index)) {
            return true;
        }

        let children = self.arena[index].get_children();

        for child in children.into_iter() {
            if self.point_test_int(child.0, point, tester) {
                return true;
            }
        }

        false
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
    pub fn add_node(&mut self, node: impl UiNode) -> OwnedIndex {
        OwnedIndex(self.arena.insert(Box::new(node) as Box<dyn UiNode>))
    }

    /// Converts a partial tree into a [`UiTree`], using the given root node.
    ///
    /// The root node is taken as an [`OwnedIndex`], since it cannot have a parent.
    ///
    /// Existing [`NodeIndex`]es are not invalidated.
    pub fn complete(self, root: OwnedIndex) -> UiTree {
        let cache = self
            .arena
            .iter()
            .map(|(id, _)| (id, Default::default()))
            .collect();
        UiTree {
            arena: self.arena,
            root: root.0,
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
/// # Arguments
///
/// The macro takes a tree and a node, recursively appending nodes to the tree. **The returned value
/// is dependent on what type of tree argument is passed.**
///
/// ### Tree
///
/// The tree can be either:
///
/// - `%%`, which creates a new [`UiTree`] and returns it.
///
/// - `%%!`, which creates a new [`PartialTree`] and returns a tuple containing it and the owned
///   index of the new node.
///
/// - Or a mutable reference to a tree, which will return the owned index of the new node. The tree
///   can be either a [`UiTree`] or a [`PartialTree`].
///
/// ### Node
///
/// Nodes are recursively defined by constructing them, modifying them, and adding them to the tree.
///
/// Each node is represented by an optional assignment; two alignment symbols, separated by a `|`; a
/// constructor for the node; and an optional list of children, contained in square brackets after
/// `=>`.
///
/// Ex `binding = +|+ HStack::new() => [ ]`
///
/// The assignment is a *name* followed by `=`, not an expression. The variable with the
/// corresponding name will be assigned the [`NodeIndex`] of the created node.
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
/// You can also replace a node with `#expr` to use a node that is already present in the tree. It
/// expects a [`OwnedIndex`], not a [`NodeIndex`].
///
/// # Example usage
/// ```rust
/// use layuit::padding::{Spacer, Minimum};
/// use layuit::stack::HStack;
///
/// let minimum;
/// let tree = layuit::ui!(
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
            tree.complete(root)
        }
    };

    (%%!, $($node:tt)*) => {
        {
            let mut tree = $crate::PartialTree::new();
            let root = $crate::ui!(@@_node &mut tree;; $($node)*);
            (tree, root)
        }
    };

    (@@_node $tree:expr;; #$expr:expr) => {
        $expr
    };

    // With binding, no children
    (@@_node $tree:expr;; $binding:ident = $ha:tt | $va:tt $node:expr) => {
        {
            let __node_idx = $tree.add_node($crate::ui!(@@_align $ha | $va $node));
            $binding = __node_idx.shareable();
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
            $binding = __node_idx.shareable();
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

    (@@_node $tree:expr;; $node:expr $(=> [ $($child:tt)* ])?) => {
        compile_error!(
            "Nodes must have an alignment, represented by two of `<`, `-`, `>`, and `+`, \
            separated by a `|`"
        )
    };

    (@@_node $tree:expr;; $($tt:tt)*) => {
        compile_error!(concat!(
            "Invalid node syntax. A node should have an optional assignment, two alignment \
            characters, separated by a pipe, and a node expression, optionally followed by \
            a fat arrow and children in square brackets.\n\nExample:\n\
            `target = -|< Node::new() => [ ]`\n\nFound: `",
            $(stringify!($tt),)*
            "`."
        ))
    };

    ($($random:tt)*, $($node:tt)*) => {
        compile_error!(concat!(
            "Expected either a mutable reference to a tree, `%%`, or `%%!`, found `",
            stringify!($($random)*),
            "` where the tree should have been."
        ));
    };

    ($($tt:tt)*) => {
        compile_error!(concat!(
            "Expected a tree followed by a `,`, followed by a node, found `",
            stringify!($($tt)*),
            "`."
        ))
    };
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn rect_intersect1() {
        let r1 = Rect::new(0.0, 0.0, 10.0, 10.0);
        let r2 = Rect::new(5.0, 0.0, 10.0, 10.0);

        assert_eq!(
            r1.intersect(r2),
            Rect::new(5.0, 0.0, 5.0, 10.0)
        );
    }

    #[test]
    fn valid_tree() {
        let root;
        let sep;
        let mut tree = ui!(
            %%,
            root = +|+ stack::HStack::new() => [
                -|> split3::HSplit3::new() => [
                    <|+ padding::Spacer::sized((10.0, 5.0)),
                    sep = -|+ padding::Spacer::sized((1.0, 10.0)),
                    >|+ padding::Spacer::sized((10.0, 5.0)),
                ],
                -|< padding::Spacer::sized((10.0, 10.0)),
            ]
        );

        tree
            .get_cast_mut::<stack::HStack>(root)
            .unwrap()
            .spacing = 10.0;

        assert_eq!(
            tree
                .get_cast::<padding::Spacer>(sep)
                .unwrap()
                .get_size(),
            (1.0, 10.0)
        )
    }


}