use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use serde_json::Value;

pub use formatter::*;
pub use pointer::Pointer;
pub use token::Token;

use crate::style::StyledLine;

mod formatter;
mod pointer;
mod token;

#[derive(Default, Clone, Copy)]
pub enum PointerValue {
    #[default]
    Object,
    Array,
    Primitive,
}

impl From<&Value> for PointerValue {
    fn from(value: &Value) -> Self {
        match value {
            Value::Object(_) => PointerValue::Object,
            Value::Array(_) => PointerValue::Array,
            _ => PointerValue::Primitive,
        }
    }
}

pub struct PointerData {
    pub value: PointerValue,
    pub children: usize,
    pub bounds: (usize, usize),
}

/// A map from tokens to first and last lines matching the formatted value.
pub type PointerMap = HashMap<Vec<Token>, PointerData>;

/// Returns the total width of the formatted JSON
fn measure_width(formatted: &[StyledLine]) -> usize {
    formatted.iter().fold(0, |w, s| {
        w.max(s.indent + s.elements.iter().fold(0, |l, e| l + e.0.chars().count()))
    })
}

/// The main application state. It holds the current pointer (user selection),
/// the active folds, the formatted lines (containing style information) as well as a map
/// containing extra data about each pointer.
pub struct Json {
    pointer: Pointer,
    pub value: Rc<Value>,
    pub folds: HashSet<Vec<Token>>,
    all_folded: bool,
    pub formatted: Vec<StyledLine>,
    pub pointer_map: PointerMap,
    pub width: usize,
}

impl From<Rc<Value>> for Json {
    fn from(value: Rc<Value>) -> Self {
        let mut formatted = vec![];
        let mut pointer_map = PointerMap::new();

        Formatter::format(Rc::clone(&value), &mut formatted, &mut pointer_map);

        let width = measure_width(&formatted);

        Self {
            value,
            folds: HashSet::new(),
            all_folded: false,
            pointer: Pointer::new(),
            formatted,
            width,
            pointer_map,
        }
    }
}

impl Json {
    /// Returns the current pointer as a list of Tokens
    pub fn tokens(&self) -> Vec<Token> {
        self.pointer.tokens()
    }

    /// Returns a reference to the current Token
    pub fn token(&self) -> Option<&Token> {
        self.pointer.current_token()
    }

    /// Returns the current pointer as a period-separated string of tokens
    pub fn period_path(&self) -> String {
        self.pointer.to_string()
    }

    /// Sets the current selection to the given path
    pub fn set_selection(&mut self, tokens: Vec<Token>) {
        self.pointer.set_path(tokens);
    }

    /// Gets the JSON value at the current pointer location.
    pub fn value(&self) -> Option<&Value> {
        self.value.pointer(&self.pointer.to_json_pointer())
    }

    pub fn bounds(&self) -> (usize, usize) {
        match self.pointer_map.get(&self.tokens()) {
            Some(&PointerData { bounds, .. }) => bounds,
            None => (0, self.formatted.len()),
        }
    }

    /// Returns the number of visible lines, accounting for folds
    pub fn visible_line_count(&self) -> usize {
        self.line_to_visible(self.formatted.len()).unwrap_or(0)
    }

    /// Converts absolute line index to visible line index (accounting for folds)
    pub fn line_to_visible(&self, target_line: usize) -> Option<usize> {
        let mut visible = 0;
        let mut line_idx = 0;

        while line_idx < target_line {
            let line = self.formatted.get(line_idx)?;
            visible += 1;
            if self.folds.contains(&line.pointer) {
                if let Some(data) = self.pointer_map.get(&line.pointer) {
                    line_idx = data.bounds.1 + 1;
                    continue;
                }
            }
            line_idx += 1;
        }

        Some(visible)
    }

    /// Returns bounds as visible line indices (accounting for folds)
    pub fn visible_bounds(&self) -> (usize, usize) {
        let (start, end) = self.bounds();
        let visible_start = self.line_to_visible(start).unwrap_or(0);
        let visible_end = self.line_to_visible(end + 1).unwrap_or(visible_start + 1);
        (visible_start, visible_end.saturating_sub(1))
    }

    pub fn toggle_fold(&mut self) -> bool {
        if self
            .value()
            .is_some_and(|v| !v.is_object() && !v.is_array())
        {
            if self.go_out() {
                return self.toggle_fold();
            }
            return false;
        }

        let tokens = self.tokens();

        if self.folds.contains(&tokens) {
            self.unfold(&tokens)
        } else {
            self.fold(tokens)
        }
    }

    pub fn toggle_fold_all(&mut self) -> bool {
        if self.all_folded {
            self.folds.clear();
            self.all_folded = false;
        } else {
            for (tokens, data) in &self.pointer_map {
                if matches!(data.value, PointerValue::Object | PointerValue::Array) {
                    self.folds.insert(tokens.clone());
                }
            }
            self.pointer.rewind();
            self.all_folded = true;
        }

        true
    }

    fn fold(&mut self, tokens: Vec<Token>) -> bool {
        self.folds.insert(tokens)
    }

