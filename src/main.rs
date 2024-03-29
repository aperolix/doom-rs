mod camera;
mod input;
mod render;
mod sys;
mod wad;
use camera::Camera;
use glutin::context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext};
use glutin::prelude::*;
use glutin::surface::{Surface, WindowSurface};
use glutin::surface::{SurfaceAttributesBuilder, SwapInterval};
use glutin::{
    config::ConfigTemplateBuilder,
    display::GetGlDisplay,
    prelude::{GlConfig, GlDisplay},
};
use glutin_winit::{self, DisplayBuilder};
use input::Input;
use kabal_app::window::{KabalApp, ProgramProc};
use kabal_render::doom_gl::DoomGl;
use raw_window_handle::HasRawWindowHandle;
use std::{cell::RefCell, num::NonZeroU32, path::Path, rc::Rc};
use winit::event::VirtualKeyCode;
use winit::{
    event::ElementState,
    window::{CursorGrabMode, WindowBuilder},
};

use sys::content::Content;
use wad::file::WadFile;

const WINDOW_TITLE: &str = "DOOM";
const WINDOW_WIDTH: u32 = 1680;
const WINDOW_HEIGHT: u32 = 1050;

struct DoomApp {
    window: winit::window::Window,
    surface: Surface<WindowSurface>,
    context: PossiblyCurrentContext,

    focused: bool,

    content: Content,
    camera: Rc<RefCell<Camera>>,
    input: Input,
}

impl DoomApp {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let window_builder = Some(
            WindowBuilder::new()
                .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                .with_resizable(false)
                .with_title(WINDOW_TITLE)
                .with_transparent(true),
        );
        let template = ConfigTemplateBuilder::new().with_alpha_size(8);
        let display_builder = DisplayBuilder::new().with_window_builder(window_builder);
        let (window, gl_config) = display_builder
            .build(event_loop, template, |configs| {
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

        // Create GL window
        let window = window.unwrap();
        let (width, height): (u32, u32) = window.inner_size().into();
        let raw_window_handle = window.raw_window_handle();
        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle,
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );

        let surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };

        // Make it current.
        let context = not_current_gl_context
            .take()
            .unwrap()
            .make_current(&surface)
            .unwrap();

        window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
        window.set_cursor_visible(false);

        DoomGl::new(&gl_display);

        let file = WadFile::new(Path::new("base/doom.wad")).unwrap();
        let content = Content::new(file);

        let camera = Rc::new(RefCell::new(Camera::new()));
        let mut input = Input::new();
        input.listeners.push(camera.clone());

        // Try setting vsync.
        if let Err(res) =
            surface.set_swap_interval(&context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
        {
            eprintln!("Error setting vsync: {:?}", res);
        }

        DoomApp {
            window,
            surface,
            context,
            focused: true,
            content,
            camera,
            input,
        }
    }
}

impl KabalApp for DoomApp {
    fn run_frame(&mut self, _delta_time: f32) {
        if self.focused {
            self.camera.try_borrow_mut().unwrap().update();
        }

        self.content.maps[0].render(&self.camera.try_borrow_mut().unwrap());
        self.surface.swap_buffers(&self.context).unwrap();
    }

    fn recreate_swapchain(&mut self) {}

    fn cleanup_swapchain(&self) {}

    fn wait_devide_idle(&mut self) {}

    fn resize_framebuffer(&mut self) {}

    fn window_ref(&self) -> &winit::window::Window {
        &self.window
    }

    fn focus_changed(&mut self, focused: bool) {
        let mode = if focused {
            CursorGrabMode::Confined
        } else {
            CursorGrabMode::None
        };
        self.window.set_cursor_grab(mode).unwrap();
        self.window.set_cursor_visible(!focused);
    }

    fn on_keyboard_event(&mut self, key_code: VirtualKeyCode, state: ElementState) {
        if self.focused {
            self.input
                .register_input_event(key_code, state == ElementState::Pressed)
        }
    }

    fn on_mouse_move(&mut self, x: f64, y: f64) {
        if self.focused {
            self.input.register_mouse_move((x, y));
        }
    }
}

fn main() {
    let proc = ProgramProc::new();
    let app = DoomApp::new(&proc.event_loop);

    proc.main_loop(app);
}
