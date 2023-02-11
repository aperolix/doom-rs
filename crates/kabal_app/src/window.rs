use kabal_core::frame_timer::FrameTimer;
use winit::{
    event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{EventLoop, EventLoopBuilder},
};

/// Initialize the game window
pub fn init_window(
    event_loop: &EventLoop<()>,
    title: &str,
    width: u32,
    height: u32,
) -> winit::window::Window {
    winit::window::WindowBuilder::new()
        .with_title(title)
        .with_inner_size(winit::dpi::LogicalSize::new(width, height))
        .build(event_loop)
        .expect("Failed to create window.")
}

/// Base trait for an app
pub trait KabalApp {
    fn run_frame(&mut self, delta_time: f32);
    fn recreate_swapchain(&mut self);
    fn cleanup_swapchain(&self);
    fn wait_devide_idle(&mut self);
    fn resize_framebuffer(&mut self);
    fn focus_changed(&mut self, focused: bool);
    fn window_ref(&self) -> &winit::window::Window;
    fn on_keyboard_event(&mut self, key_code: VirtualKeyCode, state: ElementState);
    fn on_mouse_move(&mut self, x: f64, y: f64);
}

pub struct ProgramProc {
    pub event_loop: EventLoop<()>,
}

impl ProgramProc {
    pub fn new() -> Self {
        let event_loop = EventLoopBuilder::new().build();
        ProgramProc { event_loop }
    }

    /// The app never exit this function until an exit request is sent
    /// All windows event go through here and game main frame is called as
    /// frenquently as possible
    pub fn main_loop<A: 'static + KabalApp>(self, mut app: A) {
        let mut frame_timer = FrameTimer::new();

        // Run the loop until exit
        self.event_loop
            .run(move |event, _, control_flow| match event {
                Event::WindowEvent { event, .. } => match event {
                    // This is called whenever the user close the window with the OS way
                    WindowEvent::CloseRequested => {
                        app.wait_devide_idle();
                        control_flow.set_exit();
                    }
                    // User press any key on keyboard
                    WindowEvent::KeyboardInput { input, .. } => {
                        let KeyboardInput {
                            virtual_keycode,
                            state,
                            ..
                        } = input;
                        match (virtual_keycode, state) {
                            // For now exit when escape is pressed, we may need something better
                            (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                                app.wait_devide_idle();
                                control_flow.set_exit();
                            }
                            // Forward all other keys to the app
                            (Some(key_code), state) => app.on_keyboard_event(key_code, state),
                            _ => (),
                        }
                    }
                    // Window is resized, let the render system be aware of it
                    WindowEvent::Resized(_new_size) => {
                        app.wait_devide_idle();
                        app.resize_framebuffer();
                    }
                    // Changing focus, handle input loss, auto pause etc...
                    WindowEvent::Focused(focused) => {
                        app.focus_changed(focused);
                    }
                    _ => (),
                },
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    app.on_mouse_move(delta.0, delta.1);
                }
                // No more event in queue, we can request a render
                Event::MainEventsCleared => {
                    app.window_ref().request_redraw();
                }
                // This is where the game actually run
                Event::RedrawRequested(_window_id) => {
                    let delta_time = frame_timer.delta_time();
                    app.run_frame(delta_time);

                    // println!("FPS: {}", frame_timer.fps());

                    frame_timer.tick();
                }
                // Clean up game before exiting
                Event::LoopDestroyed => {
                    app.wait_devide_idle();
                }
                Event::Resumed => {}
                _ => (),
            });
    }
}

impl Default for ProgramProc {
    fn default() -> Self {
        Self::new()
    }
}
