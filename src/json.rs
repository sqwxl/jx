use std::{collections::HashMap, fmt::Display};

use serde_json::Value;

use crate::style::{StyledJson, Styler};

#[derive(Clone, Default)]
pub struct Pointer {
    pub path: Vec<String>,
    pub depth: usize,
}

impl Pointer {
    fn new(path: Vec<String>, depth: usize) -> Self {
        Self { path, depth }
    }

    fn len(&self) -> usize {
        self.path.len()
    }

    pub fn active_path(&self) -> &[String] {
        &self.path[..self.depth]
    }

    fn current(&self) -> Option<&String> {
        self.path.get(self.depth - 1)
    }

    fn current_mut(&mut self) -> Option<&mut String> {
        self.path.get_mut(self.depth - 1)
    }

    fn parent_pointer(&self) -> Option<Self> {
        if self.depth > 0 {
            let depth = self.depth - 1;
            let path = self.path[..depth].to_vec();
            Some(Self::new(path, depth))
        } else {
            None
        }
    }

    fn move_to(&mut self, s: &str) {
        self.forget();
        *self.current_mut().unwrap() = s.to_owned();
    }

    fn push(&mut self, s: &str) {
        self.path.push(s.to_owned());
        self.depth += 1;
    }

    fn forget(&mut self) {
        self.path.truncate(self.depth);
    }

    fn forward(&mut self) -> bool {
        if self.depth < self.len() {
            self.depth += 1;
            true
        } else {
            false
        }
    }

    fn rewind(&mut self) -> bool {
        if self.depth > 0 {
            self.depth -= 1;
            true
        } else {
            false
        }
    }

    fn resolved(&self) -> String {
        let s = self.path[..self.depth].join("/");
        eprintln!("resolved: {}", s);
        if s.is_empty() {
            s
        } else {
            format!("/{}", s)
        }
    }
}

impl Display for Pointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.path
            .iter()
            .take(self.depth)
            .for_each(|p| write!(f, ".{}", p).unwrap());

        Ok(())
    }
}

/// A JSON value with a path to the current active node.
pub struct Json {
    pub value: Value,
    pub pointer: Pointer,
    pub style_map: HashMap<String, StyledJson>,
}

impl Json {
    pub fn new(value: &Value) -> Self {
        Self {
            value: value.clone(),
            pointer: Pointer::new(vec![], 0),
            style_map: HashMap::new(),
        }
    }

    pub fn go_in(&mut self) -> bool {
        if self.pointer.forward() {
            true
        } else if let Some(c) = self.first_child() {
            self.pointer.push(&c);
            true
        } else {
            false
        }
    }

    pub fn go_out(&mut self) -> bool {
        self.pointer.rewind()
    }

    pub fn go_prev(&mut self) -> bool {
        if let Some(s) = self.prev_sibling() {
            self.pointer.move_to(&s);
            true
        } else {
            self.go_out()
        }
    }

    pub fn go_next(&mut self) -> bool {
        if let Some(s) = self.next_sibling() {
            self.pointer.move_to(&s);
            true
        } else {
            self.go_out()
        }
    }

    pub fn resolve_pointer(&self, pointer_str: Option<&str>) -> Option<&Value> {
        self.value
            .pointer(pointer_str.unwrap_or(&self.pointer.resolved()))
    }

    /// Gets the first child of an object or array
    pub fn first_child(&self) -> Option<String> {
        if let Some(v) = self.resolve_pointer(None) {
            match v {
                Value::Object(o) => {
                    if let Some(key) = o.keys().next() {
                        return Some(key.to_owned());
                    }
                }
                Value::Array(a) => {
                    if !a.is_empty() {
                        return Some("0".to_owned());
                    }
                }
                _ => {}
            }
        }

        None
    }

