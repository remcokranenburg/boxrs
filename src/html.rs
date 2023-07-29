use crate::dom;

peg::parser! {
    pub grammar html_parser() for str {
        pub rule nodes() -> Vec<dom::NodeRef>
            = n:node()* { n }

        pub rule node() -> dom::NodeRef
            = __ n:(element() / text()) __ { n }

        rule element() -> dom::NodeRef
            = ot:open_tag() c:node()* ct:close_tag() {?
                if ot.0 == ct {
                    Ok(dom::NodeRef::new(dom::Node {
                        data: dom::NodeData::Element {
                            tag: ot.0,
                            attrs: ot.1,
                        },
                        children: c
                    }))
                } else {
                    Err("close tag to match opening tag")
                }
            }

        rule open_tag() -> (String, Vec<(String, String)>)
            = "<" __ t:tag() a:attribute()* __ ">" { (t, a) }

        rule close_tag() -> String
            = "</" __ t:tag() __ ">" { t }

        rule tag() -> String
            = s:$(['a'..='z' | 'A'..='Z']['a'..='z' | 'A'..='Z' | '0'..='9' | '-']*) {
                s.to_owned()
            }

        rule attribute() -> (String, String)
            = __ k:key() __ "=" __ v:value() __ { (k, v) }

        rule key() -> String
            = s:$(['a'..='z' | 'A'..='Z']['a'..='z' | 'A'..='Z' | '0'..='9' | '-']*) {
                s.to_owned()
            }

        rule value() -> String
            = "\"" s:$([^'"']*) "\"" { s.to_owned() }

        rule text() -> dom::NodeRef
            = t:$([^'<']+) { dom::text(t) }

        rule __
            = whitespace()*

        rule whitespace()
            = " " / "\r" / "\n" / "\t"
    }
}

impl From<String> for dom::NodeRef {
    fn from(s: String) -> Self {
        html_parser::node(&s).unwrap()
    }
}

impl From<&str> for dom::NodeRef {
    fn from(s: &str) -> Self {
        html_parser::node(s).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::dom::{elem, Node};

    #[test]
    fn test_from_string() {
        let expected = elem("html")
            .add_attr("lang", "NL")
            .add_child(elem("head").add_child(elem("title").add_text("Hello, world!")))
            .add_child(
                elem("body")
                    .add_child(elem("h1").add_text("Hi!"))
                    .add_child(elem("p").add_text("Bye!")),
            );
        let actual = "
            <html lang=\"NL\">
                <head>
                    <title>Hello, world!</title>
                </head>
                <body>
                    <h1>Hi!</h1>
                    <p>Bye!</p>
                </body>
            </html>
        ";
        assert_eq!(Node::from(actual), expected);
    }
}
