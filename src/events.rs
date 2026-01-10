use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

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
    ScrollLeftMax,
    ScrollRightMax,
    ToggleFold,
    ToggleFoldAll,
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
    OutputSelectionRaw,
    OutputValueRaw,
    CopySelectionPretty,
    CopyValuePretty,
    CopySelectionRaw,
    CopyValueRaw,
    ToggleLineNumbers,
    ToggleLineWrapping,
    Ignore,
}

use Action::*;
use KeyCode::*;

pub fn read_event(
    search_mode: bool,
    help_mode: bool,
    timeout: Option<Duration>,
) -> Result<Option<Action>> {
    if let Some(duration) = timeout {
        if !event::poll(duration)? {
            return Ok(None);
        }
    }

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
                    (Esc, _) | (Char('c'), KeyModifiers::CONTROL) => SearchCancel,
                    (Enter, _) => SearchConfirm,
                    (Backspace, _) => SearchBackspace,
                    (Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => SearchInput(c),
                    _ => Ignore,
                }
            } else {
                match (code, modifiers) {
                    (Char('q'), _) | (Char('c'), KeyModifiers::CONTROL) => Quit,
                    (Char('?'), _) => ShowHelp,

                    (Char('h') | Left, _) => Move(Direction::Left),
                    (Char('j') | Down, _) => Move(Direction::Down),
                    (Char('k') | Up, _) => Move(Direction::Up),
                    (Char('l') | Right, _) => Move(Direction::Right),

                    (Char('y'), KeyModifiers::CONTROL) => ScrollLine(Direction::Up),
                    (Char('e'), KeyModifiers::CONTROL) => ScrollLine(Direction::Down),
                    (Char('u'), _) => ScrollHalf(Direction::Up),
                    (Char('d'), _) => ScrollHalf(Direction::Down),
                    (Char('b'), _) => ScrollFull(Direction::Up),
                    (Char('f'), _) => ScrollFull(Direction::Down),
                    (Char('g'), _) => ScrollTop,
                    (Char('G'), _) => ScrollBottom,
                    (Char('<'), _) => ScrollLeft,
                    (Char('>'), _) => ScrollRight,
                    (Char('^') | Char('H'), _) => ScrollLeftMax,
                    (Char('$') | Char('L'), _) => ScrollRightMax,

                    (Char(' ') | Enter, _) => ToggleFold,
                    (Char('z'), _) => ToggleFoldAll,

                    (Char('/'), _) => Search,
                    (Char('n'), _) => RepeatSearch,
                    (Char('N'), _) => RepeatSearchBackward,
                    (Esc, _) => ClearSearch,

                    (Char('s'), _) => Sort,
                    (Char('S'), _) => SortReverse,
                    (Char('&'), _) => Filter,

                    (Char('y'), KeyModifiers::NONE) => CopySelectionPretty,
                    (Char('Y'), KeyModifiers::NONE) => CopyValuePretty,
                    (Char('y'), KeyModifiers::ALT) => CopySelectionRaw,
                    (Char('Y'), KeyModifiers::ALT) => CopyValueRaw,

                    (Char('o'), KeyModifiers::NONE) => OutputSelectionPretty,
                    (Char('O'), KeyModifiers::NONE) => OutputValuePretty,
                    (Char('o'), KeyModifiers::ALT) => OutputSelectionRaw,
                    (Char('O'), KeyModifiers::ALT) => OutputValueRaw,

                    (Char('#'), _) => ToggleLineNumbers,
                    (Char('w'), _) => ToggleLineWrapping,

                    _ => Ignore,
                }
            }
        }

        _ => Ignore,
    };

    Ok(Some(action))
}
