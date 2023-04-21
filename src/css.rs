use std::cmp::Reverse;
use std::str::FromStr;

pub struct Sheet(pub Vec<Rule>);

impl Sheet {
    pub fn add_rule(mut self, rule: Rule) -> Self {
        self.0.push(rule);
        self
    }
}

impl From<&Sheet> for String {
    fn from(sheet: &Sheet) -> String {
        let Sheet(rules) = sheet;
        rules.iter().fold("".to_owned(), |acc, rule| {
            format!("{}{}", acc, String::from(rule))
        })
    }
}

pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

impl Rule {
    pub fn add_selector(mut self, selector: Selector) -> Self {
        self.selectors.push(selector);
        self
    }

    pub fn add_declaration(mut self, name: &str, value: Value) -> Self {
        self.declarations.push(Declaration {
            name: name.to_owned(),
            value,
        });
        self
    }
}

impl From<&Rule> for String {
    fn from(rule: &Rule) -> String {
        let selectors_str = rule
            .selectors
            .iter()
            .map(String::from)
            .collect::<Vec<_>>()
            .join(",");

        let declarations_str = rule
            .declarations
            .iter()
            .map(String::from)
            .collect::<Vec<_>>()
            .join(";");

        format!("{}{{{}}}", selectors_str, declarations_str)
    }
}

pub type Specificity = (usize, usize, usize);

pub struct Selector {
    pub tag: Option<String>,
    pub class: Option<String>,
    pub id: Option<String>,
    pub attr: Option<(String, AttrOp, String)>,
}

impl Selector {
    pub fn add_tag(mut self, tag_name: &str) -> Self {
        self.tag = Some(tag_name.to_owned());
        self
    }

    pub fn add_class(mut self, class_name: &str) -> Self {
        self.class = Some(class_name.to_owned());
        self
    }

    pub fn add_id(mut self, id_name: &str) -> Self {
        self.id = Some(id_name.to_owned());
        self
    }

    pub fn add_attr(mut self, attr_name: &str, attr_op: AttrOp, attr_value: &str) -> Self {
        self.attr = Some((attr_name.to_owned(), attr_op, attr_value.to_owned()));
        self
    }

    pub fn get_specificity(&self) -> Specificity {
        let a = self.id.iter().count();
        let b = self.class.iter().count() + self.attr.iter().count();
        let c = self.tag.iter().count();
        (a, b, c)
    }
}

impl From<&Selector> for String {
    fn from(selector: &Selector) -> String {
        let mut selector_str = String::new();

        if let Some(ref tag_name) = selector.tag {
            selector_str.push_str(tag_name);
        }

        if let Some(ref class_name) = selector.class {
            selector_str.push('.');
            selector_str.push_str(class_name);
        }

        if let Some(ref id_name) = selector.id {
            selector_str.push('#');
            selector_str.push_str(id_name);
        }

        if let Some((ref attr_name, ref attr_op, ref attr_value)) = selector.attr {
            selector_str.push('[');
            selector_str.push_str(attr_name);
            selector_str.push_str(&String::from(attr_op));
            selector_str.push('"');
            selector_str.push_str(attr_value);
            selector_str.push('"');
            selector_str.push(']');
        }

        selector_str
    }
}

pub enum AttrOp {
    Eq,
}

