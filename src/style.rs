use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use crate::css::{Rule, Selector, Sheet, Specificity, Value};
use crate::dom::{NodeData, NodeRef};

pub type PropertyMap = HashMap<String, Value>;

#[derive(Debug)]
pub struct StyledNodeRef(Rc<RefCell<StyledNode>>);

impl Deref for StyledNodeRef {
    type Target = Rc<RefCell<StyledNode>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct StyledNode {
    pub node: NodeRef,
    pub specified_values: PropertyMap,
    pub children: Vec<StyledNodeRef>,
}

impl From<StyledNodeRef> for String {
    fn from(styled_node: StyledNodeRef) -> String {
        let mut output = String::new();

        match styled_node.node.data {
            NodeData::Element { tag, attrs, .. } => {
                output.push('<');
                output.push_str(tag);

                let attrs_str = attrs.iter().fold("".to_owned(), |acc, x| {
                    format!("{} {}=\"{}\"", acc, x.0, x.1)
                });

                output.push_str(&attrs_str);

                output.push_str(" style=\"");

                let mut specified_values: Vec<_> = styled_node.specified_values.iter().collect();
                specified_values.sort_by(|&(a, _), &(b, _)| a.cmp(b));

                for (key, value) in specified_values {
                    output.push_str(&format!("{}:{};", key, String::from(value)));
                }
                output.push('"');

                output.push('>');

                for child in &styled_node.children {
                    output.push_str(&String::from(child));
                }

                output.push_str("</");
                output.push_str(tag);
                output.push('>');
            }
            NodeData::Text(t) => {
                output.push_str(t);
            }
        }

        output
    }
}

#[derive(PartialEq)]
pub enum Display {
    Inline,
    InlineBlock,
    Block,
    None,
}

impl StyledNode {
    pub fn value(&self, name: &str) -> Option<Value> {
        self.specified_values.get(name).cloned()
    }

    pub fn lookup(&self, name: &str, fallback_name: &str, default: &Value) -> Value {
        self.value(name)
            .unwrap_or_else(|| self.value(fallback_name).unwrap_or_else(|| default.clone()))
    }

    pub fn display(&self) -> Display {
        match self.value("display") {
            Some(Value::Keyword(s)) => match &*s {
                "block" => Display::Block,
                "inline-block" => Display::InlineBlock,
                "none" => Display::None,
                _ => Display::Inline,
            },
            _ => Display::Inline,
        }
    }
}

pub fn style_tree<'a>(root: NodeRef, sheet: &'a Sheet) -> StyledNodeRef {
    match root.data {
        NodeData::Element { .. } => StyledNodeRef(Rc::new(RefCell::new(StyledNode {
            node: root,
            specified_values: get_specified_values(root, sheet),
            children: root.children
                .iter()
                .map(|child| style_tree(child, sheet))
                .collect(),
        }))),
        NodeData::Text(_) => StyledNodeRef(Rc::new(RefCell::new(StyledNode {
            node: root,
            specified_values: HashMap::new(),
            children: vec![],
        }))),
    }
}

fn get_specified_values(node: NodeRef, sheet: &Sheet) -> PropertyMap {
    let mut values = HashMap::new();
    let mut rules = matching_rules(node, sheet);

    rules.sort_by(|&(a, _), &(b, _)| a.cmp(&b));
    for (_, rule) in rules {
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }
    values
}

type MatchedRule<'a> = (Specificity, &'a Rule);

fn matching_rules<'a>(node: NodeRef, sheet: &'a Sheet) -> Vec<MatchedRule<'a>> {
    sheet.0.iter().filter_map(|rule| match_rule(node, rule)).collect()
}

fn match_rule<'a>(node: NodeRef, rule: &'a Rule) -> Option<MatchedRule<'a>> {
    rule.selectors.iter()
        .find(|selector| matches(node, selector))
        .map(|selector| (selector.get_specificity(), rule))
}

fn matches(node: NodeRef, selector: &Selector) -> bool {
    match node.data {
        NodeData::Element { tag, attrs: _ } => {
            if selector.tag.iter().any(|name| *tag != *name) {
                return false;
            }

            if selector.id.iter().any(|id| node.get_id().unwrap_or("") != id) {
                return false;
            }

            let node_classes = node.get_classes();
            if selector.class.iter().any(|class| !node_classes.contains(&**class)) {
                return false;
            }

            // TODO: match attrs

            // Only matching selector components
            true
        }
        NodeData::Text(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::css::*;
    use crate::dom::*;
    use crate::style::*;

    #[test]
    fn test_styled_node() {
        let document = elem("html").add_attr("lang", "NL").inner_html(
            r#"
            <head>
                <title>Hello, world!</title>
            </head>
            <body class="bar">
                <h1>Hi!</h1>
                <p>Bye!</p>
            </body>"#,
        );

        let style = sheet().add_rule(
            rule()
                .add_selector(selector().add_tag("body").add_class("foo"))
                .add_selector(selector().add_tag("p"))
                .add_declaration("margin", Value::Keyword("auto".to_owned()))
                .add_declaration("width", Value::Length(24.0, Unit::Px)),
        );

        let actual = style_tree(&document, &style);

        let expected = HashMap::from([
            ("margin".to_owned(), Value::Keyword("auto".to_owned())),
            ("width".to_owned(), Value::Length(24.0, Unit::Px)),
        ]);

        // element p matches selector p
        assert_eq!(actual.children[1].children[1].specified_values, expected);

        // element class bar does not match selector class foo
        assert_eq!(actual.children[1].specified_values, HashMap::new());
    }

    #[test]
    fn test_to_str() {
        let document = elem("html").inner_html(
            r#"
            <body class="bar">
                <h1>Hi!</h1>
                <p>Bye!</p>
            </body>"#,
        );

        let style = sheet().add_rule(
            rule()
                .add_selector(selector().add_tag("body").add_class("foo"))
                .add_selector(selector().add_tag("p"))
                .add_declaration("margin", Value::Keyword("auto".to_owned()))
                .add_declaration("width", Value::Length(24.0, Unit::Px)),
        );

        let actual = style_tree(&document, &style);
        let expected = r#"<html style=""><body class="bar" style=""><h1 style="">Hi!</h1><p style="margin:auto;width:24px;">Bye!</p></body></html>"#;
        assert_eq!(String::from(&actual), expected);
    }
}
