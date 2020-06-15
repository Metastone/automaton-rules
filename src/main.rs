#[macro_use]
extern crate log;

mod compiler;
mod automaton;
mod camera;
mod display;
mod inputs;
mod executor;

use std::{
    env,
    process,
};

use executor::{
    execute,
    Conf,
    MaxIterationCount
};

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        error!("USAGE : <automaton_file_path>");
        process::exit(1);
    }
    let file_name = &args[1];

    execute(&Conf {
        file_name,
        with_display: true,
        iteration_delay: 10,
        max_iteration_count: MaxIterationCount::Infinite,
    });
}
