#[macro_use]
extern crate log;

pub mod executor;
pub mod compiler;
pub mod automaton;
pub mod camera;
pub mod display;
pub mod terminal_display;
pub mod inputs;

#[cfg(feature = "wgpu_rendering")]
pub mod wgpu_display;
#[macro_use]
extern crate wgpu;
