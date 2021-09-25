mod camera;
mod input;
mod render;
mod wad;
use camera::Camera;
use glutin::{
    dpi::{PhysicalSize, Size},
    event::{DeviceEvent, ElementState, Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder,
};
use input::Input;
use render::doom_gl::gl;
use render::doom_gl::DoomGl;
use std::{cell::RefCell, path::Path, rc::Rc};

use wad::file::WadFile;
use wad::map::WadMap;

fn main() {
    let el = EventLoop::new();
    let size = Size::Physical(PhysicalSize::new(1680, 1050));
    let wb = WindowBuilder::new()
        .with_inner_size(size)
        .with_resizable(false)
        .with_title("DOOM");
    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };
    let gl = DoomGl::init(windowed_context.context());
    let mut input = Input::new();

    unsafe {
        gl.gl.Enable(gl::CULL_FACE);
        assert!(gl.gl.GetError() == 0);
        gl.gl.Enable(gl::DEPTH_TEST);
        assert!(gl.gl.GetError() == 0);
        gl.gl.DepthFunc(gl::LESS);
        assert!(gl.gl.GetError() == 0);
        gl.gl.FrontFace(gl::CCW);
        assert!(gl.gl.GetError() == 0);
        //gl.gl.Disable(gl::CULL_FACE);
        //gl.gl.PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
    }

    let file = WadFile::new(Path::new("base/doom.wad")).unwrap();

    let mut map = WadMap::load_map("E1M1", file, gl.gl).unwrap();

    map.prepare_render_finalize(map.prepare_render());

    let camera = Rc::new(RefCell::new(Camera::new()));

    input.listeners.push(camera.clone());

    let mut focus = true;
    windowed_context.window().set_cursor_grab(true).unwrap();
    windowed_context.window().set_cursor_visible(false);

    el.run(move |event, _, control_flow| {
        //println!("{:?}", event);
        *control_flow = ControlFlow::Poll;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Focused(f) => {
                    focus = f;
                    windowed_context.window().set_cursor_grab(focus).unwrap();
                    windowed_context.window().set_cursor_visible(!focus);
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
                    if focus {
                        input.register_input_event(virtual_code, state == ElementState::Pressed)
                    }
                }
                _ => (),
            },
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                if focus {
                    input.register_mouse_move(delta)
                }
            }

            Event::RedrawRequested(_) => (),
            _ => (),
        }
        if focus {
            camera.try_borrow_mut().unwrap().update();
        }
        map.render(&camera.try_borrow_mut().unwrap());
        windowed_context.swap_buffers().unwrap();
    });
}
