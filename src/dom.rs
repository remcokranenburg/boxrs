use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::Deref;
use std::rc::Rc;

use crate::html::html_parser;

#[derive(Debug)]
pub struct NodeRef(Rc<RefCell<Node>>);

impl NodeRef {
    pub fn new(n: Node) -> Self {
        Self(Rc::new(RefCell::new(n)))
    }
}

impl Deref for NodeRef {
    type Target = Rc<RefCell<Node>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct Node {
    data: NodeData,
    children: Vec<NodeRef>,
}

#[derive(Debug)]
pub enum NodeData {
    Element {
        tag: String,
        attrs: Vec<(String, String)>,
    },
    Text(String),
}

impl Node {
    pub fn elem(tag: &str) -> NodeRef {
        NodeRef::new(Node {
            data: NodeData::Element {
                tag: tag.to_owned(),
                attrs: vec![],
            },
            children: vec![],
        })
    }

    pub fn text(t: &str) -> NodeRef {
        NodeRef::new(Node {
            data: NodeData::Text(t.to_owned()),
            children: vec![],
        })
    }

    pub fn add_text(self, t: &str) -> Self {
        self.add_child(text(t));
        self
    }

    pub fn add_child(mut self, c: NodeRef) -> Self {
        self.children.push(c);
        self
    }

    pub fn add_children(mut self, cs: Vec<NodeRef>) -> Self {
        for item in cs {
            self.children.push(item);
        }
        self
    }

    pub fn add_attr(mut self, key: &str, value: &str) -> Self {
        if let NodeData::Element { ref mut attrs, .. } = self.data {
            attrs.push((key.to_owned(), value.to_owned()));
        }
        self
    }

    pub fn add_attrs(mut self, kvs: Vec<(String, String)>) -> Self {
        if let NodeData::Element { ref mut attrs, .. } = self.data {
            for item in kvs {
                attrs.push(item);
            }
        }
        self
    }

    pub fn inner_html(mut self, html: &str) -> Self {
        if let NodeData::Element { .. } = self.data {
            self.children.clear();
            self.children.append(&mut html_parser::nodes(html).unwrap());
        }
        self
    }

    pub fn get_id(&self) -> Option<&str> {
        if let NodeData::Element { ref attrs, .. } = self.data {
            for attr in attrs {
                if attr.0 == "id" {
                    return Some(&attr.1);
                }
            }
        }

        None
    }

    pub fn get_classes(&self) -> HashSet<&str> {
        if let NodeData::Element { ref attrs, .. } = self.data {
            for attr in attrs {
                if attr.0 == "class" {
                    return attr.1.split(' ').collect();
                }
            }
        }

        HashSet::new()
    }

    pub fn get_text_content(&self) -> String {
        match self.data {
            NodeData::Element { .. } => {
                let mut content = "".to_owned();
                for c in self.children {
                    content.push_str(&c.0.borrow().get_text_content());
                }
                content
            }
            NodeData::Text(t) => t.to_owned(),
        }
    }

    pub fn get_elements_by_tag_name(&self, tag_name: &str) -> Vec<&Self> {
        match self.data {
            NodeData::Element { ref tag, .. } => {
                let mut result = vec![];

                if tag == tag_name {
                    result.push(self);
                }

                for child in self.children {
                    result.append(&mut child.0.borrow().get_elements_by_tag_name(tag_name));
                }

                result
            }
            NodeData::Text(_) => vec![],
        }
    }
}

impl PartialEq for NodeRef {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        if self.children != other.children {
            return false;
        }

        match self.data {
            NodeData::Element { tag, attrs } => match other.data {
                NodeData::Element { tag: other_tag, attrs: other_attrs } => {
                    tag == other_tag && attrs == other_attrs
                },
                _ => false,
            },
            NodeData::Text(t) => match other.data {
                NodeData::Text(other_t) => t == other_t,
                _ => false,
            }
        }
    }
}

impl From<&Node> for String {
    fn from(n: &Node) -> String {
        match n.data {
            NodeData::Element { tag, attrs } => {
                let attrs_str = attrs.iter().fold("".to_owned(), |acc, x| {
                    format!("{} {}=\"{}\"", acc, x.0, x.1)
                });
                let children_str = n.children.iter().fold("".to_owned(), |acc, x| {
                    format!("{}{}", acc, String::from(&*x.0.borrow()))
                });
                format!("<{}{}>{}</{}>", &tag, attrs_str, children_str, &tag)
            }
            NodeData::Text(t) => String::from(t),
        }
    }
}

pub fn elem(tag: &str) -> NodeRef {
    Node::elem(tag)
}

pub fn text(t: &str) -> NodeRef {
    Node::text(t)
}

#[cfg(test)]
mod tests {
    use crate::dom::{elem, Node};

    #[test]
    fn test_to_string() {
        let actual = elem("html")
            .add_attr("lang", "NL")
            .add_child(elem("head").add_child(elem("title").add_text("Hello, world!")))
            .add_child(
                elem("body")
                    .add_child(elem("h1").add_text("Hi!"))
                    .add_child(elem("p").add_text("Bye!")),
            );
        let expected = "\
            <html lang=\"NL\">\
                <head>\
                    <title>Hello, world!</title>\
                </head>\
                <body>\
                    <h1>Hi!</h1>\
                    <p>Bye!</p>\
                </body>\
            </html>\
        ";
        assert_eq!(String::from(&actual), expected);
    }

    #[test]
    fn test_inner_html() {
        let actual = elem("html").inner_html("<h1>hello</h1>");
        let expected = "<html><h1>hello</h1></html>";
        assert_eq!(actual, Node::from(expected));
    }

    #[test]
    fn test_get_id() {
        let doc = elem("html").add_attr("id", "foo");
        assert_eq!(doc.get_id().unwrap(), "foo");
    }

    #[test]
    fn test_get_classes() {
        let doc = elem("html").add_attr("class", "foo bar");
        let classes = doc.get_classes();
        assert!(classes.contains("foo"));
        assert!(classes.contains("bar"));
    }
}
