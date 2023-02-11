pub struct Sheet(Vec<Rule>);

impl From<&Sheet> for String {
    fn from(sheet: &Sheet) -> String {
        let Sheet(rules) = sheet;
        rules.iter().fold(
            "".to_owned(),
            |acc, rule| format!("{}{}", acc, String::from(rule)))
    }
}

pub struct Rule(Vec<RuleItem>);

impl From<&Rule> for String {
    fn from(rule: &Rule) -> String {
        let Rule(items) = rule;
        items.iter().fold(
            "".to_owned(),
            |acc, item| format!("{}{}", acc, String::from(item)))
    }
}

pub enum RuleItem {
    Selector(Vec<SelectorItem>),
    Declaration {
        name: String,
        value: Value
    },
}

pub use RuleItem::{Selector, Declaration};

fn declare(name: &str, value: Value) -> RuleItem {
    Declaration { name: name.to_owned(), value: value }
}

impl From<&RuleItem> for String {
    // TODO: first print selectors, then declarations
    fn from(item: &RuleItem) -> String {
        match item {
            Selector(ref selector_item) => selector_item
                .iter()
                .fold("".to_owned(), |acc, x| format!("{}{}", acc, String::from(x))),
            Declaration { ref name, ref value } =>
                format!("{}:{};", name, String::from(value)),
        }
    }
}

pub enum AttrOp {
    Eq,
}

pub use AttrOp::Eq;

impl From<&AttrOp> for String {
    fn from(op: &AttrOp) -> String {
        match op {
            Eq => "=".to_owned(),
        }
    }
}

pub enum SelectorItem {
    Tag(String),
    Attr(String, AttrOp, String),
}

pub use SelectorItem::{Tag, Attr};

impl From<&SelectorItem> for String {
    fn from(selector: &SelectorItem) -> String {
        match selector {
            Tag(ref tag) => String::from(tag),
            Attr(ref name, ref op, ref value) =>
                format!("[{}{}\"{}\"]", name, String::from(op), value),
        }
    }
}

impl From<&str> for SelectorItem {
    fn from(selector_str: &str) -> SelectorItem {
        if selector_str.starts_with("[") {
            panic!("Not implemented");
        } else {
            Tag(selector_str.to_owned())
        }
    }
}

fn tag(tag: &str) -> SelectorItem {
    Tag(tag.to_owned())
}

fn attr(name: &str, op: AttrOp, value: &str) -> SelectorItem {
    Attr(name.to_owned(), op, value.to_owned())
}

pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    Color(u8, u8, u8, u8)
}

pub use Value::{Keyword, Length, Color};

fn keyword(word: &str) -> Value {
    Keyword(word.to_owned())
}

impl From<&Value> for String {
    fn from(value: &Value) -> String {
        match value {
            Keyword(ref s) => String::from(s),
            Length(v, ref u) => format!("{}{}", v, String::from(u)),
            Color(r, g, b, a) => format!("rgba({},{},{},{})", r, g, b, a),
        }
    }
}

pub enum Unit {
    Px,
}

pub use Unit::Px;

impl From<&Unit> for String {
    fn from(unit: &Unit) -> String {
        match unit {
            Px => "px".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    // #[test]
    // fn test_to_string() {
    //     let actual = vec![
    //         rule(vec![
    //             select(Some("body"), vec![("class", "foo")]),
    //             select(Some("p"), vec![]),
    //         ], vec![
    //             declare("margin", Value::Keyword("auto")),
    //             declare("width", Value::Length(24.0, Unit::Px)),
    //         ]),
    //     ];
    //     let expected = "body[class=foo],p{margin:auto,width:24.0px}";
    //     assert_eq!(String::from(actual), expected);
    // }

    // fn alt() {
    //     let actual = vec![
    //         select(Some("body"), vec![("class", "foo")])
    //             .select(Some("p"), vec![])
    //             .declare("margin", Value::Keyword("auto"))
    //             .declare("width", Value::Length(24.0, Unit::Px)),
    //     ]
    // }

    // fn alt2() {
    //     let actual = Sheet(vec![
    //         Rule(vec![
    //             select(vec![Selector::Tag("body"), Selector::Attr("class", "foo")]),
    //             select(vec![tag("p")]),
    //             declare("margin", Value::Keyword("auto")),
    //             declare("width", Value::Length(24.0, Unit::Px)),
    //         ])
    //     ]);
    // }

    use crate::css::*;

    #[test]
    fn test_to_string() {
        let actual = Sheet(vec![
            Rule(vec![
                Selector(vec![tag("body"), attr("class", Eq, "foo")]),
                Selector(vec![tag("p")]),
                declare("margin", keyword("auto")),
                declare("width", Length(24.0, Px)),
            ]),
        ]);
        let expected = "body[class=\"foo\"],p{margin:auto;width:24.0px;}";
        assert_eq!(String::from(&actual), expected);
    }
}