    fn unfold(&mut self, tokens: &[Token]) -> bool {
        self.folds.remove(tokens)
    }

    pub fn go_in(&mut self) -> bool {
        self.unfold(&self.tokens());

        if !self.pointer.is_at_end() {
            let tokens = self.pointer.forward().tokens();

            self.unfold(&tokens);

            return true;
        }

        if let Some(c) = self.first_child() {
            let tokens = self.pointer.push(&c).forward().tokens();

            self.unfold(&tokens);

            return true;
        }

        false
    }

    pub fn go_out(&mut self) -> bool {
        if self.pointer.is_at_start() {
            false
        } else {
            self.pointer.back();

            true
        }
    }

    pub fn go_prev(&mut self) -> bool {
        if let Some(s) = self.prev_sibling() {
            self.pointer.reset_cursor(&s);
            true
        } else if self.go_out() {
            self.go_prev()
        } else {
            false
        }
    }

    pub fn go_next(&mut self) -> bool {
        if let Some(s) = self.next_sibling() {
            self.pointer.reset_cursor(&s);
            true
        } else if self.go_out() {
            self.go_next()
        } else {
            false
        }
    }

    /// Gets the JSON value at the parent pointer location.
    fn parent_value(&self) -> Option<&Value> {
        let tokens = self.pointer.parent_tokens();

        let json_pointer = &Pointer::json_pointer(&tokens);
        self.value.pointer(json_pointer)
    }

    /// Get selection (key and value)
    pub fn token_value_pair(&self) -> Option<(Option<String>, &Value)> {
        let value = self.value()?;

        Some(if self.value_is_array_element() {
            (None, value)
        } else {
            self.token().map_or((None, value), |k| (k.as_key(), value))
        })
    }

    fn value_is_array_element(&self) -> bool {
        self.parent_value().is_some_and(|v| v.is_array())
    }

    /// Gets the first child of an object or array
    fn first_child(&self) -> Option<Token> {
        if let Some(v) = self.value() {
            match v {
                Value::Object(o) => {
                    return o.keys().next().map(|key| Token::Key(key.to_owned()));
                }
                Value::Array(a) => {
                    if !a.is_empty() {
                        return Some(Token::Index(0));
                    }
                }
                _ => {}
            }
        }

        None
    }

    /// Gets the last child of an object or array
    #[allow(dead_code)]
    fn last_child(&self) -> Option<Token> {
        if let Some(v) = self.value() {
            match v {
                Value::Object(o) => {
                    return o.keys().next_back().map(|key| Token::Key(key.to_owned()));
                }
                Value::Array(a) => {
                    if !a.is_empty() {
                        return Some(Token::Index(a.len() - 1));
                    }
                }
                _ => {}
            }
        }

        None
    }

