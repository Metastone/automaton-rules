use crate::automaton::Automaton;
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
            size: (200, 50)
        }
    }

    pub fn capture(&self, automaton: &Automaton) -> Vec<Vec<(u8, u8, u8)>> {
        let mut image = Vec::new();
        let grid = automaton.get_grid();
        let size = automaton.get_size();

        for x in 0..size.0 {
            let mut column = Vec::new();
            for y in 0..size.1 {
                let state_name = &grid[y * size.0 + x];
                column.push(automaton.get_color(state_name));
            }
            image.push(column);
        }

        image
    }

    pub fn translate(&mut self, direction: &Direction) {}

    pub fn get_position(& self) -> &(isize, isize) {
        &self.position
    }
}
