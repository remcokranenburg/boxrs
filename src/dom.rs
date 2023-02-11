use crate::html::Parser;

#[derive(Debug)]
pub enum Node {
    Element {
        tag: String,
        attrs: Vec<(String, String)>,
        children: Vec<Node>,
    },
    Text(String),
}

impl Node {
    pub fn elem(tag: String) -> Self {
        Node::Element {
            tag: tag,
            attrs: vec![],
            children: vec![],
        }
    }

    pub fn text(t: String) -> Self {
        Node::Text(t)
    }

    pub fn add_children(mut self, c: Vec<Self>) -> Self {
        match self {
            Node::Element { ref mut children, .. } => {
                for item in c {
                    children.push(item)
                }
            }
            Node::Text(ref _t) => (),
        }
        self
    }

    pub fn add_attr(mut self, kv: (String, String)) -> Self {
        match self {
            Node::Element { ref mut attrs, .. } => {
                attrs.push(kv);
            },
            Node::Text(ref _t) => (),
        }
        self
    }

    pub fn add_attrs(mut self, kvs: Vec<(String, String)>) -> Self {
        for kv in kvs {
            self = self.add_attr(kv);
        }
        self
    }

    pub fn inner_html(mut self, html: String) -> Self {
        match self {
            Node::Element { ref mut children, .. } => {
                children.clear();
                children.append(&mut Parser::parse_no_root(html));
            },
            _ => (),
        }
        self
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Node::Element { tag, attrs, children } => {
                match other {
                    Node::Element { tag: other_tag, attrs: other_attrs, children: other_children } => {
                        tag == other_tag && attrs == other_attrs && children == other_children
                    },
                    _ => false,
                }
            },
            Node::Text(t) => {
                if let Node::Text(other_t) = other {
                    t == other_t
                } else {
                    false
                }
            }
        }
    }
}

impl From<&Node> for String {
    fn from(n: &Node) -> String {
        match n {
            Node::Element { tag, attrs, children } => {
                let attrs_str = attrs
                    .iter()
                    .fold("".to_owned(), |acc, x| format!("{} {}=\"{}\"", acc, x.0, x.1));
                let children_str = children
                    .iter()
                    .fold("".to_owned(), |acc, x| format!("{}{}", acc, String::from(x)));
                format!("<{}{}>{}</{}>", &tag, attrs_str, children_str, &tag)
            }
            Node::Text(t) => String::from(t)
        }
    }
}

pub fn elem(tag: String) -> Node {
    Node::elem(tag)
}

pub fn text(t: String) -> Node {
    Node::text(t)
}

#[cfg(test)]
mod tests {
    use crate::dom::{Node, elem, text};
    use crate::html::Parser;

    #[test]
    fn test_to_string() {
        let actual = elem(String::from("html"))
            .add_attr((String::from("lang"), String::from("NL")))
            .add_children(vec![
                elem(String::from("head")).add_children(vec![
                    elem(String::from("title"))
                        .add_children(vec![text(String::from("Hello, world!"))]),
                ]),
                elem(String::from("body")).add_children(vec![
                    elem(String::from("h1")).add_children(vec![text(String::from("Hi!"))]),
                    elem(String::from("p")).add_children(vec![text(String::from("Bye!"))]),
                ]),
            ]);
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
        let actual = elem(String::from("html"))
            .inner_html(String::from("<h1>hello</h1>"));
        let expected = "<html><h1>hello</h1></html>";
        assert_eq!(actual, Node::from(String::from(expected)));
    }
}
