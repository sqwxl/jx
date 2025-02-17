use std::fmt::Display;

use crossterm::style::Attribute;
use crossterm::style::{Attributes, Color, ContentStyle, StyledContent};

use crate::json::Token;

#[derive(Default, Clone)]
pub struct StyledString(pub String, pub StyleClass);

impl Display for StyledString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.1.apply(self.0.as_str()))
    }
}

#[derive(Default, Clone)]
pub struct StyledLine {
    pub line_number: usize,
    pub indent: usize,
    pub elements: Vec<StyledString>,
    pub pointer: Vec<Token>,
}

impl StyledLine {
    pub fn push(&mut self, element: StyledString) {
        self.elements.push(element);
    }

    pub fn extend(&mut self, elements: Vec<StyledString>) {
        self.elements.extend(elements);
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum StyleClass {
    #[default]
    Whitespace,
    Punct,
    String,
    Number,
    Bool,
    Null,
    FoldCount,
}

impl StyleClass {
    pub fn apply<D: Display>(&self, text: D) -> StyledContent<D> {
        match self {
            StyleClass::Whitespace => STYLE_WHITESPACE.apply(text),
            StyleClass::Punct => STYLE_PUNCT.apply(text),
            StyleClass::String => STYLE_STRING.apply(text),
            StyleClass::Number => STYLE_NUMBER.apply(text),
            StyleClass::Bool => STYLE_BOOL.apply(text),
            StyleClass::Null => STYLE_NULL.apply(text),
            StyleClass::FoldCount => STYLE_FOLD_COUNT.apply(text),
        }
    }
}

pub const INDENT: usize = 4;

const STYLE_WHITESPACE: ContentStyle = ContentStyle {
    foreground_color: Some(Color::White),
    background_color: None,
    attributes: Attributes::none(),
    underline_color: None,
};

const STYLE_KEY: ContentStyle = ContentStyle {
    foreground_color: Some(Color::DarkBlue),
    background_color: None,
    attributes: Attributes::none(),
    underline_color: None,
};

const STYLE_STRING: ContentStyle = ContentStyle {
    foreground_color: Some(Color::DarkGreen),
    background_color: None,
    attributes: Attributes::none(),
    underline_color: None,
};

const STYLE_NUMBER: ContentStyle = ContentStyle {
    foreground_color: Some(Color::DarkRed),
    background_color: None,
    attributes: Attributes::none(),
    underline_color: None,
};

const STYLE_BOOL: ContentStyle = ContentStyle {
    foreground_color: Some(Color::DarkBlue),
    background_color: None,
    attributes: Attributes::none(),
    underline_color: None,
};

const STYLE_NULL: ContentStyle = ContentStyle {
    foreground_color: Some(Color::DarkMagenta),
    background_color: None,
    attributes: Attributes::none(),
    underline_color: None,
};

const STYLE_PUNCT: ContentStyle = ContentStyle {
    foreground_color: Some(Color::DarkYellow),
    background_color: None,
    attributes: Attributes::none(),
    underline_color: None,
};

const STYLE_DEFAULT: ContentStyle = ContentStyle {
    foreground_color: Some(Color::White),
    background_color: None,
    attributes: Attributes::none(),
    underline_color: None,
};

const STYLE_FOLD_COUNT: ContentStyle = ContentStyle {
    foreground_color: Some(Color::Grey),
    background_color: None,
    attributes: Attributes::none(),
    underline_color: None,
};
pub const STYLE_SELECTION: ContentStyle = ContentStyle {
    foreground_color: None,
    background_color: None,
    attributes: Attributes::with(Attributes::none(), Attribute::Underlined),
    underline_color: Some(Color::White),
};

pub const STYLE_TITLE: ContentStyle = ContentStyle {
    foreground_color: Some(Color::White),
    background_color: None,
    attributes: Attributes::none(),
    underline_color: None,
};

pub const STYLE_POINTER: ContentStyle = STYLE_TITLE;
