use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
use Direction::*;

pub enum Action {
    Quit,
    Move(Direction),
    Scroll(Direction),
    Resize(usize, usize),
    Fold,
    Ignore,
}
use Action::*;

pub fn user_event() -> std::io::Result<Action> {
    let event = event::read()?;
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            kind: _,
            state: _,
        }) => Ok(Quit),
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: _,
            state: _,
        }) => Ok(Quit),
        Event::Key(KeyEvent {
            code: KeyCode::Char('k') | KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: _,
            state: _,
        }) => Ok(Move(Up)),
        Event::Key(KeyEvent {
            code: KeyCode::Char('j') | KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: _,
            state: _,
        }) => Ok(Move(Down)),
        Event::Key(KeyEvent {
            code: KeyCode::Char('h') | KeyCode::Left,
            modifiers: KeyModifiers::NONE,
            kind: _,
            state: _,
        }) => Ok(Move(Left)),
        Event::Key(KeyEvent {
            code: KeyCode::Char('l') | KeyCode::Right,
            modifiers: KeyModifiers::NONE,
            kind: _,
            state: _,
        }) => Ok(Move(Right)),
        Event::Key(KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            kind: _,
            state: _,
        }) => Ok(Fold),
        Event::Key(KeyEvent {
            code: KeyCode::Char('u') | KeyCode::PageUp,
            modifiers: KeyModifiers::NONE | KeyModifiers::CONTROL,
            kind: _,
            state: _,
        }) => Ok(Scroll(Up)),
        Event::Key(KeyEvent {
            code: KeyCode::Char('d') | KeyCode::PageDown,
            modifiers: KeyModifiers::NONE | KeyModifiers::CONTROL,
            kind: _,
            state: _,
        }) => Ok(Scroll(Down)),
        Event::Resize(w, h) => Ok(Resize(w as usize, h as usize)),
        _ => Ok(Ignore),
    }
}
