use crate::automaton::Automaton;
use crate::camera::Camera;
use std::io::stdout;
use std::io::Write;
use termion;

pub struct Display {
    size: (usize, usize)
}

impl Display {
    pub fn new(camera: &Camera) -> Display {
        Display {
            size: camera.get_size().clone()
        }
    }

    pub fn init(&self) {
        print!("{}", termion::clear::All);
        for x in 0..self.size.0 {
            for y in 0..self.size.1 {
                print!("{}{}\u{2588}",
                       termion::cursor::Goto((x + 1) as u16, (y + 1) as u16),
                       termion::color::Fg(termion::color::White));
            }
        }
        stdout().flush().unwrap();
    }

    pub fn render(&mut self, camera: &Camera, automaton: &Automaton) {
        let grid = automaton.get_grid();
        let size = automaton.get_size();
        for x in 1..(size.0+1) {
            for y in 1..(size.1+1) {
                let state_name = &grid[y*(size.0+2) + x];
                let (r, g, b) = automaton.get_color(state_name);
                print!("{}{}\u{2588}",
                       termion::cursor::Goto(x as u16, y as u16),
                       termion::color::Fg(termion::color::AnsiValue::rgb(to_ansi_value(r), to_ansi_value(g), to_ansi_value(b))));
            }
        }
        stdout().flush().unwrap();
    }

    pub fn clean(&mut self) {
        print!("{}", termion::cursor::Goto(1, (self.size.1 + 1) as u16));
        stdout().flush().unwrap();
    }
}

/// Map a [0; 255] value to a [0; 5] value
fn to_ansi_value(x: u8) -> u8 {
    // use f32 so that is works on 32 bits systems
    (x as f32 * 5.0 / 255.0).round() as u8
}

