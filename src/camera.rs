use crate::automaton::Automaton;
use crate::inputs::Direction;

const TRANSLATION_OFFSET: usize = 1;

pub struct Image {
    pub grid: Vec<Vec<usize>>,
    pub colors: Vec<(u8, u8, u8)>   // 16M color
}

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

    pub fn capture(&self, automaton: &Automaton) -> Image {
        let mut grid = Vec::new();
        for x_c in 0..self.size.0 {
            let mut column = Vec::new();
            for y_c in 0..self.size.1 {
                let x = x_c as isize + self.position.0;
                let y = y_c as isize + self.position.1;
                column.push(automaton.get_state(x, y));
            }
            grid.push(column);
        }

        Image {
            grid: grid,
            colors: automaton.get_colors()
        }
    }

    pub fn translate(&mut self, direction: &Direction) {
        match direction {
            Direction::Left => { self.position.0 -= TRANSLATION_OFFSET as isize; }
            Direction::Right => { self.position.0 += TRANSLATION_OFFSET as isize; }
            Direction::Up => { self.position.1 -= TRANSLATION_OFFSET as isize; }
            Direction::Down => { self.position.1 += TRANSLATION_OFFSET as isize; }
        }
    }
}
