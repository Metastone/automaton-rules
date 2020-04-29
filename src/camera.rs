use crate::automaton::Automaton;
use crate::inputs::{Direction, Zoom};

const TRANSLATION_OFFSET: usize = 5;
const ZOOM_FACTOR: f64 = 1.2;

pub struct Image {
    pub grid: Vec<Vec<usize>>,
    pub colors: Vec<(u8, u8, u8)>   // 16M color
}

/// The camera's (0,0) position is at the upper-left of the field of view.
pub struct Camera {
    position: (isize, isize),
    size: (f64, f64)
}

impl Camera {
    pub fn new(x: isize, y: isize) -> Camera {
        Camera {
            position: (x, y),
            size: (200.0, 50.0)
        }
    }

    pub fn capture(&self, automaton: &Automaton) -> Image {
        let mut grid = Vec::new();
        for x_c in 0..(self.size.0 as usize) {
            let mut column = Vec::new();
            for y_c in 0..(self.size.1 as usize) {
                let x = x_c as isize + self.position.0;
                let y = y_c as isize + self.position.1;
                column.push(automaton.get_state(x, y));
            }
            grid.push(column);
        }

        Image {
            grid,
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

    pub fn zoom(&mut self, zoom: &Zoom) {
        let factor = match zoom {
            Zoom::In => 1.0 / ZOOM_FACTOR,
            Zoom::Out => ZOOM_FACTOR
        };
        self.size.0 *= factor;
        self.size.1 *= factor;
    }
}
