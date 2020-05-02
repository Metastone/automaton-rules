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

use std::{
    env,
    time::{Instant, Duration},
    thread::sleep,
    io,
    process
};
use compiler::semantic::{Rules, parse};
use automaton::Automaton;
use camera::Camera;
use display::Display;
use inputs::{Inputs, UserAction};
use termion::raw::IntoRawMode;

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        error!("USAGE : <automaton_file_path>");
        process::exit(1);
    }
    let file_name = &args[1];

    match parse(file_name) {
        Ok(rules) => {
            info!("Cellular automaton rules where parsed successfully from file {}.", file_name);
            run(rules);
        },
        Err(errors) => {
            error!("Cellular automaton rules could not be parsed from file {}.", file_name);
            for error in &errors {
                error!("{}", error);
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

    while i < 500 {
        match inputs.read_keyboard() {
            UserAction::TranslateCamera(direction) => { camera.translate(&direction); },
            UserAction::ZoomCamera(zoom) => { camera.zoom(&zoom); },
            UserAction::TogglePause => {
                pause = !pause;
                if pause {
                    runtime_duration += start.elapsed();
                } else {
                    start = Instant::now();
                }
            },
            UserAction::Quit => {
                break;
            },
            UserAction::Nop => {}
        }

        let image = camera.capture(&automaton);
        display.render(&image);
        sleep(Duration::from_millis(10));

        if !pause {
            automaton.tick();
            i += 1;
        }
    }

    display.clean();

    if !pause {
        runtime_duration += start.elapsed();
    }
    println!("Over. {} iterations / s", (i as f32 / runtime_duration.as_millis() as f32)*1000.0);
}
