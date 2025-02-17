use std::{collections::HashSet, rc::Rc};

pub use formatter::*;
pub use pointer::Pointer;
pub use token::Token;

use crate::style::StyledLine;

mod formatter;
mod pointer;
mod token;

/// A parsed JSON object with a pointer to the current active node.
pub struct Json {
    pointer: Pointer,
    pub value: Rc<serde_json::Value>,
    pub folds: HashSet<Vec<Token>>,
    pub formatted: Vec<StyledLine>,
    pub pointer_map: PointerMap,
}

impl From<Rc<serde_json::Value>> for Json {
    fn from(value: Rc<serde_json::Value>) -> Self {
        let (formatted, pointer_map) = Formatter::format(Rc::clone(&value));
        Self {
            value,
            pointer: Pointer::new(),
            folds: HashSet::new(),
            formatted,
            pointer_map,
        }
    }
}

impl Json {
    pub fn tokens(&self) -> Vec<Token> {
        self.pointer.tokens()
    }

    pub fn current_token(&self) -> Option<&Token> {
        self.pointer.current_token()
    }

    pub fn path(&self) -> String {
        self.pointer.to_string()
    }

    pub fn toggle_fold(&mut self) -> bool {
        if self
            .get_current_value()
            .is_some_and(|v| !v.is_object() && !v.is_array())
        {
            false
        } else {
            let tokens = self.tokens();

            if self.folds.contains(&tokens) {
                self.unfold(&tokens)
            } else {
                self.fold(tokens)
            }
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

    /// Gets the JSON value at the given pointer location.
    fn get_value_at(&self, pointer_str: &str) -> Option<&serde_json::Value> {
        self.value.pointer(pointer_str)
    }

    /// Gets the JSON value at the current pointer location.
    pub fn get_current_value(&self) -> Option<&serde_json::Value> {
        self.get_value_at(&self.pointer.to_json_pointer())
    }

    /// Gets the JSON value at the parent pointer location.
    fn get_parent_value(&self) -> Option<&serde_json::Value> {
        let tokens = self.pointer.parent_tokens();

        self.get_value_at(&Pointer::json_pointer(&tokens))
    }

    /// Get the key of the value at the current pointer location.
    fn get_key(&self) -> Option<String> {
        self.current_token().map(|s| s.to_string())
    }

    /// Get selection (key and value)
    pub fn get_selection(&self) -> Option<(Option<String>, &serde_json::Value)> {
        let value = self.get_current_value()?;

        Some(if self.value_is_array_element() {
            (None, value)
        } else {
            self.get_key().map_or((None, value), |k| (Some(k), value))
        })
    }

    fn value_is_array_element(&self) -> bool {
        self.get_parent_value().is_some_and(|v| v.is_array())
    }

    /// Gets the first child of an object or array
    fn first_child(&self) -> Option<Token> {
        if let Some(v) = self.get_current_value() {
            match v {
                serde_json::Value::Object(o) => {
                    return o.keys().next().map(|key| Token::Key(key.to_owned()));
                }
                serde_json::Value::Array(a) => {
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
        if let Some(v) = self.get_current_value() {
            match v {
                serde_json::Value::Object(o) => {
                    return o.keys().last().map(|key| Token::Key(key.to_owned()));
                }
                serde_json::Value::Array(a) => {
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

        if let Some(v) = self.get_parent_value() {
            match v {
                serde_json::Value::Object(o) => {
                    let key = self.current_token()?.as_key()?;
                    let key_idx = o
                        .keys()
                        .position(|k| *k == key)
                        .expect("key matching current pointer cursor should be present");

                    if key_idx > 0 {
                        return o.keys().nth(key_idx - 1).map(|k| Token::Key(k.to_string()));
                    }
                }
                serde_json::Value::Array(_) => {
                    let idx = self.current_token()?.as_index()?;

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

        if let Some(v) = self.get_parent_value() {
            match v {
                serde_json::Value::Object(o) => {
                    let key = self.current_token()?.as_key()?;
                    let key_idx = o
                        .keys()
                        .position(|k| *k == key)
                        .expect("key matching current pointer cursor should be present");

                    if key_idx < (o.keys().len() - 1) {
                        return o.keys().nth(key_idx + 1).map(|k| Token::Key(k.to_string()));
                    }
                }
                serde_json::Value::Array(a) => {
                    let idx = self.current_token()?.as_index()?;

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

    use serde_json::json;

    use super::*;

    impl From<serde_json::Value> for Json {
        fn from(value: serde_json::Value) -> Self {
            let rc_value = Rc::new(value);
            let (formatted, pointer_map) = Formatter::format(Rc::clone(&rc_value));
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
        assert_eq!(json.get_current_value(), value.pointer(""));
        assert!(json.go_in());
        assert_eq!(json.get_current_value(), value.pointer("/a"));
        assert!(json.go_in());
        assert_eq!(json.get_current_value(), value.pointer("/a/0"));
        assert!(json.go_next());
        assert_eq!(json.get_current_value(), value.pointer("/a/1"));
        assert!(json.go_in());
        assert_eq!(json.get_current_value(), value.pointer("/a/1/~1"));
        assert!(json.go_next());
        assert_eq!(json.get_current_value(), value.pointer("/a/1/~0"));
        assert!(json.go_in());
        assert_eq!(json.get_current_value(), value.pointer("/a/1/~0/0"));
        assert!(json.go_next());
        assert_eq!(json.get_current_value(), value.pointer("/a/1/~0/1"));
        assert!(json.go_out());
        assert_eq!(json.get_current_value(), value.pointer("/a/1/~0"));
        assert!(json.go_out());
        assert_eq!(json.get_current_value(), value.pointer("/a/1"));
        assert!(json.go_out());
        assert_eq!(json.get_current_value(), value.pointer("/a"));
        assert!(json.go_out());
        assert_eq!(json.get_current_value(), value.pointer(""));
        assert!(json.go_in());
        assert_eq!(json.get_current_value(), value.pointer("/a"));
        assert!(json.go_in());
        assert_eq!(json.get_current_value(), value.pointer("/a/1"));
        assert!(json.go_in());
        assert_eq!(json.get_current_value(), value.pointer("/a/1/~0"));
        assert!(json.go_in());
        assert_eq!(json.get_current_value(), value.pointer("/a/1/~0/1"));
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
        assert_eq!(json.current_token(), None)
    }

    #[test]
    fn go_out_array() {
        let mut json = Json::from(json!(["a"]));
        assert!(json.go_in());
        assert!(json.go_out());
        assert_eq!(json.tokens(), vec!["0"]);
        assert_eq!(json.current_token(), None)
    }

    #[test]
    fn go_next_object() {
        let mut json = Json::from(json!({"a": 0, "b": 1}));
        assert!(json.go_in());
        assert!(json.go_next());
        assert_eq!(json.tokens(), vec!["b"]);
        assert_eq!(json.current_token(), Some(&Token::Key("b".to_string())));
        assert!(!json.go_next());
    }

    #[test]
    fn go_next_array() {
        let mut json = Json::from(json!(["a", "b"]));
        assert!(json.go_in());
        assert!(json.go_next());
        assert_eq!(json.tokens(), vec!["1"]);
        assert_eq!(json.current_token(), Some(&Token::Index(1)));
        assert!(!json.go_next());
    }

    #[test]
    fn go_prev_object() {
        let mut json = Json::from(json!({"a": 0, "b": 1}));
        assert!(json.go_in());
        assert!(json.go_next());
        assert!(json.go_prev());
        assert_eq!(json.tokens(), vec!["a"]);
        assert_eq!(json.current_token(), Some(&Token::Key("a".to_string())));
        assert!(!json.go_prev());
    }

    #[test]
    fn go_prev_array() {
        let mut json = Json::from(json!(["a", "b"]));
        assert!(json.go_in());
        assert!(json.go_next());
        assert!(json.go_prev());
        assert_eq!(json.tokens(), vec!["0"]);
        assert_eq!(json.current_token(), Some(&Token::Index(0)));
        assert!(!json.go_prev());
    }
}
