mod camera;
mod input;
mod render;
mod sys;
mod wad;
use camera::Camera;
use glutin::context::{ContextApi, ContextAttributesBuilder};
use glutin::prelude::*;
use glutin::surface::SwapInterval;
use glutin::{
    config::ConfigTemplateBuilder,
    display::GetGlDisplay,
    prelude::{GlConfig, GlDisplay},
};
use glutin_winit::{self, DisplayBuilder};
use input::Input;
use raw_window_handle::HasRawWindowHandle;
use render::doom_gl::DoomGl;
use std::{cell::RefCell, num::NonZeroU32, path::Path, rc::Rc};
use winit::{
    dpi::{PhysicalSize, Size},
    event::{DeviceEvent, ElementState, Event, KeyboardInput, WindowEvent},
    event_loop::EventLoopBuilder,
    window::{CursorGrabMode, WindowBuilder},
};

use doom_app::GlWindow;
use sys::content::Content;
use wad::file::WadFile;

fn main() {
    let event_loop = EventLoopBuilder::new().build();
    let size = Size::Physical(PhysicalSize::new(1680, 1050));
    let window_builder = Some(
        WindowBuilder::new()
            .with_inner_size(size)
            .with_resizable(false)
            .with_title("DOOM")
            .with_transparent(true),
    );
    let template = ConfigTemplateBuilder::new().with_alpha_size(8);
    let display_builder = DisplayBuilder::new().with_window_builder(window_builder);
    let (mut window, gl_config) = display_builder
        .build(&event_loop, template, |configs| {
            configs
                .reduce(|accum, config| {
                    let transparency_check = config.supports_transparency().unwrap_or(false)
                        & !accum.supports_transparency().unwrap_or(false);

                    if transparency_check || config.num_samples() > accum.num_samples() {
                        config
                    } else {
                        accum
                    }
                })
                .unwrap()
        })
        .unwrap();

    let raw_window_handle = window.as_ref().map(|window| window.raw_window_handle());
    let gl_display = gl_config.display();
    let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(raw_window_handle);
    let mut not_current_gl_context = Some(unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .unwrap_or_else(|_| {
                gl_display
                    .create_context(&gl_config, &fallback_context_attributes)
                    .expect("failed to create context")
            })
    });

    let mut state = None;
    let mut renderer = None;
    let mut focus = CursorGrabMode::Confined;
    let mut input = Input::new();
    let mut content = None;
    let mut camera = None;

    event_loop.run(move |event, window_target, control_flow| {
        control_flow.set_poll();
        match event {
            Event::Resumed => {
                let window = window.take().unwrap_or_else(|| {
                    let window_builder = WindowBuilder::new().with_transparent(true);
                    glutin_winit::finalize_window(window_target, window_builder, &gl_config)
                        .unwrap()
                });

                let gl_window = GlWindow::new(window, &gl_config);
                gl_window.window.set_cursor_grab(focus).unwrap();
                gl_window.window.set_cursor_visible(false);

                // Make it current.
                let gl_context = not_current_gl_context
                    .take()
                    .unwrap()
                    .make_current(&gl_window.surface)
                    .unwrap();

                // The context needs to be current for the Renderer to set up shaders and
                // buffers. It also performs function loading, which needs a current context on
                // WGL.
                renderer.get_or_insert_with(|| DoomGl::new(&gl_display));
                let file = WadFile::new(Path::new("base/doom.wad")).unwrap();
                content = Some(Content::new(file));

                camera = Some(Rc::new(RefCell::new(Camera::new())));
                input.listeners.push(camera.as_mut().unwrap().clone());

                // Try setting vsync.
                if let Err(res) = gl_window
                    .surface
                    .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
                {
                    eprintln!("Error setting vsync: {:?}", res);
                }

                assert!(state.replace((gl_context, gl_window)).is_none());
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Focused(f) => {
                    focus = if f {
                        CursorGrabMode::Confined
                    } else {
                        CursorGrabMode::None
                    };
                    if let Some((_, gl_window)) = &state {
                        gl_window.window.set_cursor_grab(focus).unwrap();
                        gl_window
                            .window
                            .set_cursor_visible(focus != CursorGrabMode::None);
                    }
                }
                WindowEvent::CloseRequested => control_flow.set_exit(),
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
                if let Some((gl_context, gl_window)) = &state {
                    if focus != CursorGrabMode::None {
                        camera.as_mut().unwrap().try_borrow_mut().unwrap().update();
                    }

                    content.as_mut().unwrap().maps[0]
                        .render(&camera.as_mut().unwrap().try_borrow_mut().unwrap());
                    gl_window.surface.swap_buffers(gl_context).unwrap();
                }
            }
            _ => (),
        }
    });
}
