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
    ScrollLine(Direction),
    ScrollHalf(Direction),
    ScrollFull(Direction),
    ScrollTop,
    ScrollBottom,
    ScrollLeft,
    ScrollRight,
    Fold,
    Sort,
    SortReverse,
    Search,
    ShowHelp,
    DismissHelp,
    RepeatSearch,
    RepeatSearchBackward,
    Filter,
    ClearSearch,
    SearchInput(char),
    SearchBackspace,
    SearchConfirm,
    SearchCancel,
    OutputSelectionPretty,
    OutputValuePretty,
    OutputSelection,
    OutputValue,
    CopySelectionPretty,
    CopyValuePretty,
    CopySelection,
    CopyValue,
    ToggleLineNumbers,
    ToggleLineWrapping,
    Ignore,
}

use Action::*;

pub fn read_event(search_mode: bool, help_mode: bool) -> Result<Action> {
    let action = match event::read()? {
        Event::Resize(w, h) => Resize(w as usize, h as usize),

        Event::Key(KeyEvent {
            kind: KeyEventKind::Press,
            ..
        }) if help_mode => DismissHelp,

        Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) => {
            if search_mode {
                match (code, modifiers) {
                    (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => SearchCancel,
                    (KeyCode::Enter, _) => SearchConfirm,
                    (KeyCode::Backspace, _) => SearchBackspace,
                    (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => SearchInput(c),
                    _ => Ignore,
                }
            } else {
                match (code, modifiers) {
                    (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => Quit,

                    (KeyCode::Char('h') | KeyCode::Left, _) => Move(Left),
                    (KeyCode::Char('j') | KeyCode::Down, _) => Move(Down),
                    (KeyCode::Char('k') | KeyCode::Up, _) => Move(Up),
                    (KeyCode::Char('l') | KeyCode::Right, _) => Move(Right),

                    (KeyCode::Char('y'), _) => ScrollLine(Up),
                    (KeyCode::Char('e'), _) => ScrollLine(Down),
                    (KeyCode::Char('u'), _) => ScrollHalf(Up),
                    (KeyCode::Char('d'), _) => ScrollHalf(Down),
                    (KeyCode::Char('b'), _) => ScrollFull(Up),
                    (KeyCode::Char('f'), _) => ScrollFull(Down),
                    (KeyCode::Char('g'), _) => ScrollTop,
                    (KeyCode::Char('G'), _) => ScrollBottom,
                    (KeyCode::Char('<'), _) => ScrollLeft,
                    (KeyCode::Char('>'), _) => ScrollRight,

                    (KeyCode::Char(' '), _) => Fold,

                    (KeyCode::Char('s'), _) => Sort,
                    (KeyCode::Char('S'), _) => SortReverse,

                    (KeyCode::Char('/'), _) => Search,
                    (KeyCode::Char('?'), _) => ShowHelp,
                    (KeyCode::Char('n'), _) => RepeatSearch,
                    (KeyCode::Char('N'), _) => RepeatSearchBackward,
                    (KeyCode::Char('&'), _) => Filter,
                    (KeyCode::Esc, _) => ClearSearch,

                    (KeyCode::Enter, KeyModifiers::NONE) => OutputSelectionPretty,
                    (KeyCode::Enter, KeyModifiers::SHIFT) => OutputValuePretty,
                    (KeyCode::Char('o'), _) => OutputSelection,
                    (KeyCode::Char('O'), _) => OutputValue,

                    (KeyCode::Char('c'), _) => CopySelectionPretty,
                    (KeyCode::Char('C'), KeyModifiers::CONTROL) => CopySelectionPretty,
                    (KeyCode::Char('C'), _) => CopyValuePretty,
                    (KeyCode::Char('r'), _) => CopySelection,
                    (KeyCode::Char('R'), _) => CopyValue,

                    (KeyCode::Char('#'), _) => ToggleLineNumbers,
                    (KeyCode::Char('w'), _) => ToggleLineWrapping,

                    _ => Ignore,
                }
            }
        }

        _ => Ignore,
    };

    Ok(action)
}
