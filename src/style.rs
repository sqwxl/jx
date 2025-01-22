use crossterm::style::Color;
use serde_json::{Map, Value};

use crate::{json::Pointer, screen::Style};

pub const STYLE_INACTIVE: Style = Style(Color::White, Color::Black);
pub const STYLE_SELECTION: Style = Style(Color::Black, Color::Blue);
pub const STYLE_TITLE: Style = Style(Color::Black, Color::White);
pub const STYLE_POINTER: Style = STYLE_TITLE;

#[derive(Clone)]
pub struct StyledJson {
    pub lines: Vec<(usize, StyledStr)>,
    pub selection: (usize, usize),
}

#[derive(Clone, Debug)]
pub struct StyledStr {
    pub style: Style,
    pub text: String,
}

pub struct Styler {
    pointer: Pointer,
    prefix: Option<String>,
    depth: usize,
    path: Vec<String>,
    lines: Vec<(usize, StyledStr)>,
    selection: (usize, usize),
}

impl Styler {
    fn new(pointer: Pointer) -> Self {
        Self {
            pointer,
            prefix: None,
            depth: 0,
            path: Vec::new(),
            lines: Vec::new(),
            selection: (0, 0),
        }
    }

    /// Converts a JSON Value and a pointer to a vector of (Style, String) tuples
    pub fn style_json(value: &Value, pointer: &Pointer) -> StyledJson {
        let mut s = Self::new(pointer.clone());

        s.style_json_recursive(value);

        eprintln!("top: {}", s.selection.0);

        StyledJson {
            lines: s.lines,
            selection: s.selection,
        }
    }

    fn push_line(&mut self, text: &str) {
        let mut text = text.to_owned();
        if let Some(prefix) = &self.prefix {
            text = format!("{}{}", prefix, text);
            self.prefix = None;
        }

        let style = self.match_pointer_style();

        self.update_selection(&style);

        self.lines.push((self.depth, StyledStr { style, text }));
    }

    fn update_selection(&mut self, style: &Style) {
        if let Some(last_style) = self.lines.last().map(|(_, s)| s.style) {
            if last_style != *style {
                match *style {
                    STYLE_SELECTION => {
                        // selection start
                        self.selection.0 = self.lines.len();
                    }
                    STYLE_INACTIVE => {
                        // selection end
                        self.selection.1 = self.lines.len() - 1;
                    }
                    _ => {}
                }
            }
        }
    }

    fn append_str(&mut self, text: &str) {
        if let Some((_, last)) = self.lines.last_mut() {
            last.text.push_str(text);
        }
    }

    fn style_json_recursive(&mut self, json: &Value) {
        match json {
            Value::Object(map) => self.style_map(map),
            Value::Array(arr) => self.style_array(arr),
            _ => self.style_primitive(json),
        }
    }

    fn style_map(&mut self, map: &Map<String, Value>) {
        self.push_line("{");
        self.depth += 1;
        for (idx, (key, value)) in map.iter().enumerate() {
            if idx == 0 {
                self.path.push(key.to_owned());
            } else {
                *self.path.last_mut().unwrap() = key.to_owned();
            }
            self.prefix = Some(format!("\"{}\": ", key));
            self.style_json_recursive(value);
            if idx < map.len() - 1 {
                self.append_str(",");
            }
        }
        if !map.is_empty() {
            self.path.pop();
        }
        self.depth -= 1;
        self.push_line("}");
    }

    fn style_array(&mut self, array: &[Value]) {
        self.push_line("[");
        self.depth += 1;
        for (idx, value) in array.iter().enumerate() {
            if idx == 0 {
                self.path.push(idx.to_string());
            } else {
                *self.path.last_mut().unwrap() = idx.to_string();
            }
            self.style_json_recursive(value);
            if idx < array.len() - 1 {
                self.append_str(",");
            }
        }
        if !array.is_empty() {
            self.path.pop();
        }
        self.depth -= 1;
        self.push_line("]");
    }

    fn style_primitive(&mut self, value: &Value) {
        self.push_line(value.to_string().as_str())
    }

    fn match_pointer_style(&self) -> Style {
        if self.path.starts_with(self.pointer.active_path()) {
            STYLE_SELECTION
        } else {
            STYLE_INACTIVE
        }
    }
}
