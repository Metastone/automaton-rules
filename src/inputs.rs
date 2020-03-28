pub enum Direction {
    Right,
    Left,
    Up,
    Down
}

pub enum UserAction {
    TranslateCamera(Direction),
    TogglePause,
    ChangeSimulationSpeed,
    Quit,
    Nop
}

pub struct Inputs {}

impl Inputs {
    pub fn new() -> Inputs {
        Inputs {}
    }

    pub fn read_keyboard(&mut self) -> UserAction {
        UserAction::Nop
    }
}
