mod camera;
mod input;
mod render;
mod sys;
mod wad;
use camera::Camera;
use glutin::{
    dpi::{PhysicalSize, Size},
    event::{DeviceEvent, ElementState, Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{CursorGrabMode, WindowBuilder},
    ContextBuilder,
};
use input::Input;
use render::doom_gl::DoomGl;
use std::{cell::RefCell, path::Path, rc::Rc};

use sys::content::Content;
use wad::file::WadFile;

fn main() {
    let el = EventLoop::new();
    let size = Size::Physical(PhysicalSize::new(1680, 1050));
    let wb = WindowBuilder::new()
        .with_inner_size(size)
        .with_resizable(false)
        .with_title("DOOM");
    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };
    DoomGl::init(windowed_context.context());
    let mut input = Input::new();

    let file = WadFile::new(Path::new("base/doom.wad")).unwrap();

    let content = Content::new(file);

    let camera = Rc::new(RefCell::new(Camera::new()));

    input.listeners.push(camera.clone());

    let mut focus = CursorGrabMode::Confined;
    windowed_context.window().set_cursor_grab(focus).unwrap();
    windowed_context.window().set_cursor_visible(false);

    el.run(move |event, _, control_flow| {
        //println!("{:?}", event);
        *control_flow = ControlFlow::Poll;

        match event {
            Event::LoopDestroyed => (),
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Focused(f) => {
                    focus = if f {
                        CursorGrabMode::Confined
                    } else {
                        CursorGrabMode::None
                    };
                    windowed_context.window().set_cursor_grab(focus).unwrap();
                    windowed_context
                        .window()
                        .set_cursor_visible(focus != CursorGrabMode::None);
                }
                WindowEvent::Resized(physical_size) => windowed_context.resize(physical_size),
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(virtual_code),
                            state,
                            ..
                        },
                    ..
                } => {
                    if focus != CursorGrabMode::None {
                        input.register_input_event(virtual_code, state == ElementState::Pressed)
                    }
                }
                _ => (),
            },
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                if focus != CursorGrabMode::None {
                    input.register_mouse_move(delta)
                }
            }

            Event::MainEventsCleared => {
                if focus != CursorGrabMode::None {
                    camera.try_borrow_mut().unwrap().update();
                }
                content.maps[0].render(&camera.try_borrow_mut().unwrap());
                windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}
