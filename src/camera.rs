use crate::inputs::Direction;

/// The camera's (0,0) position is at the upper-left of the field of view.
pub struct Camera {
    position: (isize, isize),
    size: (usize, usize)
}

impl Camera {
    pub fn new(x: isize, y: isize) -> Camera {
        Camera {
            position: (x, y),
            size: (200, 100)
        }
    }

    pub fn translate(&mut self, direction: &Direction) {}

    pub fn get_position(& self) -> &(isize, isize) {
        &self.position
    }

    pub fn get_size(& self) -> &(usize, usize) {
        &self.size
    }
}
