use std::io::{stdout, Write};
use termion;
use crate::camera::Image;

pub struct Display {
    last_image: Vec<Vec<usize>>,
    colors: Vec<(u8, u8, u8)>, // ansi color
    redraw: bool
}

impl Display {
    pub fn new() -> Display {
        Display {
            last_image: Vec::new(),
            colors: Vec::new(),
            redraw: true,
        }
    }

    pub fn init(&self) {
        print!("{}", termion::clear::All);
        stdout().flush().unwrap();
    }

    pub fn render(&mut self, image: & Image) {
        if self.colors.len() == 0 {
            self.colors = image.colors.iter()
                .map(|(r, g, b)| (to_ansi_value(*r), to_ansi_value(*g), to_ansi_value(*b)))
                .collect::<Vec<_>>();
        }

        // Note : The case where the number of lines or columns of the image is 0 should be forbidden at configuration level.

        if (image.grid.len() != self.last_image.len()) || (image.grid[0].len() != self.last_image[0].len()) {
            self.last_image = vec![vec![0; image.grid[0].len()]; image.grid.len()];
            self.redraw = true;
        }

        for x in 0..image.grid.len() {
            for y in 0..image.grid[0].len() {
                if self.redraw || image.grid[x][y] != self.last_image[x][y] {
                    let color_index = image.grid[x][y];
                    let (r, g, b) = self.colors[color_index];
                    print!("{}{}\u{2588}",
                           termion::cursor::Goto((x + 1) as u16, (y + 1) as u16),
                           termion::color::Fg(termion::color::AnsiValue::rgb(r, g, b)));
                    self.last_image[x][y] = image.grid[x][y].clone();
                }
            }
        }

        self.redraw = false;
        stdout().flush().unwrap();
    }

    pub fn clean(&mut self) {
        let cursor_vert_pos = if self.last_image.len() == 0 { 1 } else { self.last_image[0].len() + 1 };
        print!("{}{}", termion::cursor::Goto(1, cursor_vert_pos as u16), termion::color::Fg(termion::color::White));
        stdout().flush().unwrap();
    }
}

/// Map a [0; 255] value to a [0; 5] value
fn to_ansi_value(x: u8) -> u8 {
    (x as f64 * 5.0 / 255.0).round() as u8
}
