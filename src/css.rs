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

#[derive(Clone, Debug, PartialEq)]
pub struct Selector {
    pub tag: Option<String>,
    pub class: Vec<String>,
    pub id: Option<String>,
    pub attr: Vec<(String, AttrOp, String)>,
}

impl Selector {
    pub fn add_tag(mut self, tag_name: &str) -> Self {
        self.tag = Some(tag_name.to_owned());
        self
    }

    pub fn add_class(mut self, class_name: &str) -> Self {
        self.class.push(class_name.to_owned());
        self
    }

    pub fn add_id(mut self, id_name: &str) -> Self {
        self.id = Some(id_name.to_owned());
        self
    }

    pub fn add_attr(mut self, attr_name: &str, attr_op: AttrOp, attr_value: &str) -> Self {
        self.attr.push((attr_name.to_owned(), attr_op, attr_value.to_owned()));
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

        for c in &selector.class {
            selector_str.push('.');
            selector_str.push_str(&c);
        }

        if let Some(ref id_name) = selector.id {
            selector_str.push('#');
            selector_str.push_str(id_name);
        }

        for a in &selector.attr {
            selector_str.push('[');
            selector_str.push_str(&a.0);
            selector_str.push_str(&String::from(&a.1));
            selector_str.push('"');
            selector_str.push_str(&a.2);
            selector_str.push('"');
            selector_str.push(']');
        }

        selector_str
    }
}

#[derive(Clone, Debug, PartialEq)]
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
        class: vec![],
        id: None,
        attr: vec![],
    }
}

impl From<&str> for Sheet {
    fn from(s: &str) -> Sheet {
        css_parser::rules(s).unwrap()
    }
}


enum SelectorComponent {
    Id(String),
    Class(String),
    Attribute(String, AttrOp, String),
    Tag(String),
    Universal,
}

peg::parser! {
    grammar css_parser() for str {
        pub rule rules() -> Sheet
            = __ r:(css_rule() ** __) __ { Sheet(r) }

        pub rule css_rule() -> Rule
            = s:selectors() __ d:declaration_block() {
                Rule {
                    selectors: s,
                    declarations: d,
                }
            }

        pub rule selectors() -> Vec<Selector>
            = selectors:(simple_selector() ++ selector_delimiter()) {
                let mut ordered_selectors = selectors as Vec<Selector>;
                ordered_selectors.sort_by_key(|s| Reverse(s.get_specificity()));
                ordered_selectors
            }

        rule selector_delimiter()
            = __ "," __

        pub rule simple_selector() -> Selector
            = components:(
                id_selector() /
                class_selector() /
                attribute_selector() /
                tag_selector() /
                universal_selector()
            )+ {?
                let mut ids = vec![];
                let mut classes = vec![];
                let mut attributes = vec![];
                let mut tags = vec![];

                for c in components {
                    match c {
                        SelectorComponent::Id(s) => ids.push(s),
                        SelectorComponent::Class(s) => classes.push(s),
                        SelectorComponent::Attribute(n, o, v) => attributes.push((n, o, v)),
                        SelectorComponent::Tag(s) => tags.push(s),
                        SelectorComponent::Universal => (),
                    }
                }

                if ids.len() > 1 {
                    return Err("a maximum of one id");
                }

                if tags.len() > 1 {
                    return Err("a maximum of one tag");
                }

                Ok(Selector {
                    tag: if tags.len() == 0 { None } else { Some(tags[0].clone()) },
                    class: classes,
                    id: if ids.len() == 0 { None } else { Some(ids[0].clone()) },
                    attr: attributes,
                })
            }

        rule id_selector() -> SelectorComponent
            = "#" s:identifier() { SelectorComponent::Id(s) }

        rule class_selector() -> SelectorComponent
            = "." s:identifier() { SelectorComponent::Class(s) }

        rule attribute_selector() -> SelectorComponent
            = "[" n:identifier() o:operator() v:identifier() "]" { SelectorComponent::Attribute(n, o, v) }

        pub rule operator() -> AttrOp
            = "=" { AttrOp::Eq }

        rule tag_selector() -> SelectorComponent
            = s:identifier() { SelectorComponent::Tag(s) }

        rule universal_selector() -> SelectorComponent
            = "*" { SelectorComponent::Universal }

        pub rule declaration_block() -> Vec<Declaration>
            = __ "{" __ d:(declaration() ** decl_delimiter()) decl_delimiter()? __ "}" __ { d }

        pub rule decl_delimiter()
            = __ ";" __

        pub rule declaration() -> Declaration
            = n:identifier() __ ":" __ v:value() {
                Declaration { name: n, value: v }
            }

        pub rule value() -> Value
            = color_value()
            / length_value()
            / keyword_value()

        pub rule keyword_value() -> Value
            = s:identifier() { Value::Keyword(s.to_owned()) }

        pub rule length_value() -> Value
            = "0" "px"? { Value::Length(0.0, Unit::Px) }
            / n:f32_value() "px" { Value::Length(n, Unit::Px) }

        pub rule color_value() -> Value
            = v:(
                color_rgb_value() /
                color_rgba_value() /
                color_hex_value_six() /
                color_hex_value_three()
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
            = s:$(['a'..='z' | 'A'..='Z'] ['a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_']*) {
                s.to_owned()
            }

        pub rule __
            = (whitespace() / comment())*

        pub rule whitespace()
            = " " / "\r" / "\n" / "\t"

        pub rule comment()
            = "/*" (!"*/"[_])* "*/"
    }
}

#[cfg(test)]
mod tests {
    use crate::css::*;

    #[test]
    fn test_comment() {
        let actual = css_parser::comment("/* foo */");
        let expected = Ok(());
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_selectors() {
        let actual = css_parser::selectors("a");
        let expected = Ok(vec![
            Selector { tag: Some("a".to_owned()), id: None, class: vec![], attr: vec![] },
            // Selector { tag: Some("b".to_owned()), id: None, class: vec![], attr: vec![] },
        ]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_identifier() {
        let actual = css_parser::identifier("a");
        let expected = Ok("a".to_owned());
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_declaration_block() {
        let actual = css_parser::declaration_block(
            "
            {
                foo: bar;
                baz: 42px;
            }
            "
        );
        let expected = Ok(vec![
            Declaration { name: "foo".to_owned(), value: Value::Keyword("bar".to_owned()) },
            Declaration { name: "baz".to_owned(), value: Value::Length(42.0, Unit::Px) },
        ]);
        assert_eq!(actual, expected);
    }

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
    fn test_color_rgb_value() {
        let actual = css_parser::color_value("rgb(1,2,3)");
        let expected = Ok(Value::ColorValue(Color { r: 1, g: 2, b: 3, a: 255 }));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_color_rgba_value() {
        let actual = css_parser::color_value("rgba(1,2,3,4)");
        let expected = Ok(Value::ColorValue(Color { r: 1, g: 2, b: 3, a: 4 }));
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
        let actual = css_parser::color_value("#abc");
        let expected = Ok(Value::ColorValue(Color { r: 170, g: 187, b: 204, a: 255 }));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_color_hex_value_six() {
        let actual = css_parser::color_value("#abcdef");
        let expected = Ok(Value::ColorValue(Color { r: 171, g: 205, b: 239, a: 255 }));
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
