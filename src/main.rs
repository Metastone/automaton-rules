#[macro_use]
extern crate log;
extern crate env_logger;
extern crate termion;
extern crate rand;

mod compiler;
mod automaton;
mod camera;
mod display;
mod inputs;

use std::time::{Instant, Duration};
use std::thread::sleep;
use std::io;
use compiler::semantic::{Rules, parse};
use automaton::Automaton;
use camera::Camera;
use display::Display;
use inputs::{Inputs, UserAction};
use termion::raw::IntoRawMode;

// TODO Make traits for inputs and display in order to allow different displays.
// TODO index states by int, not String
// TODO find a way to represent initial state
// TODO increase performances by avoiding having to recompute the colors in display for each cell

fn main() {
    env_logger::init();

    let file_name = "/home/metastone/Documents/projects/mutations/resources/game_of_life.txt";
    match parse(file_name) {
        Ok(rules) => {
            info!("Cellular automaton rules where parsed successfully from file {}.", file_name);
            run(rules);
        },
        Err(errors) => {
            error!("Cellular automaton rules could not be parsed from file {}.", file_name);
            for i in 0..errors.len() {
                error!("{}", errors[i]);
            }
        }
    }
}

fn run(rules: Rules) {
    let mut automaton = Automaton::new(rules);
    let mut camera = Camera::new(0, 0);
    let mut display = Display::new();
    let mut inputs = Inputs::new();

    let _stdout = io::stdout().into_raw_mode().unwrap();
    display.init();

    let mut start = Instant::now();
    let mut runtime_duration = Duration::new(0, 0);
    let mut i = 0;
    let mut pause = false;

    while i < 300 {
        match inputs.read_keyboard() {
            UserAction::TranslateCamera(direction) => { camera.translate(&direction); },
            UserAction::TogglePause => {
                pause = !pause;
                if pause {
                    runtime_duration += start.elapsed();
                } else {
                    start = Instant::now();
                }
            },
            UserAction::ChangeSimulationSpeed => { /* TODO */ },
            UserAction::Quit => {
                break;
            },
            UserAction::Nop => {}
        }

        if !pause {
            let image = camera.capture(&automaton);
            display.render(&image);
            automaton.tick();
            sleep(Duration::from_millis(10));
            i += 1;
        }
    }

    display.clean();

    runtime_duration += start.elapsed();
    println!("Over. {} iterations / s", (i as f32 / runtime_duration.as_millis() as f32)*1000.0);
}
