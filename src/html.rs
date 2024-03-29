use crate::dom;

pub struct Parser {
    cursor: usize,
    data: String,
}

impl Parser {
    fn next_char(&self) -> char {
        self.data[self.cursor..].chars().next().unwrap()
    }

    fn starts_with(&self, s: &str) -> bool {
        self.data[self.cursor..].starts_with(s)
    }

    fn eof(&self) -> bool {
        self.cursor >= self.data.len()
    }

    fn consume_char(&mut self) -> char {
        let mut iter = self.data[self.cursor..].char_indices();
        let (_, current_char) = iter.next().unwrap();
        let (next_cursor, _) = iter.next().unwrap_or((1, ' '));
        self.cursor += next_cursor;

        current_char
    }

    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9'))
    }

    fn parse_node(&mut self) -> dom::Node {
        match self.next_char() {
            '<' => self.parse_element(),
            _ => self.parse_text(),
        }
    }

    fn parse_text(&mut self) -> dom::Node {
        dom::text(&self.consume_while(|c| c != '<'))
    }

    fn parse_element(&mut self) -> dom::Node {
        assert!(self.consume_char() == '<');
        let tag_name = self.parse_tag_name();
        let attrs = self.parse_attributes();
        assert!(self.consume_char() == '>');

        let children = self.parse_nodes();

        assert!(self.consume_char() == '<');
        assert!(self.consume_char() == '/');
        assert!(self.parse_tag_name() == tag_name);
        assert!(self.consume_char() == '>');

        dom::elem(&tag_name).add_attrs(attrs).add_children(children)
    }

    fn parse_attr(&mut self) -> (String, String) {
        let name = self.parse_tag_name();
        assert!(self.consume_char() == '=');
        let value = self.parse_attr_value();
        (name, value)
    }

    fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');
        let value = self.consume_while(|c| c != open_quote);
        assert!(self.consume_char() == open_quote);
        value
    }

    fn parse_attributes(&mut self) -> Vec<(String, String)> {
        let mut attributes = vec![];
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }
            let (name, value) = self.parse_attr();
            attributes.push((name, value));
        }
        attributes
    }

    fn parse_nodes(&mut self) -> Vec<dom::Node> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();

            if self.starts_with("<!") {
                self.consume_while(|c| c != '>');
                continue
            }

            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }
        nodes
    }

    pub fn parse_no_root(source: String) -> Vec<dom::Node> {
        Parser {
            cursor: 0,
            data: source,
        }
        .parse_nodes()
    }

    pub fn parse(source: String) -> dom::Node {
        let mut nodes = Parser::parse_no_root(source);

        if nodes.len() == 1 {
            nodes.pop().unwrap()
        } else {
            dom::elem("html").add_children(nodes)
        }
    }
}

impl From<String> for dom::Node {
    fn from(s: String) -> dom::Node {
        Parser::parse(s)
    }
}

impl From<&str> for dom::Node {
    fn from(s: &str) -> dom::Node {
        Parser::parse(s.to_owned())
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
