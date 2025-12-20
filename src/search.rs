use std::collections::HashSet;

use crate::style::{StyleClass, StyledLine};

#[derive(Clone)]
pub struct SearchMatch {
    pub line_number: usize,
    pub element_index: usize,
    pub char_offset: usize, // Offset within element where match starts
    pub char_positions: HashSet<usize>, // Which characters to highlight
}

#[derive(Clone)]
pub struct SearchResults {
    pub query: String,
    pub matches: Vec<SearchMatch>,
    pub current_index: Option<usize>,
}

impl SearchResults {
    pub fn new(query: String, matches: Vec<SearchMatch>) -> Self {
        Self {
            query,
            matches,
            current_index: None,
        }
    }

    pub fn next(&mut self) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }
        self.current_index = Some(match self.current_index {
            Some(i) => (i + 1) % self.matches.len(),
            None => 0,
        });
        self.current()
    }

    pub fn prev(&mut self) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }
        self.current_index = Some(match self.current_index {
            Some(0) => self.matches.len() - 1,
            Some(i) => i - 1,
            None => self.matches.len() - 1,
        });
        self.current()
    }

    pub fn current(&self) -> Option<&SearchMatch> {
        self.current_index.and_then(|i| self.matches.get(i))
    }

    pub fn status_text(&self) -> String {
        if self.matches.is_empty() {
            "0/0".to_string()
        } else {
            match self.current_index {
                Some(i) => format!("{}/{}", i + 1, self.matches.len()),
                None => format!("-/{}", self.matches.len()),
            }
        }
    }

    pub fn get_current(&self, line_number: usize, element_index: usize) -> Option<&SearchMatch> {
        self.current()
            .filter(|m| m.line_number == line_number && m.element_index == element_index)
    }

    pub fn get_match(&self, line_number: usize, element_index: usize) -> Option<&SearchMatch> {
        self.matches
            .iter()
            .find(|m| m.line_number == line_number && m.element_index == element_index)
    }
}

pub fn perform_search(formatted: &[StyledLine], query: &str) -> SearchResults {
    if query.is_empty() {
        return SearchResults::new(query.to_string(), vec![]);
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    for line in formatted {
        for (elem_idx, elem) in line.elements.iter().enumerate() {
            // Skip punctuation and whitespace
            if matches!(elem.1, StyleClass::Punct | StyleClass::Whitespace) {
                continue;
            }

            let text = &elem.0;
            // Check if text has quotes (for offset calculation)
            let has_leading_quote = text.starts_with('"');
            let quote_offset = if has_leading_quote { 1 } else { 0 };
            let search_text = text.trim_matches('"');
            let search_text_lower = search_text.to_lowercase();

            // Find all occurrences of the query in this element
            let mut start = 0;
            while let Some(pos) = search_text_lower[start..].find(&query_lower) {
                let match_start = start + pos;
                let match_end = match_start + query.chars().count();

                // Character positions for highlighting (adjusted for quote)
                let char_positions: HashSet<usize> =
                    (match_start..match_end).map(|i| i + quote_offset).collect();

                matches.push(SearchMatch {
                    line_number: line.line_number,
                    element_index: elem_idx,
                    char_offset: match_start + quote_offset,
                    char_positions,
                });

                // Move past this match to find the next one
                start = match_start + 1;
            }
        }
    }

    SearchResults::new(query.to_string(), matches)
}
