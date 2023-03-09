pub struct Sheet(Vec<Rule>);

impl Sheet {
    pub fn add_rule(mut self, rule: Rule) -> Self {
        self.0.push(rule);
        self
    }
}

impl From<&Sheet> for String {
    fn from(sheet: &Sheet) -> String {
        let Sheet(rules) = sheet;
        rules.iter().fold("".to_owned(), |acc, rule| format!("{}{}", acc, String::from(rule)))
    }
}

pub struct Rule {
    selectors: Vec<Selector>,
    declarations: Vec<Declaration>,
}

impl Rule {
    pub fn add_selector(mut self, selector: Selector) -> Self {
        self.selectors.push(selector);
        self
    }

    pub fn add_declaration(mut self, name: &str, value: Value) -> Self {
        self.declarations.push(Declaration { name: name.to_owned(), value: value });
        self
    }
}

impl From<&Rule> for String {
    fn from(rule: &Rule) -> String {
        let selectors_str = rule.selectors.iter()
            .map(|selector| String::from(selector))
            .collect::<Vec<_>>()
            .join(",");

        let declarations_str = rule.declarations.iter()
            .map(|declaration| String::from(declaration))
            .collect::<Vec<_>>()
            .join(";");

        format!("{}{{{}}}", selectors_str, declarations_str)
    }
}

pub struct Selector {
    _tag: Option<String>,
    _class: Option<String>,
    _id: Option<String>,
    _attr: Option<(String, AttrOp, String)>,
}

impl Selector {
    pub fn tag(mut self, tag_name: &str) -> Self {
        self._tag = Some(tag_name.to_owned());
        self
    }

    pub fn class(mut self, class_name: &str) -> Self {
        self._class = Some(class_name.to_owned());
        self
    }

    pub fn id(mut self, id_name: &str) -> Self {
        self._id = Some(id_name.to_owned());
        self
    }

    pub fn attr(mut self, attr_name: &str, attr_op: AttrOp, attr_value: &str) -> Self {
        self._attr = Some((attr_name.to_owned(), attr_op, attr_value.to_owned()));
        self
    }
}


impl From<&Selector> for String {
    fn from(selector: &Selector) -> String {
        let mut selector_str = String::new();

        if let Some(ref tag_name) = selector._tag {
            selector_str.push_str(&tag_name);
        }

        if let Some(ref class_name) = selector._class {
            selector_str.push('.');
            selector_str.push_str(&class_name);
        }

        if let Some(ref id_name) = selector._id {
            selector_str.push('#');
            selector_str.push_str(&id_name);
        }

        if let Some((ref attr_name, ref attr_op, ref attr_value)) = selector._attr {
            selector_str.push('[');
            selector_str.push_str(&attr_name);
            selector_str.push_str(&String::from(attr_op));
            selector_str.push('"');
            selector_str.push_str(&attr_value);
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

struct Declaration {
    name: String,
    value: Value,
}

impl From<&Declaration> for String {
    fn from(declaration: &Declaration) -> String {
        format!("{}:{}", declaration.name, String::from(&declaration.value))
    }
}

pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    Color(u8, u8, u8, u8)
}

impl From<&Value> for String {
    fn from(value: &Value) -> String {
        match value {
            Value::Keyword(ref s) => String::from(s),
            Value::Length(v, ref u) => format!("{}{}", v, String::from(u)),
            Value::Color(r, g, b, a) => format!("rgba({},{},{},{})", r, g, b, a),
        }
    }
}

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

pub fn sheet() -> Sheet {
    Sheet(vec![])
}

pub fn rule() -> Rule {
    Rule { selectors: vec![], declarations: vec![] }
}

pub fn selector() -> Selector {
    Selector{ _tag: None, _class: None, _id: None, _attr: None }
}


#[cfg(test)]
mod tests {
    use crate::css::*;

    #[test]
    fn test_to_string() {
        let actual = sheet()
            .add_rule(rule()
                .add_selector(selector().tag("body").attr("class", AttrOp::Eq, "foo"))
                .add_selector(selector().tag("p"))
                .add_declaration("margin", Value::Keyword("auto".to_owned()))
                .add_declaration("width", Value::Length(24.0, Unit::Px)));
        let expected = r#"body[class="foo"],p{margin:auto;width:24px}"#;
        assert_eq!(String::from(&actual), expected);
    }
}
