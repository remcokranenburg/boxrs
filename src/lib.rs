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

pub fn build_style_tree<'a>(h: &'a dom::Node, c: &'a css::Sheet) -> style::StyledNode<'a> {
    style::style_tree(h, c)
}

pub fn build_layout_tree<'a>(s: &'a style::StyledNode, d: layout::Dimensions) -> layout::LayoutBox<'a> {
    layout::layout_tree(s, d)
}

pub fn build_display_list(l: &layout::LayoutBox) -> painting::DisplayList {
    painting::build_display_list(l)
}
