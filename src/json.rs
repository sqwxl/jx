use std::{collections::BTreeSet, fmt::Display};

use serde_json::Value;

#[derive(Clone, Default, PartialEq, Eq, Hash)]
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

    fn move_to(&mut self, s: &str) -> &mut Self {
        self.forget();
        *self.current_mut().unwrap() = s.to_owned();

        self
    }

    fn push(&mut self, s: &str) -> &mut Self {
        self.path.push(s.to_owned());
        self.depth += 1;

        self
    }

    fn forget(&mut self) -> &mut Self {
        self.path.truncate(self.depth);

        self
    }

    /// Advances the pointer depth, if possible.
    fn forward(&mut self) -> bool {
        if self.depth < self.len() {
            self.depth += 1;
            true
        } else {
            false
        }
    }

    /// Rewinds the pointer depth
    fn rewind(&mut self) -> bool {
        if self.depth > 0 {
            self.depth -= 1;
            true
        } else {
            false
        }
    }

    fn to_path_string(&self) -> String {
        let s = self.path[..self.depth].join("/");

        if s.is_empty() {
            s
        } else {
            format!("/{}", s)
        }
    }
}

impl Display for Pointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.depth == 0 {
            write!(f, ".")?;
            return Ok(());
        }

        self.path
            .iter()
            .take(self.depth)
            .for_each(|p| write!(f, ".{}", p).unwrap());

        Ok(())
    }
}

/// A parsed JSON object with a pointer to the current active node.
#[derive(PartialEq, Eq, Hash)]
pub struct Json {
    pub value: Value,
    pub pointer: Pointer,
    folds: BTreeSet<String>,
}

impl Json {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            pointer: Pointer::new(vec![], 0),
            folds: BTreeSet::new(),
        }
    }

    pub fn toggle_fold(&mut self, path: Option<String>) -> bool {
        let path = path.unwrap_or_else(|| self.pointer.to_path_string());

        if self.folds.contains(&path) {
            self.folds.remove(&path);
        } else {
            self.folds.insert(path);
        }

        true
    }

    pub fn unfold_all(&mut self) {
        self.folds.clear();
    }

    pub fn go_in(&mut self) -> bool {
        if self.pointer.forward() {
            self.folds.remove(&self.pointer.to_string());
            true
        } else if let Some(c) = self.first_child() {
            self.folds.remove(&self.pointer.push(&c).to_string());
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
            self.folds.remove(&self.pointer.move_to(&s).to_string());
            true
        } else {
            // self.go_out()
            false
        }
    }

    pub fn go_next(&mut self) -> bool {
        if let Some(s) = self.next_sibling() {
            self.folds.remove(&self.pointer.move_to(&s).to_string());
            true
        } else {
            // self.go_out()
            false
        }
    }

    /// Gets the JSON value at the current, or given, pointer location.
    pub fn get_value(&self, pointer_str: Option<&str>) -> Option<&Value> {
        if let Some(str) = pointer_str {
            self.value.pointer(str)
        } else {
            self.value.pointer(&self.pointer.to_path_string())
        }
    }

    /// Get the key of the value at the current pointer location.
    fn get_key(&self) -> Option<String> {
        self.pointer.path.last().map(|s| s.to_string())
    }

    /// Get selection (key and value)
    pub fn get_selection(&self) -> Option<(Option<String>, &Value)> {
        let value = self.get_value(None)?;

        Some(if self.value_is_array_element() {
            (None, value)
        } else {
            self.get_key().map_or((None, value), |k| (Some(k), value))
        })
    }

    fn value_is_array_element(&self) -> bool {
        self.get_parent_value()
            .map(|v| v.is_array())
            .unwrap_or(false)
    }

    /// Gets the parent value of the current pointer location.
    fn get_parent_value(&self) -> Option<&Value> {
        if let Some(parent) = self.pointer.parent_pointer() {
            self.get_value(Some(&parent.to_path_string()))
        } else {
            None
        }
    }

    /// Gets the first child of an object or array
    pub fn first_child(&self) -> Option<String> {
        if let Some(v) = self.get_value(None) {
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

    /// Gets the last child of an object or array
    #[allow(dead_code)]
    pub fn last_child(&self) -> Option<String> {
        if let Some(v) = self.get_value(None) {
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

    /// Gets the previous sibling element for a given pointer index.
    /// If `idx` is `None`, the last element in the pointer is used.
    pub fn prev_sibling(&self) -> Option<String> {
        if let Some(v) = self.get_parent_value() {
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
        if let Some(v) = self.get_parent_value() {
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
        Json::new(get_json_value())
    }

    #[test]
    fn test_pointer_str() {
        let state = get_state();
        assert_eq!(state.pointer.to_path_string().as_str(), "");
    }

    #[test]
    fn test_pointer_value() {
        let state = get_state();
        assert_eq!(state.get_value(None), Some(&get_json_value()));
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
        let mut state = Json::new(json!("foo"));
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
