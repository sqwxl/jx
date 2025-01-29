use std::{collections::BTreeSet, fmt::Display};

#[derive(Clone, Default, PartialEq, Eq, Hash)]
/// A path to a specific location in a JSON document.
///
/// * `pointer`: A list of Strings representing the path.
/// * `cursor`: An index representing the current selected path element.
pub struct Pointer {
    items: Vec<String>,
    cursor: usize,
}

impl Pointer {
    fn new() -> Self {
        Self {
            items: vec![],
            cursor: 0,
        }
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    /// Advances the pointer cursor, if possible.
    fn forward(&mut self) -> bool {
        if self.cursor + 1 < self.len() {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    /// Rewinds the pointer cursor
    fn rewind(&mut self) -> bool {
        if self.cursor > 0 {
            self.cursor -= 1;
            true
        } else {
            false
        }
    }

    /// Returns the element at the cursor.
    fn current(&self) -> Option<&String> {
        self.items.get(self.cursor)
    }

    /// Returns a slice of the pointer elements up to, and including, the 'cursor'.
    pub fn current_slice(&self) -> &[String] {
        if self.items.is_empty() {
            &[]
        } else {
            &self.items[..=self.cursor]
        }
    }

    /// Changes the element at the cursor.
    /// Also drops the elements past the cursor.
    fn reset_cursor(&mut self, s: &str) -> &mut Self {
        self.items.truncate(self.cursor + 1);

        self.items[self.cursor] = s.to_owned();

        self
    }

    fn push(&mut self, s: &str) -> &mut Self {
        self.items.push(s.to_owned());

        self
    }

    /// JSON Pointer defines a string syntax for identifying a specific value
    /// within a JavaScript Object Notation (JSON) document.
    /// https://datatracker.ietf.org/doc/html/rfc6901
    fn to_json_pointer(&self) -> String {
        self.current_slice()
            .iter()
            .fold(String::new(), |mut acc, s| {
                acc += format!("/{}", Self::escape_reference_token(s)).as_str();

                acc
            })
    }

    fn parent_pointer(&self) -> String {
        self.current_slice()
            .iter()
            .take(self.cursor)
            .fold(String::new(), |mut acc, s| {
                acc += format!("/{}", Self::escape_reference_token(s)).as_str();

                acc
            })
    }

    fn escape_reference_token(s: &str) -> String {
        s.replace("/", "~1").replace("~", "~0")
    }
}

impl Display for Pointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.cursor == 0 {
            write!(f, ".")?;
            return Ok(());
        }

        self.items
            .iter()
            .take(self.cursor + 1)
            .for_each(|p| write!(f, ".{}", p).unwrap());

        Ok(())
    }
}

/// A parsed JSON object with a pointer to the current active node.
#[derive(PartialEq, Eq, Hash)]
pub struct Json {
    pub value: serde_json::Value,
    pub pointer: Pointer,
    folds: BTreeSet<String>,
}

impl Json {
    pub fn new(value: serde_json::Value) -> Self {
        Self {
            value,
            pointer: Pointer::new(),
            folds: BTreeSet::new(),
        }
    }

    pub fn toggle_fold(&mut self, json_pointer: Option<String>) {
        let pointer = json_pointer.unwrap_or_else(|| self.pointer.to_json_pointer());

        if self.folds.contains(&pointer) {
            self.folds.remove(&pointer);
        } else {
            self.folds.insert(pointer);
        }
    }

    fn fold(&mut self, pointer: String) {
        self.folds.insert(pointer);
    }

    fn unfold(&mut self, pointer: String) {
        self.folds.remove(&pointer);
    }

    pub fn unfold_all(&mut self) {
        self.folds.clear();
    }

    pub fn go_in(&mut self) -> bool {
        if self.pointer.forward() {
            self.unfold(self.pointer.to_json_pointer());
            true
        } else if let Some(c) = self.first_child() {
            self.pointer.push(&c).forward();
            self.unfold(self.pointer.to_json_pointer());
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
            self.pointer.reset_cursor(&s);
            self.unfold(self.pointer.to_json_pointer());
            true
        } else {
            // self.go_out()
            false
        }
    }

    pub fn go_next(&mut self) -> bool {
        if let Some(s) = self.next_sibling() {
            self.pointer.reset_cursor(&s);
            self.unfold(self.pointer.to_json_pointer());
            true
        } else {
            // self.go_out()
            false
        }
    }

    /// Gets the JSON value at the current, or given, pointer location.
    pub fn get_value(&self, pointer_str: Option<String>) -> Option<&serde_json::Value> {
        if let Some(str) = pointer_str {
            self.value.pointer(&str)
        } else {
            self.value.pointer(&self.pointer.to_json_pointer())
        }
    }

    /// Gets the parent value of the current pointer location.
    fn get_parent_value(&self) -> Option<&serde_json::Value> {
        self.get_value(Some(self.pointer.parent_pointer()))
    }

    /// Get the key of the value at the current pointer location.
    fn get_key(&self) -> Option<String> {
        self.pointer.current().map(|s| s.to_string())
    }

    /// Get selection (key and value)
    pub fn get_selection(&self) -> Option<(Option<String>, &serde_json::Value)> {
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

    /// Gets the first child of an object or array
    pub fn first_child(&self) -> Option<String> {
        if let Some(v) = self.get_value(None) {
            match v {
                serde_json::Value::Object(o) => {
                    if let Some(key) = o.keys().next() {
                        return Some(key.to_owned());
                    }
                }
                serde_json::Value::Array(a) => {
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
                serde_json::Value::Object(o) => {
                    if let Some(key) = o.keys().last() {
                        return Some(key.to_owned());
                    }
                }
                serde_json::Value::Array(a) => {
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
    pub fn prev_sibling(&self) -> Option<String> {
        if let Some(v) = self.get_parent_value() {
            match v {
                serde_json::Value::Object(o) => {
                    let key_idx = o
                        .keys()
                        .position(|k| Some(k) == self.pointer.current())
                        .expect("key matching current pointer cursor should be present");
                    if key_idx > 0 {
                        return Some(o.keys().nth(key_idx - 1).unwrap().to_string());
                    }
                }
                serde_json::Value::Array(_) => {
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
    pub fn next_sibling(&self) -> Option<String> {
        if let Some(v) = self.get_parent_value() {
            match v {
                serde_json::Value::Object(o) => {
                    let key_idx = o
                        .keys()
                        .position(|k| Some(k) == self.pointer.current())
                        .unwrap();
                    if key_idx < (o.keys().len() - 1) {
                        return Some(o.keys().nth(key_idx + 1).unwrap().to_string());
                    }
                }
                serde_json::Value::Array(a) => {
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

    fn get_json_value() -> serde_json::Value {
        serde_json::from_str(J).unwrap()
    }

    fn get_state() -> Json {
        Json::new(get_json_value())
    }

    #[test]
    fn test_pointer_str() {
        let state = get_state();
        assert_eq!(state.pointer.to_json_pointer().as_str(), "");
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
        assert_eq!(state.pointer.items, vec!["a"]);
        state.go_out();
        assert_eq!(state.pointer.items, vec!["a"]);
    }

    #[test]
    fn test_go_down_up() {
        let mut state = get_state();
        state.go_in();
        state.go_next();
        assert_eq!(state.pointer.items, vec!["b"]);
        state.go_prev();
        assert_eq!(state.pointer.items, vec!["a"]);
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
        assert_eq!(state.pointer.items, vec!["e", "0"]);
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
        assert_eq!(state.pointer.items, vec!["e", "1", "bar", "1", "qux"]);
    }
}
