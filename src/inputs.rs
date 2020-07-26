use std::{
    io,
    io::Stdout,
};
use termion::{
    AsyncReader,
    event::Key,
    input::TermRead
};
use termion::raw::{
    IntoRawMode,
    RawTerminal,
};

pub enum Direction {
    Right,
    Left,
    Up,
    Down
}

pub enum Zoom {
    In,
    Out
}

pub enum UserAction {
    TranslateCamera(Direction),
    ZoomCamera(Zoom),
    TogglePause,
    Quit,
    Nop
}

pub struct Inputs {
    keys: termion::input::Keys<AsyncReader>,
    _stdout: RawTerminal<Stdout>
}

impl Default for Inputs {
    fn default() -> Self {
        Self::new()
    }
}

impl Inputs {
    pub fn new() -> Inputs {
        Inputs {
            keys: termion::async_stdin().keys(),
            _stdout: io::stdout().into_raw_mode().unwrap()
        }
    }

    pub fn read_keyboard(&mut self) -> UserAction {
        if let Some(Ok(key)) = self.keys.next() {
            match key {
                Key::Esc => UserAction::Quit,
                Key::Left => UserAction::TranslateCamera(Direction::Left),
                Key::Right => UserAction::TranslateCamera(Direction::Right),
                Key::Up => UserAction::TranslateCamera(Direction::Up),
                Key::Down => UserAction::TranslateCamera(Direction::Down),
                Key::Char('z') => UserAction::ZoomCamera(Zoom::In),
                Key::Char('s') => UserAction::ZoomCamera(Zoom::Out),
                Key::Char('p') => UserAction::TogglePause,
                _ => UserAction::Nop
            }
        } else {
            UserAction::Nop
        }
    }
}
