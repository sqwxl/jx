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

/// The main application state. It holds the current pointer (user selection),
/// the active folds, the formatted lines (containing style information) as well as a map
/// containing extra data about each pointer.
pub struct Json {
    pointer: Pointer,
    pub value: Rc<Value>,
    pub folds: HashSet<Vec<Token>>,
    pub formatted: Vec<StyledLine>,
    pub pointer_map: PointerMap,
}

impl From<Rc<Value>> for Json {
    fn from(value: Rc<Value>) -> Self {
        let mut formatted = vec![];
        let mut pointer_map = PointerMap::new();

        Formatter::format(Rc::clone(&value), &mut formatted, &mut pointer_map);

        Self {
            value,
            folds: HashSet::new(),
            pointer: Pointer::new(),
            formatted,
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

    pub fn toggle_fold(&mut self) -> bool {
        if self
            .value()
            .is_some_and(|v| !v.is_object() && !v.is_array())
        {
            return false;
        }

        let tokens = self.tokens();

        if self.folds.contains(&tokens) {
            self.unfold(&tokens)
        } else {
            self.fold(tokens)
        }
    }

    pub fn unfold_all(&mut self) {
        self.folds.clear();
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
            self.pointer.rewind();

            true
        }
    }

    pub fn go_prev(&mut self) -> bool {
        if let Some(s) = self.prev_sibling() {
            let tokens = self.pointer.reset_cursor(&s).tokens();

            self.unfold(&tokens);

            true
        } else {
            // self.go_out()
            false
        }
    }

    pub fn go_next(&mut self) -> bool {
        if let Some(s) = self.next_sibling() {
            let tokens = self.pointer.reset_cursor(&s).tokens();

            self.unfold(&tokens);
            true
        } else {
            // self.go_out()
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
                    return o.keys().last().map(|key| Token::Key(key.to_owned()));
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
            let mut pointer_map = HashMap::new();
            Formatter::format(Rc::clone(&rc_value), &mut formatted, &mut pointer_map);
            Self {
                value: rc_value,
                pointer: Pointer::new(),
                folds: HashSet::new(),
                formatted,
                pointer_map,
            }
        }
    }

    #[test]
    fn toggle_fold() {
        let mut json = Json::from(json!({"a": {"b": 0}}));
        json.toggle_fold();
        assert_eq!(json.folds, std::collections::HashSet::from([vec![]]));

        json.go_in();
        json.toggle_fold();
        assert_eq!(
            json.folds,
            std::collections::HashSet::from([vec![], vec![Token::Key("a".to_string())]])
        );

        json.go_out();
        json.toggle_fold();
        assert_eq!(
            json.folds,
            std::collections::HashSet::from([vec![Token::Key("a".to_string())]])
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
        assert_eq!(json.tokens(), vec!["a"]);
        assert_eq!(json.token(), None)
    }

    #[test]
    fn go_out_array() {
        let mut json = Json::from(json!(["a"]));
        assert!(json.go_in());
        assert!(json.go_out());
        assert_eq!(json.tokens(), vec!["0"]);
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
