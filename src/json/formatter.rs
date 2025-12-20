use std::{borrow::Borrow, rc::Rc};

use serde_json::{Map, Value};

use crate::{
    json::{Pointer, PointerData, PointerMap, PointerValue, Token},
    style::{StyleClass, StyledLine, StyledString, INDENT},
};

pub struct Formatter<'lines, 'pm> {
    depth: usize,
    value: Rc<Value>,
    tokens: Vec<Token>,
    lines: &'lines mut Vec<StyledLine>,
    pointer_map: &'pm mut PointerMap,
}

impl<'lines, 'pm> Formatter<'lines, 'pm> {
    pub fn format(
        value: Rc<Value>,
        lines: &'lines mut Vec<StyledLine>,
        pointer_map: &'pm mut PointerMap,
    ) {
        let value_clone = Rc::clone(&value);
        let mut formatter = Self {
            depth: 0,
            value,
            tokens: vec![],
            lines,
            pointer_map,
        };

        formatter.format_value(value_clone.borrow());
    }

    fn format_value(&mut self, value: &Value) {
        match value {
            Value::Object(obj) => self.format_object(obj),
            Value::Array(arr) => self.format_array(arr),
            _ => self.format_primitive(value),
        };
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
                    .map(|o| o.len())
                    .or_else(|| value.as_array().map(|a| a.len()))
                    .unwrap_or(0);

                PointerData {
                    value: PointerValue::from(value),
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
    let json = Value::String(string.to_owned()).to_string();
    let escaped = &json[1..json.len() - 1];
    StyledString(escaped.to_owned(), StyleClass::String)
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

pub fn curly_fold(key: Option<&str>, n: usize) -> Vec<StyledString> {
    let fold = format_fold_count(n);
    match key {
        Some(key) => [format_key(key), surround_punct(fold, "{", "}")].concat(),
        None => surround_punct(fold, "{", "}"),
    }
}

pub fn bracket_fold(n: usize) -> Vec<StyledString> {
    let fold = format_fold_count(n);
    surround_punct(fold, "[", "]")
}
