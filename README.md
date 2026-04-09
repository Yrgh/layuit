# Layuit

A renderer-agnostic UI layout system. Layuit handles computing the size and position of various
[`UiNode`]s in a [`UiTree`]. Layuit does not handle rendering, but provides tools for doing so.

Layuit provides several organizational nodes such as [`HStack`] and [`Margin`], but allows users
to create their own nodes.

Layuit also provides a helpful [`ui!`] macro to make constructing complex static trees much easier
and more readable.

## Core concepts

- **[`UiTree`]**: Owns the [`UiNode`]s and layout information in an arena and handles
  computation and access.
- **[`UiNode`]**: A trait implemented by all UI nodes, containing alignment and any number of
  children.
- **[`Rect`]**: A rectangle in space, represented with `f32` coordinates.
- **[`Alignment`]**: An alignment primarily used for determining node placement.

## Layout process

Layout runs in two passes, when [`UiTree::calculate_layout`] is called:

1. **Bottom-up: minimum size.** Children are visited before their parent. Each node computes its
   minimum size based on its children through [`calculate_min_size`] and stores it in its
   [`NodeCache::min_size`].

2. **Top-down: rectangles.** Starting from the root, each node computes the position and size of
   its immediate children through [`calculate_rects`]. Each child then uses its restricted
   [`Rect`] to do the same for its children. The [`NodeCache::rect`] field is populated with
   the resulting [`Rect`]s.

## Provided nodes

- [`HStack`] - Horizontal arrangement
- [`VStack`] - Vertical arrangement
- [`Overlap`] - Independent arrangement of children
- [`Margin`] - Adds padding to a child
- [`Minimum`] - Creates a minimum size constraint for precise control
- [`Spacer`] - A leaf node with configurable empty space
- [`Clip`] - Allows a child to outgrow the node with the assumption that the renderer will
  clip it, and enables a scroll offset to be applied if the child is larger.
- [`Hider`] - Allows a child's visibility to be controlled. An invisible node has no minimum
  size and should not be attempted to be rendered.
- [`Selector`] - Selects a single child node to be visible at a time.
- [`AspectRatio`] - Maintains a horizontal:vertical ratio
- [`HSplit`] - Horizontal split between two children
- [`VSplit`] - Vertical split between two children
- [`Percent`] - Maintains a percentage of space for a child
- [`HEqual`] - Horizontal arrangement with each child getting equal space
- [`VEqual`] - Vertical arrangement with each child getting equal space
- [`Grid`] - 2D grid of children
- [`Clamp`] - Constrains a child to a maximum size.

## External dependencies

- [`thunderdome`] - Used publicly to provide indexing into [`UiTree`]s.
- [`indexmap`] - Used privately to provide a stable ordering of children in multi-child nodes.

[`Rect`]: https://docs.rs/layuit/latest/layuit/struct.Rect.html
[`Alignment`]: https://docs.rs/layuit/latest/layuit/enum.Alignment.html
[`NodeCache`]: https://docs.rs/layuit/latest/layuit/struct.NodeCache.html
[`UiTree`]: https://docs.rs/layuit/latest/layuit/struct.UiTree.html
[`UiTree::calculate_layout`]: https://docs.rs/layuit/latest/layuit/struct.UiTree.html#method.calculate_layout
[`UiTree::remove_node`]: https://docs.rs/layuit/latest/layuit/struct.UiTree.html#method.remove_node
[`NodeVisitor`]: https://docs.rs/layuit/latest/layuit/trait.NodeVisitor.html
[`UiNode`]: https://docs.rs/layuit/latest/layuit/trait.UiNode.html
[`calculate_min_size`]: https://docs.rs/layuit/latest/layuit/trait.UiNode.html#tymethod.calculate_min_size
[`calculate_rects`]: https://docs.rs/layuit/latest/layuit/trait.UiNode.html#tymethod.calculate_rects
[`HStack`]: https://docs.rs/layuit/latest/layuit/stacks/struct.HStack.html
[`VStack`]: https://docs.rs/layuit/latest/layuit/stacks/struct.VStack.html
[`Overlap`]: https://docs.rs/layuit/latest/layuit/overlap/struct.Overlap.html
[`Margin`]: https://docs.rs/layuit/latest/layuit/padding/struct.Margin.html
[`Minimum`]: https://docs.rs/layuit/latest/layuit/padding/struct.Minimum.html
[`Spacer`]: https://docs.rs/layuit/latest/layuit/padding/struct.Spacer.html
[`Clip`]: https://docs.rs/layuit/latest/layuit/clip/struct.Clip.html
[`Hider`]: https://docs.rs/layuit/latest/layuit/visibility/struct.Hider.html
[`Selector`]: https://docs.rs/layuit/latest/layuit/visibility/struct.Selector.html
[`AspectRatio`]: https://docs.rs/layuit/latest/layuit/proportional/struct.AspectRatio.html
[`HSplit`]: https://docs.rs/layuit/latest/layuit/proportional/struct.HSplit.html
[`VSplit`]: https://docs.rs/layuit/latest/layuit/proportional/struct.VSplit.html
[`Percent`]: https://docs.rs/layuit/latest/layuit/proportional/struct.Percent.html
[`HEqual`]: https://docs.rs/layuit/latest/layuit/grid/struct.HEqual.html
[`VEqual`]: https://docs.rs/layuit/latest/layuit/grid/struct.VEqual.html
[`Grid`]: https://docs.rs/layuit/latest/layuit/grid/struct.Grid.html
[`Clamp`]: https://docs.rs/layuit/latest/layuit/macro.ui.html
[`ui!`]: 

[`thunderdome`]: https://crates.io/crates/thunderdome
[`indexmap`]: https://crates.io/crates/indexmap