use crate::camera::Image;

pub trait Display {
    fn render(&mut self, _image: & Image) {}

    fn clean(&mut self) {}
}

#[derive(Default)]
pub struct DummyDisplay;

impl DummyDisplay {
    pub fn new() -> DummyDisplay {
        DummyDisplay {}
    }
}

impl Display for DummyDisplay {}

