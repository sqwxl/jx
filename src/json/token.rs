use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Key(String),
    Index(usize),
}

impl Token {
    pub fn as_key(&self) -> Option<String> {
        match self {
            Token::Key(k) => Some(k.clone()),
            _ => None,
        }
    }

    pub fn as_index(&self) -> Option<usize> {
        match self {
            Token::Index(i) => Some(*i),
            _ => None,
        }
    }
}

impl From<String> for Token {
    fn from(s: String) -> Self {
        Token::Key(s)
    }
}

impl From<usize> for Token {
    fn from(i: usize) -> Self {
        Token::Index(i)
    }
}

impl PartialEq<&str> for Token {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Token::Key(k) => k == *other,
            Token::Index(i) => i.to_string().as_str() == *other,
        }
    }
}

impl PartialEq<String> for Token {
    fn eq(&self, other: &String) -> bool {
        match self {
            Token::Key(k) => k == other,
            _ => false,
        }
    }
}

impl PartialEq<usize> for Token {
    fn eq(&self, other: &usize) -> bool {
        match self {
            Token::Index(i) => i == other,
            _ => false,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Key(k) => write!(f, "{}", k),
            Token::Index(i) => write!(f, "{}", i),
        }
    }
}
