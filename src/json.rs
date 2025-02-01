use std::{collections::HashSet, fmt::Display};

/// A parsed JSON object with a pointer to the current active node.
pub struct Json {
    pub value: serde_json::Value,
    pub pointer: Pointer,
    pub folds: HashSet<Vec<String>>,
}

impl Json {
    pub fn from(value: serde_json::Value) -> Self {
        Self {
            value,
            pointer: Pointer::new(),
            folds: HashSet::new(),
        }
    }

    pub fn tokens(&self) -> Vec<String> {
        self.pointer.tokens()
    }

    pub fn path(&self) -> String {
        self.pointer.to_string()
    }

    pub fn toggle_fold(&mut self) {
        let tokens = self.pointer.tokens();

        if self.folds.contains(&tokens) {
            self.unfold(&tokens);
        } else {
            self.fold(tokens);
        }
    }

    fn fold(&mut self, tokens: Vec<String>) {
        self.folds.insert(tokens);
    }

    fn unfold(&mut self, tokens: &[String]) {
        self.folds.remove(tokens);
    }

    pub fn unfold_all(&mut self) {
        self.folds.clear();
    }

    pub fn go_in(&mut self) -> bool {
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
        self.pointer.current_token().map(|s| s.to_string())
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
    fn first_child(&self) -> Option<String> {
        if let Some(v) = self.get_current_value() {
            match v {
                serde_json::Value::Object(o) => {
                    return o.keys().next().map(|key| key.to_owned());
                }
                serde_json::Value::Array(a) => {
                    if !a.is_empty() {
                        return Some("0".to_string());
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
        if let Some(v) = self.get_current_value() {
            match v {
                serde_json::Value::Object(o) => {
                    return o.keys().last().map(|key| key.to_owned());
                }
                serde_json::Value::Array(a) => {
                    if !a.is_empty() {
                        return Some((a.len() - 1).to_string());
                    }
                }
                _ => {}
            }
        }

        None
    }

    /// Gets the previous sibling element for a given pointer index.
    pub fn prev_sibling(&self) -> Option<String> {
        if self.pointer.is_at_start() {
            return None;
        }

        if let Some(v) = self.get_parent_value() {
            match v {
                serde_json::Value::Object(o) => {
                    let key_idx = o
                        .keys()
                        .position(|k| Some(k) == self.pointer.current_token())
                        .expect("key matching current pointer cursor should be present");

                    if key_idx > 0 {
                        return o.keys().nth(key_idx - 1).map(|k| k.to_string());
                    }
                }
                serde_json::Value::Array(_) => {
                    let idx = self
                        .pointer
                        .current_token()
                        .map(|s| s.parse::<usize>().expect("array index should be a number"))?;

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
        if self.pointer.is_at_start() {
            return None;
        }

        if let Some(v) = self.get_parent_value() {
            match v {
                serde_json::Value::Object(o) => {
                    let key_idx = o
                        .keys()
                        .position(|k| Some(k) == self.pointer.current_token())
                        .expect("key matching current pointer cursor should be present");

                    if key_idx < (o.keys().len() - 1) {
                        return o.keys().nth(key_idx + 1).map(|k| k.to_string());
                    }
                }
                serde_json::Value::Array(a) => {
                    let idx = self
                        .pointer
                        .current_token()
                        .map(|s| s.parse::<usize>().expect("array index should be a number"))?;

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

#[derive(Clone)]
/// A path to a specific location in a JSON document.
///
/// * `tokens`: A list of Strings representing a path to a specific location in a JSON document.
/// * `cursor`: An index representing the current selected path element.
pub struct Pointer {
    tokens: Vec<String>,
    cursor: usize,
}

impl Pointer {
    fn new() -> Self {
        Self {
            tokens: vec!["".to_string()],
            cursor: 0,
        }
    }

    fn len(&self) -> usize {
        self.tokens.len()
    }

    fn is_at_start(&self) -> bool {
        self.cursor == 0
    }

    fn is_at_end(&self) -> bool {
        self.cursor + 1 >= self.len()
    }

    /// Advances the pointer cursor, if possible.
    fn forward(&mut self) -> &mut Self {
        if !self.is_at_end() {
            self.cursor += 1;
        }

        self
    }

    /// Rewinds the pointer cursor
    fn rewind(&mut self) -> &mut Self {
        if !self.is_at_start() {
            self.cursor -= 1;
        }

        self
    }

    /// Returns the element at the cursor.
    fn current_token(&self) -> Option<&String> {
        if self.is_at_start() {
            None
        } else {
            self.tokens.get(self.cursor)
        }
    }

    /// Returns a collection of tokens up to, and including, the 'cursor'.
    pub fn tokens(&self) -> Vec<String> {
        if self.is_at_start() {
            vec![]
        } else {
            self.tokens[1..=self.cursor].to_owned()
        }
    }

    /// Returns a collection of tokens up to, and excluding, the 'cursor'.
    fn parent_tokens(&self) -> Vec<String> {
        if self.is_at_start() {
            vec![]
        } else {
            self.tokens[1..self.cursor].to_owned()
        }
    }

    /// Changes the element at the cursor.
    /// Also drops the elements past the cursor.
    fn reset_cursor(&mut self, s: &str) -> &mut Self {
        if self.is_at_start() {
            panic!("Cannot change the first token");
        }

        self.tokens.truncate(self.cursor + 1);

        self.tokens[self.cursor] = s.to_owned();

        self
    }

    fn push(&mut self, s: &str) -> &mut Self {
        self.tokens.push(s.to_owned());

        self
    }

    fn to_json_pointer(&self) -> String {
        Self::json_pointer(&self.tokens())
    }

    /// JSON Pointer defines a string syntax for identifying a specific value
    /// within a JavaScript Object Notation (JSON) document.
    /// https://datatracker.ietf.org/doc/html/rfc6901
    pub fn json_pointer(tokens: &[String]) -> String {
        tokens.iter().fold(String::new(), |mut acc, token| {
            acc += format!("/{}", Self::escape_token(token)).as_str();

            acc
        })
    }

    fn escape_token(s: &str) -> String {
        s.replace("/", "~1").replace("~", "~0")
    }
}

impl Display for Pointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.cursor == 0 {
            write!(f, ".")?;
            return Ok(());
        }

        self.tokens
            .iter()
            .skip(1)
            .take(self.cursor + 1)
            .for_each(|p| write!(f, ".{}", p).unwrap());

        Ok(())
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
        Json::from(get_json_value())
    }

    #[should_panic]
    fn change_first_token() {
        let mut state = get_state();
        state.pointer.reset_cursor("foo");
    }

    #[test]
    fn to_json_pointer() {
        let state = get_state();
        assert_eq!(state.pointer.to_json_pointer().as_str(), "");
    }

    #[test]
    fn get_current_value() {
        let state = get_state();
        assert_eq!(state.get_current_value(), Some(&get_json_value()));
    }

    #[test]
    fn get_first_child() {
        let state = get_state();
        assert_eq!(state.first_child(), Some("a".to_string()));
    }

    #[test]
    fn get_last_child() {
        let state = get_state();
        assert_eq!(state.last_child(), Some("e".to_string()));
    }

    #[test]
    fn move_on_primitive() {
        let mut state = Json::from(json!("foo"));
        state.go_in();
        assert_eq!(state.pointer.len(), 1);
        state.go_out();
        assert_eq!(state.pointer.len(), 1);
        state.go_prev();
        assert_eq!(state.pointer.len(), 1);
        state.go_next();
        assert_eq!(state.pointer.len(), 1);
    }

    #[test]
    fn go_in() {
        let mut state = get_state();
        state.go_in();
        assert_eq!(state.pointer.tokens, vec!["", "a"]);
    }

    #[test]
    fn go_in_twice() {
        let mut json = Json::from(json!({"a":{"b": 0}}));
        json.go_in();
        json.go_in();
        assert_eq!(json.pointer.tokens, vec!["", "a", "b"]);
    }

    #[test]
    fn go_out() {
        let mut state = get_state();
        state.go_in();
        assert_eq!(state.pointer.tokens, vec!["", "a"]);
        state.go_out();
        assert_eq!(state.pointer.tokens, vec!["", "a"]);
    }

    #[test]
    fn go_down_up() {
        let mut state = get_state();
        state.go_in();
        state.go_next();
        assert_eq!(state.pointer.tokens, vec!["", "b"]);
        state.go_prev();
        assert_eq!(state.pointer.tokens, vec!["", "a"]);
    }

    #[test]
    fn go_into_array() {
        let mut state = get_state();

        state.go_in(); // "a"
        state.go_next(); // "b"
        state.go_next(); // "c"
        state.go_next(); // "d"
        state.go_next(); // "e" (array)
        state.go_in(); // "0"
        assert_eq!(state.pointer.tokens, vec!["", "e", "0"]);
    }

    #[test]
    fn go_deep() {
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
        assert_eq!(state.pointer.tokens, vec!["", "e", "1", "bar", "1", "qux"]);
    }
}
