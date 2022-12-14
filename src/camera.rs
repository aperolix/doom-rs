use crate::input::InputListener;
use cgmath::{
    Deg, InnerSpace, Matrix4, Point3, Quaternion, Rotation, Rotation3, Vector2, Vector3, Zero,
};
use std::time::Instant;
use winit::event::VirtualKeyCode;

pub struct Camera {
    pub persp: Matrix4<f32>,
    pub origin: Point3<f32>,
    pub direction: Quaternion<f32>,
    movement: Vector3<f32>,
    last_update: Instant,
    yaw: Deg<f32>,
    pitch: Deg<f32>,
    last_delta: Vector2<f32>,
}

impl Camera {
    pub fn new() -> Self {
        let origin = Point3::new(-1280.0f32, 48.0f32, -3295.0f32);
        let direction = Rotation::look_at(Vector3::unit_x(), Vector3::unit_y());

        let persp = cgmath::perspective(Deg(45.0), 16.0 / 9.0, 10.0, 10000.0);

        Camera {
            persp,
            origin,
            direction,
            movement: Vector3::zero(),
            last_update: Instant::now(),
            yaw: Deg::zero(),
            pitch: Deg::zero(),
            last_delta: Vector2::zero(),
        }
    }
    pub fn update(&mut self) {
        let now = Instant::now();
        let delta = now - self.last_update;
        self.last_update = now;

        let move_dir = self.direction * self.movement;
        let move_speed_factor = 2.0;
        self.origin += move_dir * 256.0 * delta.as_secs_f32() * move_speed_factor;
    }
}

impl InputListener for Camera {
    fn on_input_change(&mut self, key: VirtualKeyCode, pressed: bool) {
        match key {
            VirtualKeyCode::Z => self.movement.z = if pressed { 1.0 } else { 0.0 },
            VirtualKeyCode::S => self.movement.z = if pressed { -1.0 } else { 0.0 },
            VirtualKeyCode::Q => self.movement.x = if pressed { 1.0 } else { 0.0 },
            VirtualKeyCode::D => self.movement.x = if pressed { -1.0 } else { 0.0 },
            VirtualKeyCode::Space => self.movement.y = if pressed { 1.0 } else { 0.0 },
            VirtualKeyCode::C => self.movement.y = if pressed { -1.0 } else { 0.0 },

            _ => (),
        }
        if !self.movement.is_zero() {
            self.movement = self.movement.normalize();
        }
    }

    fn on_mouse_move(&mut self, delta: (f64, f64)) {
        let x_sensitivity = 0.5;
        let y_sensitivity = 0.35;

        let delta = Vector2::new(delta.0 as f32, delta.1 as f32);
        let smoothed = (self.last_delta + delta) / 2.0;
        self.last_delta = delta;

        self.yaw -= Deg(smoothed.x as f32 * x_sensitivity);
        self.pitch += Deg(smoothed.y as f32 * y_sensitivity);

        if self.pitch > Deg(88.0) {
            self.pitch = Deg(88.0);
        } else if self.pitch < Deg(-88.0) {
            self.pitch = Deg(-88.0);
        }

        let quat_yaw: Quaternion<f32> = Rotation3::from_angle_y(self.yaw);
        let quat_pitch: Quaternion<f32> = Rotation3::from_angle_x(self.pitch);

        self.direction = quat_yaw * quat_pitch;
    }
}
