use crate::automaton::Automaton;
use crate::inputs::{Direction, Zoom};

const TRANSLATION_OFFSET: usize = 5;
const ZOOM_FACTOR: f64 = 1.2;

pub struct Image {
    pub grid: Vec<Vec<usize>>,
    pub colors: Vec<(u8, u8, u8)>   // 16M color
}

impl Image {
    fn new(size: (f64, f64), automaton: &Automaton) -> Image {
        Image {
            grid: vec![vec![0; size.1 as usize]; size.0 as usize],
            colors: automaton.get_colors()
        }
    }

    fn resize(&mut self, new_size: (f64, f64)) {
        self.grid = vec![vec![0; new_size.1 as usize]; new_size.0 as usize];
    }

    fn capture(&mut self, camera_pos: (isize, isize), automaton: &Automaton) {
        for (x_c, column) in self.grid.iter_mut().enumerate() {
            for (y_c, pixel) in column.iter_mut().enumerate() {
                let x = x_c as isize + camera_pos.0;
                let y = y_c as isize + camera_pos.1;
                *pixel = automaton.get_state(x, y);
            }
        }
    }
}

/// The camera's (0,0) position is at the upper-left of the field of view.
pub struct Camera {
    position: (isize, isize),
    size: (f64, f64), // The size is stored as floating-point number because it makes zooming more consistent
    image: Image
}

impl Camera {
    pub fn new(x: isize, y: isize, automaton: &Automaton) -> Camera {
        let size = (200.0, 50.0);
        Camera {
            position: (x, y),
            size,
            image: Image::new(size, automaton)
        }
    }

    pub fn capture(&mut self, automaton: &Automaton) -> &Image {
        self.image.capture(self.position, automaton);
        &self.image
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
        self.image.resize(self.size);
    }
}
