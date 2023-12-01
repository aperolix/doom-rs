use kabal_core::frame_timer::FrameTimer;
use winit::keyboard::Key;
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
use winit::{
    event::{DeviceEvent, Event, KeyEvent, WindowEvent},
    event_loop::{EventLoop, EventLoopBuilder},
    keyboard::NamedKey,
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
    fn recreate_swap_chain(&mut self);
    fn cleanup_swap_chain(&self);
    fn wait_device_idle(&mut self);
    fn resize_framebuffer(&mut self);
    fn focus_changed(&mut self, focused: bool);
    fn window_ref(&self) -> &winit::window::Window;
    fn on_keyboard_event(&mut self, key_event: &KeyEvent);
    fn on_mouse_move(&mut self, x: f64, y: f64);
}

pub struct ProgramProc {
    pub event_loop: EventLoop<()>,
}

impl ProgramProc {
    pub fn new() -> Self {
        let event_loop = EventLoopBuilder::new().build().unwrap();
        ProgramProc { event_loop }
    }

    /// The app never exit this function until an exit request is sent
    /// All windows event go through here and game main frame is called as
    /// frenquently as possible
    pub fn main_loop<A: 'static + KabalApp>(
        self,
        mut app: A,
    ) -> Result<(), impl std::error::Error> {
        let mut frame_timer = FrameTimer::new();

        // Run the loop until exit
        self.event_loop
            .run(move |event, window_target| match event {
                Event::WindowEvent { event, .. } => match event {
                    // This is called whenever the user close the window with the OS way
                    WindowEvent::CloseRequested => {
                        app.wait_device_idle();
                        window_target.exit();
                    }
                    // User press any key on keyboard
                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.key_without_modifiers().as_ref() == Key::Named(NamedKey::Escape) {
                            // For now exit when escape is pressed, we may need something better
                            app.wait_device_idle();
                            window_target.exit();
                        } else {
                            // Forward all other keys to the app
                            app.on_keyboard_event(&event);
                        }
                    }
                    // Window is resized, let the render system be aware of it
                    WindowEvent::Resized(_new_size) => {
                        app.wait_device_idle();
                        app.resize_framebuffer();
                    }
                    // Changing focus, handle input loss, auto pause etc...
                    WindowEvent::Focused(focused) => {
                        app.focus_changed(focused);
                    }
                    // This is where the game actually run
                    WindowEvent::RedrawRequested => {
                        let delta_time = frame_timer.delta_time();
                        app.run_frame(delta_time);

                        // println!("FPS: {}", frame_timer.fps());

                        frame_timer.tick();
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
                Event::AboutToWait => {
                    app.window_ref().request_redraw();
                }
                // Clean up game before exiting
                Event::LoopExiting => {
                    app.wait_device_idle();
                }
                Event::Resumed => {}
                _ => (),
            })
    }
}

impl Default for ProgramProc {
    fn default() -> Self {
        Self::new()
    }
}
