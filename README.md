# Layuit

A renderer-agnostic UI layout system. Layuit handles computing the size and position of various
[`UiNode`]s in a [`UiTree`]. Layuit does not handle rendering, but provides tools for doing so.

Layuit provides several organizational nodes such as [`HStack`] and [`Margin`], but allows users
to create their own nodes.

Layuit uses the [`thunderdome`] crate for the tree structure. To access nodes from a tree, use
[`thunderdome::Index`].

## Core concepts

- **[`UiTree`]**: Owns the [`UiNode`]s and layout information in an arena and handles
  computation and access.
- **[`UiNode`]**: A trait implemented by all UI nodes, containing alignment and any number of
  children.
- **[`NodeCache`]**: The cached layout information for a node, produced by
  [`UiTree::calculate_layout`].
- **[`Rect`]**: A rectangle in space, represented with `f32` coordinates.
- **[`Alignment`]**: An alignment primarily used for determining node placement.
- **[`NodeVisitor`]**: A trait implemented e.g. by renderers to process and/or manipulate nodes.

## Layout process

Layout runs in two passes, when [`UiTree::calculate_layout`] is called:

1. **Bottom-up: minimum size.** Children are visited before their parent. Each node computes its
   minimum size based on its children through [`calculate_min_size`] and stores it in its
   [`NodeCache::min_size`].

2. **Top-down: rectangles.** Starting from the root, each node computes the position and size of
   its immediate children through [`calculate_rects`]. Each child then uses its restricted
   [`Rect`] to do the same for its children. The [`NodeCache::rect`] field is populated with
   the resulting [`Rect`]s.

## Caveats

**The cache is stale before [`UiTree::calculate_layout`] is called**, and becomes stale if
children are added, removed, moved, or otherwise changed. The cache always produces valid
results, but they may be out of date or set to 0.

**Minimum size is a practice, not a requirement**. When implementing custom nodes, be wary of
ensuring each node's minimum size is enforced. This can easily become a problem if the space
required by the entire tree is smaller than the one provided to [`UiTree::calculate_layout`].

## Implementing custom nodes

Custom nodes are essential to using Layuit. Without them, no meaningful UI can be rendered.
However, it is important to ensure you follow the rules:

1. **Children must be accurately reported.** Failure to report children will result in them not
   being updated during [`UiTree::calculate_layout`] or removed during [`UiTree::remove_node`].

2. **Minimum size must be correctly calculated.** Under-representing the minimum size can and
   often will result in nodes overflowing into each other.

3. **Rectangles must be properly assigned.** Similar to #2, it is the responsibility of the
   *parent* node to ensure that each node get both enough space and not too much. Failing to do
   so will result in nodes overlapping.

One common custom node is the `Label`:

```rust
use layuit::{Alignment, NodeCache, Rect, UiTree, UiNode};

pub struct Label {
    text: String,
    align: (Alignment, Alignment),
    cached_size: (f32, f32),
}

/* Label methods and constructors... */

impl UiNode for Label {
    fn get_align(&self) -> (Alignment, Alignment) { self.align }
    fn get_align_mut(&mut self) -> (&mut Alignment, &mut Alignment) {
        (&mut self.align.0, &mut self.align.1)
    }

    fn calculate_min_size(&mut self, _tree: &UiTree, _cache: &mut NodeCache) -> (f32, f32) {
        self.cached_size
    }

    // calculate_rects and get_children are omitted for leaf nodes
}
```

## Creating a tree

Every tree needs a root node, which cannot be removed. Good choices are [`Overlap`] and either
[`HStack`] or [`VStack`]. A custom node can also be used.

```rust
use layuit::{UiTree, UiNode, NodeVisitor};
use layuit::stacks::HStack;

// The root node can be any UiNode, but must be specified.
let mut tree = UiTree::new(HStack::new().with_spacing(4.0));

// Create a label wrapped in a 4px margin
let padded_label = Margin::new()
    .with_margins(4.0, 4.0, 4.0, 4.0)
    .with_child(Label::new("Hello, world!"), &mut tree);

// Add the label to the root stack
tree.get_root_mut()
    .downcast_mut::<HStack>()
    .unwrap()
    .with_child(padded_label, &mut tree);

tree.calculate_layout(Rect::new(0.0, 0.0, 640.0, 480.0));

// Render the UI tree

struct Renderer {
    // ...
}

impl NodeVisitor for Renderer {
    fn visit(&mut self, node: &mut dyn UiNode, rect: layuit::Rect) {
        if let Some(label) = node.downcast_mut::<Label>() {
            // ...
        }
    }
}

let mut renderer = /* ... */;
tree.visit(&mut renderer);
```

## Provided nodes

Containers:
- [`HStack`] - Horizontal arrangement
- [`VStack`] - Vertical arrangement
- [`Overlap`] - Independent arrangement of children
- [`Margin`] - Adds padding to a child
- [`Minimum`] - Creates a minimum size constraint for precise control

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

[`thunderdome`]: https://crates.io/crates/thunderdome
[`thunderdome::Index`]: https://docs.rs/thunderdome/latest/thunderdome/struct.Index.html