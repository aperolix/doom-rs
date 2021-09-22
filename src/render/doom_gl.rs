use glam::{Vec2, Vec3};
use glutin::{self, PossiblyCurrent};

pub mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct GVertex {
    pub pos: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub light: f32,
}

pub struct DoomGl {
    pub gl: gl::Gl,
}

impl DoomGl {
    pub fn init(gl_context: &glutin::Context<PossiblyCurrent>) -> Self {
        let gl = gl::Gl::load_with(|ptr| gl_context.get_proc_address(ptr) as *const _);

        DoomGl { gl }
    }
}
