use std::cell::RefCell;
use std::default::Default;
use std::ops::Deref;
use std::rc::Rc;

use crate::css::Unit::Px;
use crate::css::Value::{Keyword, Length};
use crate::style::{Display, StyledNodeRef};

pub use self::BoxType::*;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Dimensions {
    pub content: Rect,
    pub padding: EdgeSizes,
    pub border: EdgeSizes,
    pub margin: EdgeSizes,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct EdgeSizes {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[derive(Debug)]
pub struct LayoutBoxRef(Rc<RefCell<LayoutBox>>);

impl LayoutBoxRef {
    pub fn new(b: LayoutBox) -> Self {
        Self(Rc::new(RefCell::new(b)))
    }
}

impl Deref for LayoutBoxRef {
    type Target = Rc<RefCell<LayoutBox>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct LayoutBox {
    pub dimensions: Dimensions,
    pub box_type: BoxType,
    pub children: Vec<LayoutBoxRef>,
}

#[derive(Debug)]
pub enum BoxType {
    BlockNode(StyledNodeRef),
    InlineNode(StyledNodeRef),
    InlineBlockNode(StyledNodeRef),
    AnonymousBlock,
    // LineBox,
}

impl LayoutBox {
    fn new(box_type: BoxType) -> LayoutBox {
        LayoutBox {
            box_type,
            dimensions: Default::default(),
            children: Vec::new(),
        }
    }

    fn get_style_node(&self) -> StyledNodeRef {
        match self.box_type {
            BlockNode(node) | InlineNode(node) | InlineBlockNode(node) => node,
            AnonymousBlock => panic!("Anonymous block box has no style node"),
        }
    }
}

pub fn layout_tree(node: StyledNodeRef, mut containing_block: Dimensions) -> LayoutBoxRef {
    // The layout algorithm expects the container height to start at 0.
    // TODO: Save the initial containing block height, for calculating percent heights.
    containing_block.content.height = 0.0;

    let mut root_box = build_layout_tree(node);
    root_box.borrow().layout(containing_block);
    root_box
}

/// Build the tree of LayoutBoxes, but don't perform any layout calculations yet.
fn build_layout_tree(style_node: StyledNodeRef) -> LayoutBoxRef {
    // Create the root box.
    let mut root = LayoutBox::new(match style_node.borrow().display() {
        Display::Block => BlockNode(style_node),
        Display::Inline => InlineNode(style_node),
        Display::InlineBlock => InlineBlockNode(style_node),
        Display::None => panic!("Root node has display: none."),
    });

    // Create the descendant boxes.
    for child in style_node.borrow().children {
        match child.borrow().display() {
            Display::Block => root.children.push(build_layout_tree(child)),
            Display::Inline | Display::InlineBlock => root
                .get_inline_container()
                .borrow()
                .children
                .push(build_layout_tree(child)),
            Display::None => {} // Don't lay out nodes with `display: none;`
        }
    }
    LayoutBoxRef(Rc::new(RefCell::new(root)))
}

impl LayoutBox {
    /// Lay out a box and its descendants.
    fn layout(&mut self, containing_block: Dimensions) {
        match self.box_type {
            BlockNode(_) => self.layout_block(containing_block),
            InlineBlockNode(_) => self.layout_inline_block(containing_block),
            InlineNode(_) => self.layout_inline(containing_block),
            AnonymousBlock => {} // TODO
        }
    }

    fn layout_inline_block(&mut self, containing_block: Dimensions) {
        println!("layout inline block!");
        self.calculate_block_width(containing_block);
        self.calculate_inline_position(containing_block);
        self.layout_block_children();
        self.calculate_block_height();
    }

    fn layout_inline(&mut self, containing_block: Dimensions) {
        self.calculate_inline_width(containing_block);
        self.calculate_inline_position(containing_block);
        self.layout_inline_children();
        self.calculate_inline_height();
    }

    fn layout_block(&mut self, containing_block: Dimensions) {
        self.calculate_block_width(containing_block);
        self.calculate_block_position(containing_block);
        self.layout_block_children();
        self.calculate_block_height();
    }

    fn calculate_inline_width(&mut self, containing_block: Dimensions) {
        // TODO implement
        // If the width is set to an explicit length, use that exact length.
        // Otherwise, just keep the value set by `layout_inline_children`.
        if let Some(Length(w, Px)) = self.get_style_node().borrow().value("width") {
            self.dimensions.content.width = w;
            println!("setting box width to {}", w);
        } else {
            println!("box width remains {}", self.dimensions.content.width)
        }
    }

    /// Calculate the width of a block-level non-replaced element in normal flow.
    ///
    /// http://www.w3.org/TR/CSS2/visudet.html#blockwidth
    ///
    /// Sets the horizontal margin/padding/border dimensions, and the `width`.
    fn calculate_block_width(&mut self, containing_block: Dimensions) {
        // Child width can depend on parent width, so we need to calculate this box's width before
        // laying out its children.
        let style = self.get_style_node();

        // `width` has initial value `auto`.
        let auto = Keyword("auto".to_string());
        let mut width = style.borrow().value("width").unwrap_or(auto.clone());

        // margin, border, and padding have initial value 0.
        let zero = Length(0.0, Px);

        let mut margin_left = style.borrow().lookup("margin-left", "margin", &zero);
        let mut margin_right = style.borrow().lookup("margin-right", "margin", &zero);

        let border_left = style.borrow().lookup("border-left-width", "border-width", &zero);
        let border_right = style.borrow().lookup("border-right-width", "border-width", &zero);

        let padding_left = style.borrow().lookup("padding-left", "padding", &zero);
        let padding_right = style.borrow().lookup("padding-right", "padding", &zero);

        let total = sum([
            &margin_left,
            &margin_right,
            &border_left,
            &border_right,
            &padding_left,
            &padding_right,
            &width,
        ]
        .iter()
        .map(|v| v.to_px()));

        // If width is not auto and the total is wider than the container, treat auto margins as 0.
        if width != auto && total > containing_block.content.width {
            if margin_left == auto {
                margin_left = Length(0.0, Px);
            }
            if margin_right == auto {
                margin_right = Length(0.0, Px);
            }
        }

        // Adjust used values so that the above sum equals `containing_block.width`.
        // Each arm of the `match` should increase the total width by exactly `underflow`,
        // and afterward all values should be absolute lengths in px.
        let underflow = containing_block.content.width - total;

        match (width == auto, margin_left == auto, margin_right == auto) {
            // If the values are overconstrained, calculate margin_right.
            (false, false, false) => {
                margin_right = Length(margin_right.to_px() + underflow, Px);
            }

            // If exactly one size is auto, its used value follows from the equality.
            (false, false, true) => {
                margin_right = Length(underflow, Px);
            }
            (false, true, false) => {
                margin_left = Length(underflow, Px);
            }

            // If width is set to auto, any other auto values become 0.
            (true, _, _) => {
                if margin_left == auto {
                    margin_left = Length(0.0, Px);
                }
                if margin_right == auto {
                    margin_right = Length(0.0, Px);
                }

                if underflow >= 0.0 {
                    // Expand width to fill the underflow.
                    width = Length(underflow, Px);
                } else {
                    // Width can't be negative. Adjust the right margin instead.
                    width = Length(0.0, Px);
                    margin_right = Length(margin_right.to_px() + underflow, Px);
                }
            }

            // If margin-left and margin-right are both auto, their used values are equal.
            (false, true, true) => {
                margin_left = Length(underflow / 2.0, Px);
                margin_right = Length(underflow / 2.0, Px);
            }
        }

        let d = &mut self.dimensions;
        d.content.width = width.to_px();

        d.padding.left = padding_left.to_px();
        d.padding.right = padding_right.to_px();

        d.border.left = border_left.to_px();
        d.border.right = border_right.to_px();

        d.margin.left = margin_left.to_px();
        d.margin.right = margin_right.to_px();
    }

    fn calculate_inline_position(&mut self, containing_block: Dimensions) {
        // TODO implement
        // inline is positioned horizontally next to the previous sibling, until the line is
        // full. Then a new linebox is created and we start from the left again.
        let style = self.get_style_node();
        let d = &mut self.dimensions;

        // margin, border, and padding have initial value 0.
        let zero = Length(0.0, Px);

        // If margin-top or margin-bottom is `auto`, the used value is zero.
        d.margin.top = style.borrow().lookup("margin-top", "margin", &zero).to_px();
        d.margin.bottom = style.borrow().lookup("margin-bottom", "margin", &zero).to_px();

        d.border.top = style
            .borrow().lookup("border-top-width", "border-width", &zero)
            .to_px();
        d.border.bottom = style
            .borrow().lookup("border-bottom-width", "border-width", &zero)
            .to_px();

        d.padding.top = style.borrow().lookup("padding-top", "padding", &zero).to_px();
        d.padding.bottom = style.borrow().lookup("padding-bottom", "padding", &zero).to_px();

        // position block to right of preceding blocks in container
        // TODO: go to a new line if it doesn't fit in container content area
        d.content.x = containing_block.content.width
            + containing_block.content.x
            + d.margin.left
            + d.border.left
            + d.padding.left;

        d.content.y = containing_block.content.y + d.margin.top + d.border.top + d.padding.top;
    }

    /// Finish calculating the block's edge sizes, and position it within its containing block.
    ///
    /// http://www.w3.org/TR/CSS2/visudet.html#normal-block
    ///
    /// Sets the vertical margin/padding/border dimensions, and the `x`, `y` values.
    fn calculate_block_position(&mut self, containing_block: Dimensions) {
        // Determine where the box is located within its container.

        let style = self.get_style_node();
        let d = &mut self.dimensions;

        // margin, border, and padding have initial value 0.
        let zero = Length(0.0, Px);

        // If margin-top or margin-bottom is `auto`, the used value is zero.
        d.margin.top = style.borrow().lookup("margin-top", "margin", &zero).to_px();
        d.margin.bottom = style.borrow().lookup("margin-bottom", "margin", &zero).to_px();

        d.border.top = style
            .borrow().lookup("border-top-width", "border-width", &zero)
            .to_px();
        d.border.bottom = style
            .borrow().lookup("border-bottom-width", "border-width", &zero)
            .to_px();

        d.padding.top = style.borrow().lookup("padding-top", "padding", &zero).to_px();
        d.padding.bottom = style.borrow().lookup("padding-bottom", "padding", &zero).to_px();

        d.content.x = containing_block.content.x + d.margin.left + d.border.left + d.padding.left;

        // Position the box below all the previous boxes in the container.
        d.content.y = containing_block.content.height
            + containing_block.content.y
            + d.margin.top
            + d.border.top
            + d.padding.top;
    }

    fn layout_inline_children(&mut self) {
        let d = &mut self.dimensions;
        for child in self.children {
            child.borrow_mut().layout(*d);
            // Increment the width so each child is laid out next to the previous one.
            d.content.width += child.borrow().dimensions.margin_box().width;
        }
    }

    /// Lay out the block's children within its content area.
    ///
    /// Sets `self.dimensions.height` to the total content height.
    fn layout_block_children(&mut self) {
        let d = &mut self.dimensions;
        for child in self.children {
            child.borrow_mut().layout(*d);
            // Increment the height so each child is laid out below the previous one.
            d.content.height += child.borrow().dimensions.margin_box().height;
        }
    }

    fn calculate_inline_height(&mut self) {
        // If the height is set to an explicit length, use that exact length.
        // Otherwise, just keep the value set by `layout_block_children`.
        if let Some(Length(h, Px)) = self.get_style_node().borrow().value("height") {
            self.dimensions.content.height = h;
            println!("setting box height to {}", h);
        } else {
            println!("box height remains {}", self.dimensions.content.height)
        }
    }

    /// Height of a block-level non-replaced element in normal flow with overflow visible.
    fn calculate_block_height(&mut self) {
        // Parent height can depend on child height, so `calculate_height` must be called after the
        // children are laid out.

        // If the height is set to an explicit length, use that exact length.
        // Otherwise, just keep the value set by `layout_block_children`.
        if let Some(Length(h, Px)) = self.get_style_node().borrow().value("height") {
            self.dimensions.content.height = h;
            println!("setting block height to {}", h);
        } else {
            println!("block height remains {}", self.dimensions.content.height)
        }
    }

    /// Where a new inline child should go.
    fn get_inline_container(&mut self) -> LayoutBoxRef {
        match self.box_type {
            InlineNode(_) | InlineBlockNode(_) | AnonymousBlock => LayoutBoxRef::new(self),
            BlockNode(_) => {
                // If we've just generated an anonymous block box, keep using it.
                // Otherwise, create a new one.
                if let Some(c)
                match self.children.last().borrow() {
                    Some( {
                        box_type: AnonymousBlock,
                        ..
                    }) => {}
                    _ => self.children.push(LayoutBox::new(AnonymousBlock)),
                }
                self.children.last_mut().unwrap()
            }
        }
    }
}

impl Rect {
    pub fn expanded_by(self, edge: EdgeSizes) -> Rect {
        Rect {
            x: self.x - edge.left,
            y: self.y - edge.top,
            width: self.width + edge.left + edge.right,
            height: self.height + edge.top + edge.bottom,
        }
    }
}

impl Dimensions {
    /// The area covered by the content area plus its padding.
    pub fn padding_box(self) -> Rect {
        self.content.expanded_by(self.padding)
    }
    /// The area covered by the content area plus padding and borders.
    pub fn border_box(self) -> Rect {
        self.padding_box().expanded_by(self.border)
    }
    /// The area covered by the content area plus padding, borders, and margin.
    pub fn margin_box(self) -> Rect {
        self.border_box().expanded_by(self.margin)
    }
}

fn sum<I>(iter: I) -> f32
where
    I: Iterator<Item = f32>,
{
    iter.fold(0., |a, b| a + b)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::css::*;
    use crate::dom::*;
    use crate::layout::*;
    use crate::style::*;

    #[test]
    fn test_layout() {
        let document = Node::from(
            r#"
            <html>
                <body class="bar">
                    <h1>Hi!</h1>
                    <p>Bye!</p>
                </body>
            </html>
        "#,
        );

        let style = Sheet::from(
            r#"
            html, body, h1, p {
                display: block;
            }

            html {
                height: 600px;
            }

            body.foo, p {
                margin: auto;
                width: 24px;
            }
        "#,
        );

        let style = style_tree(&document, &style);

        let mut viewport: Dimensions = Default::default();
        viewport.content.width = 800.0;
        viewport.content.height = 600.0;

        let actual = layout_tree(&style, viewport);

        let body = &actual.children[0];
        let h1 = &body.children[0];
        let p = &body.children[1];

        assert_eq!(actual.dimensions, viewport);
        assert_eq!(body.dimensions.content.width, 800.0);
        assert_eq!(h1.dimensions.content.width, 800.0);
        assert_eq!(p.dimensions.content.width, 24.0);
    }

    #[test]
    fn test_layout_inline() {
        let document = Node::from(
            "
            <a>
                <b>
                    <c>Hello</c>
                    <c>world!</c>
                </b>
                <b>
                    <c>Bye</c>
                    <c>all!</c>
                </b>
            </a>
        ",
        );

        let style = Sheet::from(
            "
            a, b {
                display: block;
            }

            a {
                height: 600px;
            }

            b {
                background-color: #ff0000;
                margin: 24px;
                width: 100px;
                height: 100px;
            }

            c {
                display: inline;
                background-color: blue;
                margin: 24px;
                width: 32px;
                height: 24px;
            }
        ",
        );

        let applied_styles = style_tree(&document, &style);

        let mut viewport: Dimensions = Default::default();
        viewport.content.width = 800.0;
        viewport.content.height = 600.0;

        let actual = layout_tree(&applied_styles, viewport);

        assert_eq!(actual.dimensions, viewport);

        let b0 = &actual.children[0];
        let c0 = &b0.children[0].children[0]; // TODO: unnecessary anonymous box
        let c1 = &b0.children[0].children[1];

        assert_eq!(actual.dimensions, viewport);
        assert_eq!(
            b0.dimensions,
            Dimensions {
                content: Rect {
                    x: 24.0,
                    y: 24.0,
                    width: 100.0,
                    height: 100.0
                },
                padding: EdgeSizes {
                    left: 0.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0
                },
                border: EdgeSizes {
                    left: 0.0,
                    right: 0.0,
                    top: 0.0,
                    bottom: 0.0
                },
                margin: EdgeSizes {
                    left: 24.0,
                    right: 676.0,
                    top: 24.0,
                    bottom: 24.0
                },
            }
        );

        // TODO: inline positioning not implemented yet

        // assert_eq!(c0.dimensions, Dimensions {
        //     content: Rect { x: 24.0, y: 24.0, width: 32.0, height: 32.0 },
        //     padding: EdgeSizes { left: 0.0, right: 0.0, top: 0.0, bottom: 0.0 },
        //     border: EdgeSizes { left: 0.0, right: 0.0, top: 0.0, bottom: 0.0 },
        //     margin: EdgeSizes { left: 24.0, right: 676.0, top: 24.0, bottom: 24.0 },

        // });
        // assert_eq!(c1.dimensions.content.width, 32.0);

        if let BoxType::InlineNode(_) = c0.box_type {
        } else {
            panic!();
        }

        if let BoxType::InlineNode(_) = c1.box_type {
        } else {
            panic!();
        }
    }
}
