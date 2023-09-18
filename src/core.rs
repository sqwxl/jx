use serde_json::Value;

#[derive(Debug, Default)]
pub struct View {
    pub size: (u16, u16),
    pub scroll: (u16, u16),
}
impl View {
    pub fn new(size: (u16, u16)) -> Self {
        Self {
            size,
            scroll: (0, 0),
        }
    }

    pub fn resize(&mut self, size: (u16, u16)) {
        self.size = size;
    }
}

#[derive(Debug, Default)]
/// Core struct providing application state
///
/// * `value`: the JSON value
/// * `pointer`: the path to the current active node in the value object
pub struct Core {
    pub value: Value,
    pub pointer: Vec<String>,
    pub view: View,
}
impl Core {
    pub fn new(value: Value, size: (u16, u16)) -> Self {
        Self {
            value,
            pointer: vec![], // start at root
            view: View::new(size),
        }
    }

    /// Moves the cursor to the 1st element of an object or array.
    /// Does nothing if the current cursor position is a primitive.
    pub fn go_in(&mut self) -> bool {
        if let Some(first_child) = self.first_child(None) {
            self.pointer.push(first_child);
            true
        } else {
            false
        }
    }

    /// Moves the cursor out of the current element, unless the pointer is already at the root
    pub fn go_out(&mut self) -> bool {
        if !self.pointer.is_empty() {
            self.pointer.pop();
            true
        } else {
            false
        }
    }

    /// Moves the pointer to the previous sibling element of the current pointer position
    pub fn go_prev(&mut self) -> bool {
        if let Some(s) = self.prev_sibling(None) {
            let last = self.pointer.len() - 1;
            self.pointer[last] = s;
            true
        } else {
            false
        }
    }

    /// Moves the pointer to the next sibling element of the current pointer position
    pub fn go_next(&mut self) -> bool {
        if let Some(s) = self.next_sibling(None) {
            let last = self.pointer.len() - 1;
            self.pointer[last] = s;
            true
        } else {
            false
        }
    }

    fn pointer_str(&self, end: Option<usize>) -> String {
        let mut s = String::new();
        let end = end.unwrap_or(self.pointer.len());
        for p in self.pointer.iter().take(end) {
            s.push('/');
            s.push_str(p);
        }
        println!("pointer_str: {:?}", s);

        s
    }

    pub fn depth(&self) -> usize {
        self.pointer.len()
    }

    pub fn pointer_value(&self, idx: Option<usize>) -> Option<&Value> {
        let v = self.value.pointer(&self.pointer_str(idx));
        println!("pointer_value: {:?}", v);
        v
    }

    pub fn pointer_last(&self) -> &str {
        self.pointer.last().unwrap()
    }

    pub fn pointer_parent_value(&self, idx: Option<usize>) -> Option<&Value> {
        if self.pointer.is_empty() {
            None
        } else {
            let idx = idx.unwrap_or(self.pointer.len());
            self.pointer_value(Some(idx - 1))
        }
    }

    /// Gets the first child of an object or array (the key or index, not the value itself)
    pub fn first_child(&self, idx: Option<usize>) -> Option<String> {
        if let Some(v) = self.pointer_value(idx) {
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

    /// Gets the last child of an object or array (the key or index, not the value itself)
    pub fn last_child(&self, idx: Option<usize>) -> Option<String> {
        if let Some(v) = self.pointer_value(idx) {
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
    pub fn prev_sibling(&self, idx: Option<usize>) -> Option<String> {
        if let Some(v) = self.pointer_parent_value(idx) {
            match v {
                Value::Object(o) => {
                    let key_idx = o.keys().position(|k| k == self.pointer_last()).unwrap();
                    if key_idx > 0 {
                        return Some(o.keys().nth(key_idx - 1).unwrap().to_string());
                    }
                }
                Value::Array(_) => {
                    let current_idx = self.pointer_last().parse::<usize>().unwrap();
                    if current_idx > 0 {
                        return Some((current_idx - 1).to_string());
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Gets the next sibling element for a given pointer index.
    /// If `idx` is `None`, the last element in the pointer is used.
    pub fn next_sibling(&self, idx: Option<usize>) -> Option<String> {
        if let Some(v) = self.pointer_parent_value(idx) {
            match v {
                Value::Object(o) => {
                    let key_idx = o.keys().position(|k| k == self.pointer_last()).unwrap();
                    if key_idx < (o.keys().len() - 1) {
                        return Some(o.keys().nth(key_idx + 1).unwrap().to_string());
                    }
                }
                Value::Array(a) => {
                    let current_idx = self.pointer_last().parse::<usize>().unwrap();
                    if current_idx < a.len() - 1 {
                        return Some((current_idx + 1).to_string());
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

    fn get_state() -> Core {
        Core::new(get_json_value(), (80, 24))
    }

    #[test]
    fn test_pointer_str() {
        let state = get_state();
        assert_eq!(state.pointer_str(None), "");
    }

    #[test]
    fn test_pointer_value() {
        let state = get_state();
        assert_eq!(state.pointer_value(None), Some(&get_json_value()));
    }

    #[test]
    fn test_first_child() {
        let state = get_state();
        assert_eq!(state.first_child(None), Some("a".to_string()));
    }

    #[test]
    fn test_last_child() {
        let state = get_state();
        assert_eq!(state.last_child(None), Some("e".to_string()));
    }

    #[test]
    fn test_move_on_primitive() {
        let mut state = Core::new(json!("foo"), (80, 24));
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
        assert_eq!(state.pointer, vec!["a"]);
        state.go_out();
        assert_eq!(state.pointer.len(), 0);
    }

    #[test]
    fn test_go_down_up() {
        let mut state = get_state();
        state.go_in();
        state.go_next();
        assert_eq!(state.pointer, vec!["b"]);
        state.go_prev();
        assert_eq!(state.pointer, vec!["a"]);
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
        assert_eq!(state.pointer, vec!["e", "0"]);
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
        assert_eq!(state.pointer, vec!["e", "1", "bar", "1", "qux"]);
    }
}