impl From<&AttrOp> for String {
    fn from(op: &AttrOp) -> String {
        match op {
            AttrOp::Eq => "=".to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub value: Value,
}

impl From<&Declaration> for String {
    fn from(declaration: &Declaration) -> String {
        format!("{}:{}", declaration.name, String::from(&declaration.value))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
}

impl Value {
    pub fn to_px(&self) -> f32 {
        match *self {
            Value::Length(f, Unit::Px) => f, // TODO: device-independent pixels
            _ => 0.0,
        }
    }
}

impl From<&Value> for String {
    fn from(value: &Value) -> String {
        match value {
            Value::Keyword(ref s) => String::from(s),
            Value::Length(v, ref u) => format!("{}{}", v, String::from(u)),
            Value::ColorValue(c) => format!("rgba({},{},{},{})", c.r, c.g, c.b, c.a),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Unit {
    Px,
}

impl From<&Unit> for String {
    fn from(unit: &Unit) -> String {
        match unit {
            Unit::Px => "px".to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub fn sheet() -> Sheet {
    Sheet(vec![])
}

pub fn rule() -> Rule {
    Rule {
        selectors: vec![],
        declarations: vec![],
    }
}

pub fn selector() -> Selector {
    Selector {
        tag: None,
        class: None,
        id: None,
        attr: None,
    }
}

impl From<&str> for Sheet {
    fn from(s: &str) -> Sheet {
        let mut parser = Parser {
            cursor: 0,
            data: s.to_owned(),
        };
        Sheet(parser.parse_rules())
    }
}

peg::parser! {
    grammar css_parser() for str {
        pub rule declaration() -> Declaration
            = n:identifier() whitespace()* ":" whitespace()* v:value() {
                Declaration { name: n, value: v }
            }

        pub rule value() -> Value
            = color_value()
            / length_value()
            / keyword_value()

        pub rule keyword_value() -> Value
            = s:identifier() { Value::Keyword(s.to_owned()) }

        pub rule length_value() -> Value
            = "0" { Value::Length(0.0, Unit::Px) }
            / n:f32_value() "px" { Value::Length(n, Unit::Px) }

        pub rule color_value() -> Value
            = v:(
                color_rgb_value() /
                color_rgba_value() /
                color_hex_value_three() /
                color_hex_value_six()
            ) { Value::ColorValue(v) }

        pub rule color_rgb_value() -> Color
            = "rgb(" r:dec_value() "," g:dec_value() "," b:dec_value() ")" {
                Color { r, g, b, a: 255 }
            }

        pub rule color_rgba_value() -> Color
            = "rgba(" r:dec_value() "," g:dec_value() "," b:dec_value() "," a:dec_value() ")" {
                Color { r, g, b, a }
            }

        pub rule color_hex_value_three() -> Color
            = "#" v:hex_value_one()*<3,3> {
                Color {
                    r: v[0] + v[0] * 16,
                    g: v[1] + v[1] * 16,
                    b: v[2] + v[2] * 16,
                    a: 255,
                }
            }
            / expected!("# followed by three hexadecimal digits")

        pub rule color_hex_value_six() -> Color
            = "#" v:hex_value_two()*<3,3> {
                Color {
                    r: v[0],
                    g: v[1],
                    b: v[2],
                    a: 255,
                }
            }
            / expected!("# followed by six hexadecimal digits")

        pub rule f32_value() -> f32
            = n:$(
                "-"? ['0'..='9']+ ("." ['0'..='9']+)? /
                "-"? "." ['0'..='9']+
            ) { f32::from_str(n).unwrap() }

        pub rule dec_value() -> u8
            = n:$(['0'..='9']+) { u8::from_str_radix(n, 10).unwrap() }

        pub rule hex_value_one() -> u8
            = n:$(['0'..='9' | 'a'..='f' | 'A'..='F']) { u8::from_str_radix(n, 16).unwrap() }

        pub rule hex_value_two() -> u8
            = n:$(['0'..='9' | 'a'..='f' | 'A'..='F']*<2,2>) { u8::from_str_radix(n, 16).unwrap() }

        pub rule identifier() -> String
            = s:$(['a'..='z' | 'A'..='Z'] ['a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_']+) {
                s.to_owned()
            }

        pub rule word()
            = [^ ' ' | '\r' | '\n' | '\t']+

        pub rule whitespace()
            = [' ' | '\r' | '\n' | '\t']
    }
}

struct Parser {
    cursor: usize,
    data: String,
}

impl Parser {
    /// Parse a list of rule sets, separated by optional whitespace.
    fn parse_rules(&mut self) -> Vec<Rule> {
        let mut rules = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() {
                break;
            }
            rules.push(self.parse_rule());
        }
        rules
    }

    /// Parse a rule set: `<selectors> { <declarations> }`.
    fn parse_rule(&mut self) -> Rule {
        Rule {
            selectors: self.parse_selectors(),
            declarations: self.parse_declarations(),
        }
    }

    /// Parse a comma-separated list of selectors.
    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();
        loop {
            selectors.push(self.parse_simple_selector());
            self.consume_whitespace();
            match self.next_char() {
                ',' => {
                    self.consume_char();
                    self.consume_whitespace();
                }
                '{' => break,
                c => panic!("Unexpected character {} in selector list", c),
            }
        }
        // Return selectors with highest specificity first, for use in matching.
        selectors.sort_by_key(|s| Reverse(s.get_specificity()));
        selectors
    }

    fn parse_simple_selector(&mut self) -> Selector {
        let mut selector = Selector {
            tag: None,
            id: None,
            class: None,
            attr: None,
        };
        while !self.eof() {
            match self.next_char() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    selector.class = Some(self.parse_identifier());
                }
                // '[' => {
                //     self.consume_char();
                //     let attr = self.parse_attribute();
                //     self.consume_whitespace();
                //     let op = self.consume_char();
                //     self.consume_whitespace();
                //     let value = self.parse_attribute_value();
                //     selector.attr = Some((attr, op, value));
                // }
                '*' => {
                    // universal selector
                    self.consume_char();
                }
                c if valid_identifier_char(c) => {
                    selector.tag = Some(self.parse_identifier());
                }
                _ => break,
            }
        }
        selector
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        assert_eq!(self.consume_char(), '{');
        let mut declarations = Vec::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '}' {
                self.consume_char();
                break;
            }
            declarations.push(self.parse_declaration());
        }
        declarations
    }

    fn parse_declaration(&mut self) -> Declaration {
        let name = self.parse_identifier();
        self.consume_whitespace();
        assert_eq!(self.consume_char(), ':');
        self.consume_whitespace();
        let value = self.parse_value();
        self.consume_whitespace();
        assert_eq!(self.consume_char(), ';');

        Declaration { name, value }
    }

    fn parse_value(&mut self) -> Value {
        match self.next_char() {
            '0'..='9' => self.parse_length(),
            '#' => self.parse_color(),
            _ => Value::Keyword(self.parse_identifier()),
        }
    }

    fn parse_length(&mut self) -> Value {
        Value::Length(self.parse_float(), self.parse_unit())
    }

    fn parse_float(&mut self) -> f32 {
        let s = self.consume_while(|c| matches!(c, '0'..='9' | '.'));
        s.parse().unwrap()
    }

    fn parse_unit(&mut self) -> Unit {
        match &*self.parse_identifier().to_ascii_lowercase() {
            "px" => Unit::Px,
            _ => panic!("unrecognized unit"),
        }
    }

    fn parse_color(&mut self) -> Value {
        assert_eq!(self.consume_char(), '#');
        Value::ColorValue(Color {
            r: self.parse_hex_pair(),
            g: self.parse_hex_pair(),
            b: self.parse_hex_pair(),
            a: 255,
        })
    }

    fn parse_hex_pair(&mut self) -> u8 {
        let s = &self.data[self.cursor..self.cursor + 2];
        self.cursor += 2;
        u8::from_str_radix(s, 16).unwrap()
    }

    fn parse_identifier(&mut self) -> String {
        self.consume_while(valid_identifier_char)
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

    fn consume_char(&mut self) -> char {
        let mut iter = self.data[self.cursor..].char_indices();
        let (_, current_char) = iter.next().unwrap();
        let (next_cursor, _) = iter.next().unwrap_or((1, ' '));
        self.cursor += next_cursor;

        current_char
    }

    fn next_char(&self) -> char {
        self.data[self.cursor..].chars().next().unwrap()
    }

    fn eof(&self) -> bool {
        self.cursor >= self.data.len()
    }
}

fn valid_identifier_char(c: char) -> bool {
    match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => true, // TODO: Include U+00A0 and higher.
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::css::*;

    #[test]
    fn test_declaration() {
        let actual = css_parser::declaration("foo: bar");
        let expected = Ok(Declaration {
            name: "foo".to_owned(),
            value: Value::Keyword("bar".to_owned())
        });
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_color_value() {
        let actual = css_parser::color_value("rgb(1,2,3)");
        let expected = Ok(Value::ColorValue(Color { r: 1, g: 2, b: 3, a: 255 }));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_color_rgb_value() {
        let actual = css_parser::color_rgb_value("rgb(1,2,3)");
        let expected = Ok(Color { r: 1, g: 2, b: 3, a: 255 });
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_color_rgba_value() {
        let actual = css_parser::color_rgba_value("rgba(1,2,3,4)");
        let expected = Ok(Color { r: 1, g: 2, b: 3, a: 4 });
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_hex_value_one() {
        let actual = css_parser::hex_value_one("f");
        let expected = Ok(15);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_hex_value_two() {
        let actual = css_parser::hex_value_two("ff");
        let expected = Ok(255);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_color_hex_value_three() {
        let actual = css_parser::color_hex_value_three("#abc");
        let expected = Ok(Color { r: 170, g: 187, b: 204, a: 255 });
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_color_hex_value_six() {
        let actual = css_parser::color_hex_value_six("#abcdef");
        let expected = Ok(Color { r: 171, g: 205, b: 239, a: 255 });
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_to_string() {
        let actual = sheet().add_rule(
            rule()
                .add_selector(
                    selector()
                        .add_tag("body")
                        .add_attr("class", AttrOp::Eq, "foo"),
                )
                .add_selector(selector().add_tag("p"))
                .add_declaration("margin", Value::Keyword("auto".to_owned()))
                .add_declaration("width", Value::Length(24.0, Unit::Px)),
        );
        let expected = r#"body[class="foo"],p{margin:auto;width:24px}"#;
        assert_eq!(String::from(&actual), expected);
    }

    #[test]
    fn test_from_str() {
        let css = Sheet::from(
            "
            a, b {
                display: block;
                background-color: #ff0000;
                margin: 24px;
                width: 100px;
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

        assert_eq!(css.0[0].selectors[0].tag, Some("a".to_owned()));
        assert_eq!(css.0[0].selectors[1].tag, Some("b".to_owned()));
        assert_eq!(css.0[0].declarations[0].name, "display".to_owned());

        assert_eq!(css.0[1].selectors[0].tag, Some("c".to_owned()));
    }
}
