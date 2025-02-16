use std::fmt::Display;

use super::Token;

#[derive(Clone)]
/// A path to a specific location in a JSON document.
///
/// * `tokens`: A list of Strings representing a path to a specific location in a JSON document.
/// * `cursor`: An index representing the current selected path element.
pub struct Pointer {
    tokens: Vec<Token>,
    cursor: Option<usize>,
}

impl Pointer {
    pub fn new() -> Self {
        Self {
            tokens: vec![],
            cursor: None,
        }
    }

    fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn is_at_start(&self) -> bool {
        self.cursor.is_none()
    }

    pub fn is_at_end(&self) -> bool {
        match self.cursor {
            None => self.tokens.is_empty(),
            Some(c) => c + 1 >= self.len(),
        }
    }

    /// Advances the pointer cursor, if possible.
    pub fn forward(&mut self) -> &mut Self {
        match self.cursor {
            None if !self.tokens.is_empty() => self.cursor = Some(0),
            Some(c) if c + 1 < self.len() => self.cursor = Some(c + 1),
            _ => {}
        }

        self
    }

    /// Rewinds the pointer cursor
    pub fn rewind(&mut self) -> &mut Self {
        match self.cursor {
            None => {}
            Some(0) => self.cursor = None,
            Some(c) => self.cursor = Some(c - 1),
        }

        self
    }

    /// Returns the element at the cursor.
    pub fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.cursor?)
    }

    /// Returns a collection of tokens up to, and including, the 'cursor'.
    pub fn tokens(&self) -> Vec<Token> {
        match self.cursor {
            None => vec![],
            Some(c) => self.tokens[..=c].to_owned(),
        }
    }

    /// Returns a collection of tokens up to, and excluding, the 'cursor'.
    pub fn parent_tokens(&self) -> Vec<Token> {
        match self.cursor {
            None => vec![],
            Some(c) => self.tokens[..c].to_owned(),
        }
    }

    /// Changes the element at the cursor.
    /// Also drops the elements past the cursor.
    pub fn reset_cursor(&mut self, token: &Token) -> &mut Self {
        match self.cursor {
            None => panic!("shouldn't change the first token"),
            Some(c) => {
                self.tokens.truncate(c + 1);

                self.tokens[c] = token.to_owned();
            }
        }

        self
    }

    pub fn push(&mut self, s: &Token) -> &mut Self {
        self.tokens.push(s.to_owned());

        self
    }

    pub fn to_json_pointer(&self) -> String {
        Self::json_pointer(&self.tokens())
    }

    /// JSON Pointer defines a string syntax for identifying a specific value
    /// within a JavaScript Object Notation (JSON) document.
    /// https://datatracker.ietf.org/doc/html/rfc6901
    pub fn json_pointer(tokens: &[Token]) -> String {
        tokens.iter().fold(String::new(), |mut acc, token| {
            acc += format!("/{}", Self::escape_token(token)).as_str();

            acc
        })
    }

    fn escape_token(s: &Token) -> String {
        match s {
            Token::Key(k) => k.replace("~", "~0").replace("/", "~1"),
            Token::Index(i) => format!("{}", i),
        }
    }
}

impl Display for Pointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.cursor {
            None => {
                write!(f, ".")?;
                return Ok(());
            }

            Some(c) => self
                .tokens
                .iter()
                .take(c + 1)
                .for_each(|p| write!(f, ".{}", p).unwrap()),
        }

        Ok(())
    }
}