    /// Gets the previous sibling element for a given pointer index.
    fn prev_sibling(&self) -> Option<Token> {
        if self.pointer.is_at_start() {
            return None;
        }

        if let Some(v) = self.parent_value() {
            match v {
                Value::Object(o) => {
                    let key = self.token()?.as_key()?;
                    let key_idx = o
                        .keys()
                        .position(|k| *k == key)
                        .expect("key matching current pointer cursor should be present");

                    if key_idx > 0 {
                        return o.keys().nth(key_idx - 1).map(|k| Token::Key(k.to_string()));
                    }
                }
                Value::Array(_) => {
                    let idx = self.token()?.as_index()?;

                    if idx > 0 {
                        return Some(Token::Index(idx - 1));
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Gets the next sibling element for a given pointer index.
    fn next_sibling(&self) -> Option<Token> {
        if self.pointer.is_at_start() {
            return None;
        }

        if let Some(v) = self.parent_value() {
            match v {
                Value::Object(o) => {
                    let key = self.token()?.as_key()?;
                    let key_idx = o
                        .keys()
                        .position(|k| *k == key)
                        .expect("key matching current pointer cursor should be present");

                    if key_idx < (o.keys().len() - 1) {
                        return o.keys().nth(key_idx + 1).map(|k| Token::Key(k.to_string()));
                    }
                }
                Value::Array(a) => {
                    let idx = self.token()?.as_index()?;

                    if idx < a.len() - 1 {
                        return Some(Token::Index(idx + 1));
                    }
                }
                _ => {}
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::rc::Rc;

    use serde_json::{json, Value};

    use super::*;

    impl From<Value> for Json {
        fn from(value: Value) -> Self {
            let rc_value = Rc::new(value);
            let mut formatted = Vec::new();
            let width = measure_width(&formatted);
            let mut pointer_map = HashMap::new();
            Formatter::format(Rc::clone(&rc_value), &mut formatted, &mut pointer_map);
            Self {
                value: rc_value,
                pointer: Pointer::new(),
                folds: HashSet::new(),
                all_folded: false,
                formatted,
                width,
                pointer_map,
            }
        }
    }

    #[test]
    fn toggle_fold() {
        let mut json = Json::from(json!({"a": {"b": 0}}));
        json.toggle_fold();
        assert_eq!(json.folds, std::collections::HashSet::from([vec![]]));

        // go_in unfolds the current position (root), then navigates to "a"
        json.go_in();
        json.toggle_fold();
        assert_eq!(
            json.folds,
            std::collections::HashSet::from([vec![Token::Key("a".to_string())]])
        );

        json.go_out();
        json.toggle_fold();
        assert_eq!(
            json.folds,
            std::collections::HashSet::from([vec![], vec![Token::Key("a".to_string())]])
        );
    }

    #[test]
    fn move_around() {
        let value = json!({ "a": [0, { "/": "foo", "~": [true, null] }] });
        let mut json = Json::from(value.clone());
        assert_eq!(json.value(), value.pointer(""));
        assert!(json.go_in());
        assert_eq!(json.value(), value.pointer("/a"));
        assert!(json.go_in());
        assert_eq!(json.value(), value.pointer("/a/0"));
        assert!(json.go_next());
        assert_eq!(json.value(), value.pointer("/a/1"));
        assert!(json.go_in());
        assert_eq!(json.value(), value.pointer("/a/1/~1"));
        assert!(json.go_next());
        assert_eq!(json.value(), value.pointer("/a/1/~0"));
        assert!(json.go_in());
        assert_eq!(json.value(), value.pointer("/a/1/~0/0"));
        assert!(json.go_next());
        assert_eq!(json.value(), value.pointer("/a/1/~0/1"));
        assert!(json.go_out());
        assert_eq!(json.value(), value.pointer("/a/1/~0"));
        assert!(json.go_out());
        assert_eq!(json.value(), value.pointer("/a/1"));
        assert!(json.go_out());
        assert_eq!(json.value(), value.pointer("/a"));
        assert!(json.go_out());
        assert_eq!(json.value(), value.pointer(""));
        assert!(json.go_in());
        assert_eq!(json.value(), value.pointer("/a"));
        assert!(json.go_in());
        assert_eq!(json.value(), value.pointer("/a/1"));
        assert!(json.go_in());
        assert_eq!(json.value(), value.pointer("/a/1/~0"));
        assert!(json.go_in());
        assert_eq!(json.value(), value.pointer("/a/1/~0/1"));
    }

    #[test]
    fn go_in_object_no_op() {
        let mut json = Json::from(json!({}));
        assert!(!json.go_in());
    }

    #[test]
    fn go_in_array_no_op() {
        let mut json = Json::from(json!([]));
        assert!(!json.go_in());
    }

    #[test]
    fn go_in_object() {
        let mut json = Json::from(json!({"a": {"b": 0}}));
        assert!(json.go_in());
        assert_eq!(json.tokens(), vec!["a"]);
        assert!(json.go_in());
        assert_eq!(json.tokens(), vec!["a", "b"]);
    }

    #[test]
    fn go_in_array() {
        let mut json = Json::from(json!([["a"]]));
        assert!(json.go_in());
        assert_eq!(json.tokens(), vec!["0"]);
        assert!(json.go_in());
        assert_eq!(json.tokens(), vec!["0", "0"]);
    }

    #[test]
    fn go_out_object() {
        let mut json = Json::from(json!({"a":0}));
        assert!(json.go_in());
        assert!(json.go_out());
        // cursor is None after go_out from depth 1, so tokens() returns []
        assert_eq!(json.tokens(), Vec::<Token>::new());
        assert_eq!(json.token(), None)
    }

    #[test]
    fn go_out_array() {
        let mut json = Json::from(json!(["a"]));
        assert!(json.go_in());
        assert!(json.go_out());
        // cursor is None after go_out from depth 1, so tokens() returns []
        assert_eq!(json.tokens(), Vec::<Token>::new());
        assert_eq!(json.token(), None)
    }

    #[test]
    fn go_next_object() {
        let mut json = Json::from(json!({"a": 0, "b": 1}));
        assert!(json.go_in());
        assert!(json.go_next());
        assert_eq!(json.tokens(), vec!["b"]);
        assert_eq!(json.token(), Some(&Token::Key("b".to_string())));
        assert!(!json.go_next());
    }

    #[test]
    fn go_next_array() {
        let mut json = Json::from(json!(["a", "b"]));
        assert!(json.go_in());
        assert!(json.go_next());
        assert_eq!(json.tokens(), vec!["1"]);
        assert_eq!(json.token(), Some(&Token::Index(1)));
        assert!(!json.go_next());
    }

    #[test]
    fn go_prev_object() {
        let mut json = Json::from(json!({"a": 0, "b": 1}));
        assert!(json.go_in());
        assert!(json.go_next());
        assert!(json.go_prev());
        assert_eq!(json.tokens(), vec!["a"]);
        assert_eq!(json.token(), Some(&Token::Key("a".to_string())));
        assert!(!json.go_prev());
    }

    #[test]
    fn go_prev_array() {
        let mut json = Json::from(json!(["a", "b"]));
        assert!(json.go_in());
        assert!(json.go_next());
        assert!(json.go_prev());
        assert_eq!(json.tokens(), vec!["0"]);
        assert_eq!(json.token(), Some(&Token::Index(0)));
        assert!(!json.go_prev());
    }
}