    #[allow(dead_code)]
    /// Gets the last child of an object or array
    pub fn last_child(&self) -> Option<String> {
        if let Some(v) = self.resolve_pointer(None) {
            match v {
                Value::Object(o) => {
                    if let Some(key) = o.keys().last() {
                        return Some(key.to_owned());
                    }
                }
                Value::Array(a) => {
                    if !a.is_empty() {
                        let last_idx = a.len() - 1;
                        return Some(last_idx.to_string());
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn pointer_parent_value(&self) -> Option<&Value> {
        if let Some(parent) = self.pointer.parent_pointer() {
            self.resolve_pointer(Some(&parent.resolved()))
        } else {
            None
        }
    }

    /// Gets the previous sibling element for a given pointer index.
    /// If `idx` is `None`, the last element in the pointer is used.
    pub fn prev_sibling(&self) -> Option<String> {
        if let Some(v) = self.pointer_parent_value() {
            match v {
                Value::Object(o) => {
                    let key_idx = o
                        .keys()
                        .position(|k| k == self.pointer.current().unwrap())
                        .unwrap();
                    if key_idx > 0 {
                        return Some(o.keys().nth(key_idx - 1).unwrap().to_string());
                    }
                }
                Value::Array(_) => {
                    let idx = self.pointer.current().unwrap().parse::<usize>().unwrap();
                    if idx > 0 {
                        return Some((idx - 1).to_string());
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Gets the next sibling element for a given pointer index.
    /// If `idx` is `None`, the last element in the pointer is used.
    pub fn next_sibling(&self) -> Option<String> {
        if let Some(v) = self.pointer_parent_value() {
            match v {
                Value::Object(o) => {
                    let key_idx = o
                        .keys()
                        .position(|k| k == self.pointer.current().unwrap())
                        .unwrap();
                    if key_idx < (o.keys().len() - 1) {
                        return Some(o.keys().nth(key_idx + 1).unwrap().to_string());
                    }
                }
                Value::Array(a) => {
                    let idx = self.pointer.current().unwrap().parse::<usize>().unwrap();
                    if idx < a.len() - 1 {
                        return Some((idx + 1).to_string());
                    }
                }
                _ => {}
            }
        }
        None
    }

    pub fn style_json(&mut self) -> StyledJson {
        self.style_map
            .entry(self.pointer.active_path().to_owned().join("").to_string())
            .or_insert(Styler::style_json(&self.value, &self.pointer))
            .to_owned()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const J: &str = r#"{
    "a": "John Doe",
    "b": 43,
    "c": null,
    "d": true,
    "e": [
        "foo",
        {
            "bar": ["baz", {"qux": "quux"}],
            "corge": "grault"
        }
    ]
}"#;

    fn get_json_value() -> Value {
        serde_json::from_str(J).unwrap()
    }

    fn get_state() -> Json {
        Json::new(&get_json_value())
    }

    #[test]
    fn test_pointer_str() {
        let state = get_state();
        assert_eq!(state.pointer.resolved().as_str(), "");
    }

    #[test]
    fn test_pointer_value() {
        let state = get_state();
        assert_eq!(state.resolve_pointer(None), Some(&get_json_value()));
    }

    #[test]
    fn test_first_child() {
        let state = get_state();
        assert_eq!(state.first_child(), Some("a".to_string()));
    }

    #[test]
    fn test_last_child() {
        let state = get_state();
        assert_eq!(state.last_child(), Some("e".to_string()));
    }

    #[test]
    fn test_move_on_primitive() {
        let mut state = Json::new(&json!("foo"));
        state.go_in();
        assert_eq!(state.pointer.len(), 0);
        state.go_out();
        assert_eq!(state.pointer.len(), 0);
        state.go_prev();
        assert_eq!(state.pointer.len(), 0);
        state.go_next();
        assert_eq!(state.pointer.len(), 0);
    }

    #[test]
    fn test_go_in_out() {
        let mut state = get_state();
        state.go_in();
        assert_eq!(state.pointer.path, vec!["a"]);
        state.go_out();
        assert_eq!(state.pointer.path, vec!["a"]);
    }

    #[test]
    fn test_go_down_up() {
        let mut state = get_state();
        state.go_in();
        state.go_next();
        assert_eq!(state.pointer.path, vec!["b"]);
        state.go_prev();
        assert_eq!(state.pointer.path, vec!["a"]);
    }

    #[test]
    fn test_go_in_array() {
        let mut state = get_state();

        state.go_in(); // "a"
        state.go_next(); // "b"
        state.go_next(); // "c"
        state.go_next(); // "d"
        state.go_next(); // "e" (array)
        state.go_in(); // "0"
        assert_eq!(state.pointer.path, vec!["e", "0"]);
    }

    #[test]
    fn test_go_deep() {
        let mut state = get_state();

        state.go_in(); // "a"
        state.go_next(); // "b"
        state.go_next(); // "c"
        state.go_next(); // "d"
        state.go_next(); // "e" (array)
        state.go_in(); // "0"
        state.go_next(); // "1"
        state.go_in(); // "bar" (object)
        state.go_in(); // "0" (array)
        state.go_next(); // "1"
        state.go_in(); // "qux" (object)
        assert_eq!(state.pointer.path, vec!["e", "1", "bar", "1", "qux"]);
    }
}
