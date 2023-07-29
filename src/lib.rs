extern crate peg;

pub mod css;
pub mod dom;
pub mod html;
pub mod layout;
pub mod painting;
pub mod style;

pub fn parse_html(h: &str) -> dom::Node {
    dom::Node::from(h)
}

pub fn parse_css(c: &str) -> css::Sheet {
    css::Sheet::from(c)
}

pub fn build_style_tree<'a>(h: dom::NodeRef, c: &'a css::Sheet) -> style::StyledNodeRef {
    style::style_tree(h, c)
}

pub fn build_layout_tree(s: style::StyledNodeRef, d: layout::Dimensions) -> layout::LayoutBoxRef {
    layout::layout_tree(s, d)
}

pub fn build_display_list(l: layout::LayoutBoxRef) -> painting::DisplayList {
    painting::build_display_list(l)
}
