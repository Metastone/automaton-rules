use crate::camera::Image;

pub trait Display {
    fn render(&mut self, image: & Image);

    fn clean(&mut self);
}

pub struct DummyDisplay;

impl DummyDisplay {
    pub fn new() -> DummyDisplay {
        DummyDisplay {}
    }
}

impl Display for DummyDisplay {
    fn render(&mut self, _image: & Image) {}

    fn clean(&mut self) {}
}
