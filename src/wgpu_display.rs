use winit::{
    event::{Event, WindowEvent, ElementState, VirtualKeyCode, KeyboardInput},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use crate::display::Display;
use std::borrow::Cow::Borrowed;

pub struct WgpuDisplay {
    event_loop: EventLoop<()>,
    window: Window,
}

impl WgpuDisplay {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = winit::window::Window::new(&event_loop).unwrap();
        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput {
                        input,
                        ..
                    } => {
                        match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        }
                    }
                    _ => {}
                }
                _ => {}
            }
        });
        WgpuDisplay {
            event_loop,
            window
        }
    }
}

impl Display for WgpuDisplay {

}
