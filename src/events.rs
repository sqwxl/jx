use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

use Direction::*;

pub enum Action {
    Resize(usize, usize),
    Quit,
    Move(Direction),
    Scroll(Direction),
    ScrollPage(Direction),
    Fold,
    OutputSelectionPretty,
    OutputValuePretty,
    OutputSelection,
    OutputValue,
    CopySelectionPretty,
    CopyValuePretty,
    CopySelection,
    CopyValue,
    Ignore,
}

use Action::*;

pub fn read_event() -> Result<Action> {
    Ok(match event::read()? {
        Event::Resize(w, h) => Resize(w as usize, h as usize),

        Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) => match (code, modifiers) {
            // Quit on 'q', 'Escape' or '^C'
            (KeyCode::Char('q') | KeyCode::Esc, _)
            | (KeyCode::Char('c'), KeyModifiers::CONTROL) => Quit,

            // Move on 'hjkl' or '← ↑ ↓ →'
            (KeyCode::Char('h') | KeyCode::Left, _) => Move(Left),
            (KeyCode::Char('j') | KeyCode::Down, _) => Move(Down),
            (KeyCode::Char('k') | KeyCode::Up, _) => Move(Up),
            (KeyCode::Char('l') | KeyCode::Right, _) => Move(Right),

            // Toggle fold on 'Space'
            (KeyCode::Char(' '), _) => Fold,

            // Scroll up/down
            (KeyCode::Char('u'), _) => Scroll(Up),
            (KeyCode::Char('d'), _) => Scroll(Down),
            (KeyCode::Char('b'), _) => ScrollPage(Up),
            (KeyCode::Char('f'), _) => ScrollPage(Down),

            // Output
            (KeyCode::Enter, KeyModifiers::NONE) => OutputSelectionPretty,
            (KeyCode::Enter, KeyModifiers::SHIFT) => OutputValuePretty,
            (KeyCode::Char('o'), _) => OutputSelection,
            (KeyCode::Char('O'), _) => OutputValue,

            // Clipboard
            (KeyCode::Char('y'), _) => CopySelectionPretty,
            (KeyCode::Char('c'), modifiers) => {
                if modifiers == KeyModifiers::CONTROL | KeyModifiers::SHIFT {
                    CopySelectionPretty
                } else {
                    Ignore
                }
            }
            (KeyCode::Char('C'), _) => CopySelectionPretty,
            (KeyCode::Char('Y'), _) => CopyValuePretty,
            (KeyCode::Char('r'), _) => CopySelection,
            (KeyCode::Char('R'), _) => CopyValue,

            _ => Ignore,
        },

        _ => Ignore,
    })
}
