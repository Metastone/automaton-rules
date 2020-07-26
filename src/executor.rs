extern crate env_logger;
extern crate termion;
extern crate rand;

use std::{
    time::{Instant, Duration},
    thread::sleep,
};
use crate::compiler::semantic::{Rules, parse};
use crate::automaton::Automaton;
use crate::camera::Camera;
use crate::display::DummyDisplay;
use crate::rendy_display::RendyDisplay;
use crate::terminal_display::TerminalDisplay;
use crate::display::Display;
use crate::inputs::{Inputs, UserAction};

pub enum MaxIterationCount {
    Infinite,
    Finite(usize)
}

pub struct Conf<'a> {
    pub file_name: &'a str,
    pub with_display: bool,
    pub iteration_delay: usize,
    pub max_iteration_count: MaxIterationCount,
}

pub fn execute(conf: &Conf) {
    match parse(conf.file_name) {
        Ok(rules) => {
            info!("Cellular automaton rules where parsed successfully from file {}.", conf.file_name);
            execute_rules(conf, rules);
        },
        Err(errors) => {
            error!("Cellular automaton rules could not be parsed from file {}.", conf.file_name);
            for error in &errors {
                error!("{}", error);
            }
        }
    }
}

fn execute_rules(conf: &Conf, rules: Rules) {
    let mut automaton = Automaton::new(rules);
    let mut camera = Camera::new(0, 0, &automaton);
    let mut inputs = Inputs::new();
    let mut display: Box<dyn Display> = create_display(conf);

    let mut start = Instant::now();
    let mut runtime_duration = Duration::new(0, 0);
    let mut i = 0;
    let mut pause = false;

    let mut continue_simulation = true;
    while continue_simulation {
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

        if conf.with_display {
            let image = camera.capture(&automaton);
            display.render(&image);
            sleep(Duration::from_millis(10));
        }

        if !pause {
            automaton.tick();
            i += 1;
        }

        continue_simulation = match conf.max_iteration_count {
            MaxIterationCount::Infinite => true,
            MaxIterationCount::Finite(max) => i < max
        };
    }

    display.clean();

    if !pause {
        runtime_duration += start.elapsed();
    }
    println!("Over. {} iterations / s", (i as f32 / runtime_duration.as_millis() as f32)*1000.0);
}

fn create_display(conf: &Conf) -> Box<dyn Display> {
    if conf.with_display {
        if cfg!(feature = "rendy_rendering") {
            Box::new(RendyDisplay::new())
        } else {
            Box::new(TerminalDisplay::new())
        }
    } else {
        Box::new(DummyDisplay::new())
    }
}
