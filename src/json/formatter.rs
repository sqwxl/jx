use std::{borrow::Borrow, collections::HashMap, rc::Rc};

use serde_json::{Map, Value};

use crate::{
    json::{Pointer, Token},
    style::{StyleClass, StyledLine, StyledString, INDENT},
};

pub struct PointerData {
    pub node_type: NodeType,
    pub children: usize,
    pub bounds: (usize, usize),
}

/// A map from tokens to first and last lines matching the formatted value.
pub type PointerMap = HashMap<Vec<Token>, PointerData>;

#[derive(Default, Clone, Copy)]
pub enum NodeType {
    #[default]
    Object,
    Array,
    Value,
}

impl From<&Value> for NodeType {
    fn from(value: &Value) -> Self {
        match value {
            Value::Object(_) => NodeType::Object,
            Value::Array(_) => NodeType::Array,
            _ => NodeType::Value,
        }
    }
}

pub struct Formatter {
    depth: usize,
    value: Rc<Value>,
    node_type: Option<NodeType>,
    tokens: Vec<Token>,
    lines: Vec<StyledLine>,
    pointer_map: PointerMap,
}

impl Formatter {
    pub fn format(value: Rc<Value>) -> (Vec<StyledLine>, PointerMap) {
        let value_clone = Rc::clone(&value);
        let mut formatter = Self {
            depth: 0,
            value,
            node_type: None,
            tokens: vec![],
            lines: vec![],
            pointer_map: HashMap::new(),
        };

        let lines = formatter.format_value(value_clone.borrow());

        (lines, formatter.pointer_map)
    }

    fn format_value(&mut self, value: &Value) -> Vec<StyledLine> {
        match value {
            Value::Object(obj) => self.format_object(obj),
            Value::Array(arr) => self.format_array(arr),
            _ => self.format_primitive(value),
        };

        self.lines.clone()
    }

    fn update_map(&mut self) {
        self.pointer_map
            .entry(self.tokens.clone())
            .and_modify(
                |PointerData {
                     bounds: (_, last), ..
                 }| *last = self.lines.len() - 1,
            )
            .or_insert_with(|| {
                let value = self
                    .value
                    .pointer(&Pointer::json_pointer(&self.tokens))
                    .expect("should have value");

                let children: usize = value
                    .as_object()
                    .and_then(|o| Some(o.len()))
                    .or_else(|| value.as_array().and_then(|a| Some(a.len())))
                    .unwrap_or(0);

                PointerData {
                    node_type: NodeType::from(value),
                    children,
                    bounds: (self.lines.len() - 1, self.lines.len() - 1),
                }
            });
    }

    fn push_token(&mut self, token: &Token) {
        self.tokens.push(token.clone());
    }

    fn pop_token(&mut self) {
        self.tokens.pop();
    }

    fn set_token(&mut self, token: &Token) {
        if self.tokens.is_empty() {
            self.push_token(token);
        } else {
            let last = self.tokens.last_mut().expect("should have element");
            *last = token.clone();
        }
    }

    fn new_line(&mut self) {
        self.lines.push(StyledLine {
            line_number: self.lines.len(),
            indent: self.depth * INDENT,
            pointer: self.tokens.clone(),
            elements: vec![],
        });

        self.update_map();
    }

    fn append_line(&mut self, element: StyledString) {
        if self.lines.is_empty() {
            self.new_line();
        }

        self.lines
            .last_mut()
            .expect("should have element")
            .push(element);
    }

    fn extend_line(&mut self, elements: Vec<StyledString>) {
        if self.lines.is_empty() {
            self.new_line();
        }

        self.lines
            .last_mut()
            .expect("should have element")
            .extend(elements);
    }

    fn open_bracket(&mut self, symbol: &str) {
        self.append_line(format_punct(symbol));
        self.depth += 1;
    }

    fn close_bracket(&mut self, symbol: &str) {
        self.depth -= 1;
        self.new_line();

        self.append_line(format_punct(symbol));
    }

    fn format_object(&mut self, object: &Map<String, Value>) {
        self.open_bracket("{");

        for (idx, (key, value)) in object.iter().enumerate() {
            let token: Token = key.clone().into();

            if idx == 0 {
                self.push_token(&token);
            } else {
                self.set_token(&token);
            }

            self.new_line();

            self.extend_line(format_key(&token.as_key().unwrap()));

            self.format_value(value);

            if idx < object.len() - 1 {
                self.append_line(format_punct(","));
            }
        }

        if !object.is_empty() {
            self.tokens.pop();
        }

        self.close_bracket("}");
    }

    fn format_array(&mut self, array: &[Value]) {
        self.open_bracket("[");

        for (idx, value) in array.iter().enumerate() {
            let token = Token::Index(idx);
            if token == 0 {
                self.push_token(&token);
            } else {
                self.set_token(&token);
            }

            self.new_line();

            self.format_value(value);

            if idx < array.len() - 1 {
                self.append_line(format_punct(","));
            }
        }

        if !array.is_empty() {
            self.tokens.pop();
        }

        self.close_bracket("]");
    }

    fn format_primitive(&mut self, value: &Value) {
        let content = match value {
            Value::Number(n) => format_number(&n.to_string()),
            Value::Bool(b) => format_bool(&b.to_string()),
            Value::Null => format_null(),
            Value::String(s) => {
                self.extend_line(double_quote(format_string(s)));
                return;
            }
            _ => panic!("Unexpected value type; should be a primitive."),
        };

        self.append_line(content);
    }
}

fn surround_punct(styled_string: StyledString, before: &str, after: &str) -> Vec<StyledString> {
    vec![format_punct(before), styled_string, format_punct(after)]
}

fn double_quote(styled_string: StyledString) -> Vec<StyledString> {
    surround_punct(styled_string, "\"", "\"")
}

fn format_key(key: &str) -> Vec<StyledString> {
    [double_quote(format_string(key)), vec![format_punct(": ")]].concat()
}

fn format_string(string: &str) -> StyledString {
    StyledString(string.to_owned(), StyleClass::String)
}

fn format_number(number: &str) -> StyledString {
    StyledString(number.to_owned(), StyleClass::Number)
}

fn format_bool(bool: &str) -> StyledString {
    StyledString(bool.to_owned(), StyleClass::Bool)
}

fn format_null() -> StyledString {
    StyledString("null".to_owned(), StyleClass::Null)
}

fn format_punct(punct: &str) -> StyledString {
    StyledString(punct.into(), StyleClass::Punct)
}

fn format_fold_count(n: usize) -> StyledString {
    StyledString(format!(" ({}) ", n), StyleClass::FoldCount)
}

pub fn bracket_fold(n: usize) -> Vec<StyledString> {
    let fold = format_fold_count(n);
    surround_punct(fold, "[", "]")
}

pub fn curly_fold(n: usize) -> Vec<StyledString> {
    let fold = format_fold_count(n);
    surround_punct(fold, "{", "}")
}
