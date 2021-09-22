use crate::input::InputListener;
use gl_matrix::common::to_radian;
use glam::{EulerRot, Mat4, Quat, Vec3};
use glutin::event::VirtualKeyCode;
use std::time::Instant;

pub struct Camera {
    pub persp: Mat4,
    pub origin: Vec3,
    pub direction: Quat,
    movement: Vec3,
    last_update: Instant,
}

impl Camera {
    pub fn new() -> Self {
        let origin = Vec3::new(1280.0, 48.0, -3295.0);
        let direction = Quat::IDENTITY;

        let persp = Mat4::perspective_lh(to_radian(45.0), 16.0 / 9.0, 10.0, 10000.0);
        Camera {
            persp,
            origin,
            direction,
            movement: Vec3::ZERO,
            last_update: Instant::now(),
        }
    }
    pub fn update(&mut self) {
        let now = Instant::now();
        let delta = now - self.last_update;
        self.last_update = now;

        let move_dir = self.direction * self.movement;
        self.origin += move_dir * 256.0 * delta.as_secs_f32();
    }
}

impl InputListener for Camera {
    fn on_input_change(&mut self, key: VirtualKeyCode, pressed: bool) {
        match key {
            VirtualKeyCode::Z => self.movement.z = if pressed { 1.0 } else { 0.0 },
            VirtualKeyCode::S => self.movement.z = if pressed { -1.0 } else { 0.0 },
            VirtualKeyCode::Q => self.movement.x = if pressed { -1.0 } else { 0.0 },
            VirtualKeyCode::D => self.movement.x = if pressed { 1.0 } else { 0.0 },
            VirtualKeyCode::Space => self.movement.y = if pressed { 1.0 } else { 0.0 },
            VirtualKeyCode::C => self.movement.y = if pressed { -1.0 } else { 0.0 },

            _ => (),
        }
        self.movement = self.movement.normalize_or_zero();
    }

    fn on_mouse_move(&mut self, delta: (f64, f64)) {
        let delta = (to_radian(delta.0 as f32), to_radian(delta.1 as f32));
        let mut euler = self.direction.to_euler(EulerRot::YXZ);
        euler.0 += delta.0;
        euler.1 += delta.1;
        self.direction = Quat::from_euler(EulerRot::YXZ, euler.0, euler.1, euler.2);
    }
}
