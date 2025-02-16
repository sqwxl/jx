use std::collections::HashMap;

use crossterm::style::{Attributes, Color, ContentStyle};
use serde_json::{Map, Value};

use crate::json::Token;

pub type StyledString = (String, ContentStyle);

#[derive(Default, Clone)]
pub struct StyledLine {
    pub indent: usize,
    pub elements: Vec<StyledString>,
    pub line_number: usize,
}

impl StyledLine {
    pub fn new() -> Self {
        Self {
            indent: 0,
            elements: vec![],
            line_number: 0,
        }
    }

    pub fn push(&mut self, element: StyledString) {
        self.elements.push(element);
    }

    pub fn extend(&mut self, elements: Vec<StyledString>) {
        self.elements.extend(elements);
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }
}

/// A map from tokens to first and last lines matching the formatted value.
pub type PointerMap = HashMap<Vec<Token>, (usize, usize)>;

pub fn format(value: &Value) -> (Vec<StyledLine>, PointerMap) {
    let mut formatter = Formatter {
        depth: 0,
        tokens: vec![],
        lines: vec![],
        pointer_map: HashMap::new(),
    };

    let lines = formatter.format_value(value);

    (lines, formatter.pointer_map)
}

struct Formatter {
    depth: usize,
    tokens: Vec<Token>,
    lines: Vec<StyledLine>,
    pointer_map: PointerMap,
}

impl Formatter {
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
            .and_modify(|(_, last)| *last = self.lines.len() - 1)
            .or_insert((self.lines.len() - 1, self.lines.len() - 1));
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
            elements: vec![],
            indent: self.depth * INDENT,
            line_number: self.lines.len(),
        });

        self.update_map();
    }

    fn append_to_line(&mut self, element: StyledString) {
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

    fn open_bracket(&mut self) {
        self.append_to_line(self.format_punct("["));
    }

    fn close_bracket(&mut self) {
        self.new_line();

        self.append_to_line(self.format_punct("]"));
    }

    fn bracket_fold(&mut self) {
        self.append_to_line(self.format_punct("[ ... ]"));
    }

    fn open_curly(&mut self) {
        self.append_to_line(self.format_punct("{"));
    }

    fn close_curly(&mut self) {
        self.new_line();

        self.append_to_line(self.format_punct("}"));
    }

    fn curly_fold(&mut self) {
        self.append_to_line(self.format_punct("{ ... }"));
    }

    fn double_quote(&self, styled_string: StyledString) -> Vec<StyledString> {
        vec![
            self.format_punct("\""),
            styled_string,
            self.format_punct("\""),
        ]
    }

    fn format_object(&mut self, object: &Map<String, Value>) {
        // if self.json.folds.contains(&self.tokens) {
        //     self.curly_fold();
        //     return;
        // }

        self.open_curly();
        self.depth += 1;

        for (idx, (key, value)) in object.iter().enumerate() {
            let token: Token = key.clone().into();

            if idx == 0 {
                self.push_token(&token);
            } else {
                self.set_token(&token);
            }

            self.new_line();

            // TODO: handle quoting like string primitives
            self.extend_line(self.format_key(&token.as_key().unwrap()));

            self.format_value(value);

            if idx < object.len() - 1 {
                self.append_to_line(self.format_punct(","));
            }
        }

        if !object.is_empty() {
            self.tokens.pop();
        }

        self.depth -= 1;
        self.close_curly();
    }

    fn format_array(&mut self, array: &[Value]) {
        // if self.json.folds.contains(&self.tokens) {
        //     self.bracket_fold();
        //     return;
        // }

        self.open_bracket();
        self.depth += 1;

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
                self.append_to_line(self.format_punct(","));
            }
        }

        if !array.is_empty() {
            self.tokens.pop();
        }

        self.depth -= 1;
        self.close_bracket();
    }

    fn format_primitive(&mut self, value: &Value) {
        let content = match value {
            Value::Number(n) => self.format_number(&n.to_string()),
            Value::Bool(b) => self.format_bool(&b.to_string()),
            Value::Null => self.format_null(),
            Value::String(s) => {
                self.extend_line(self.double_quote(self.format_string(s)));
                return;
            }
            _ => panic!("Unexpected value type; should be a primitive."),
        };

        self.append_to_line(content);
    }

    fn format_key(&self, key: &str) -> Vec<StyledString> {
        [
            self.double_quote(self.format_string(key)),
            vec![self.format_punct(": ")],
        ]
        .concat()
    }

    fn format_string(&self, string: &str) -> StyledString {
        (string.to_owned(), STYLE_STRING)
    }

    fn format_number(&self, number: &str) -> StyledString {
        (number.to_owned(), STYLE_NUMBER)
    }

    fn format_bool(&self, bool: &str) -> StyledString {
        (bool.to_owned(), STYLE_BOOL)
    }

    fn format_null(&self) -> StyledString {
        ("null".to_owned(), STYLE_NULL)
    }

    fn format_punct(&self, punct: &str) -> StyledString {
        (punct.into(), STYLE_PUNCT)
    }
}

const INDENT: usize = 4;

const STYLE_WHITESPACE: ContentStyle = ContentStyle {
    foreground_color: Some(Color::White),
    background_color: None,
    attributes: Attributes::none(),
    underline_color: None,
};

const STYLE_KEY: ContentStyle = ContentStyle {
    foreground_color: Some(Color::DarkBlue),
    background_color: None,
    attributes: Attributes::none(),
    underline_color: None,
};

const STYLE_STRING: ContentStyle = ContentStyle {
    foreground_color: Some(Color::DarkGreen),
    background_color: None,
    attributes: Attributes::none(),
    underline_color: None,
};

const STYLE_NUMBER: ContentStyle = ContentStyle {
    foreground_color: Some(Color::DarkRed),
    background_color: None,
    attributes: Attributes::none(),
    underline_color: None,
};

const STYLE_BOOL: ContentStyle = ContentStyle {
    foreground_color: Some(Color::DarkBlue),
    background_color: None,
    attributes: Attributes::none(),
    underline_color: None,
};

const STYLE_NULL: ContentStyle = ContentStyle {
    foreground_color: Some(Color::DarkMagenta),
    background_color: None,
    attributes: Attributes::none(),
    underline_color: None,
};

const STYLE_PUNCT: ContentStyle = ContentStyle {
    foreground_color: Some(Color::DarkYellow),
    background_color: None,
    attributes: Attributes::none(),
    underline_color: None,
};

const STYLE_DEFAULT: ContentStyle = ContentStyle {
    foreground_color: Some(Color::White),
    background_color: None,
    attributes: Attributes::none(),
    underline_color: None,
};
