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

use std::time::Duration;
use std::thread::sleep;
use compiler::semantic::{Rules, parse};
use automaton::Automaton;
use camera::Camera;
use display::Display;
use inputs::{Inputs, UserAction};

// TODO Make traits for inputs and display in order to allow different displays.
// TODO Capture an image with camera and change only if necessary.
// TODO index states by int, not String

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
    let mut display = Display::new(&camera);
    let mut inputs = Inputs::new();

    display.init();

    loop {
        match inputs.read_keyboard() {
            UserAction::TranslateCamera(direction) => { camera.translate(&direction); },
            UserAction::TogglePause => { /* TODO */ },
            UserAction::ChangeSimulationSpeed => { /* TODO */ },
            UserAction::Quit => {
                break;
            },
            UserAction::Nop => {}
        }

        display.render(&camera, &automaton);
        automaton.tick();
        sleep(Duration::from_millis(10));
    }

    display.clean();
}
