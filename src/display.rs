use std::io::stdout;
use std::io::Write;
use termion;

pub struct Display {
    last_image: Vec<Vec<(u8, u8, u8)>>
}

impl Display {
    pub fn new() -> Display {
        Display {
            last_image: Vec::new()
        }
    }

    pub fn init(&self) {
        print!("{}", termion::clear::All);
        stdout().flush().unwrap();
    }

    pub fn render(&mut self, image: &Vec<Vec<(u8, u8, u8)>>) {
        /* Note : I deliberately ignore the case where the number of lines or columns of the image is 0.
         * It doesn't make any sense anyway (should be forbidden at configuration level). */

        if (image.len() != self.last_image.len()) || (image[0].len() != self.last_image[0].len()) {
            self.last_image = vec![vec![(0, 0, 0); image[0].len()]; image.len()];
        }

        for x in 0..image.len() {
            for y in 0..image[0].len() {
                if image[x][y] != self.last_image[x][y] {
                    let (r, g, b) = image[x][y];
                    print!("{}{}\u{2588}",
                           termion::cursor::Goto((x + 1) as u16, (y + 1) as u16),
                           termion::color::Fg(termion::color::AnsiValue::rgb(to_ansi_value(r), to_ansi_value(g), to_ansi_value(b))));
                    self.last_image[x][y] = image[x][y].clone();
                }
            }
        }
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

