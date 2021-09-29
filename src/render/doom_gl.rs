use cgmath::{Vector2, Vector3};
use glutin::{self, PossiblyCurrent};

pub mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct GVertex {
    pub pos: Vector3<f32>,
    pub uv: Vector2<f32>,
    pub light: f32,
}

//https://www.khronos.org/opengl/wiki/Debug_Output
extern "system" fn gl_debug_message_callback(
	_source: gl::types::GLenum,
	msg_type: gl::types::GLenum,
	_id: gl::types::GLuint,
	severity: gl::types::GLenum,
	_length: gl::types::GLsizei,
	message: *const gl::types::GLchar,
	_user_param: *mut std::ffi::c_void) {

    let msg_type = match msg_type {
        GL_DEBUG_TYPE_ERROR => { "Error" },
        GL_DEBUG_TYPE_DEPRECATED_BEHAVIOR => {"Deprecated Behavior"},
        GL_DEBUG_TYPE_UNDEFINED_BEHAVIOR => { "Undefined Behavior" },
        GL_DEBUG_TYPE_PORTABILITY => { "Portability" },
        GL_DEBUG_TYPE_PERFORMANCE => { "Performance" },
        GL_DEBUG_TYPE_MARKER => { "Marker" },
        GL_DEBUG_TYPE_PUSH_GROUP => { "push group" },
        GL_DEBUG_TYPE_POP_GROUP => { "pop group" },
        GL_DEBUG_TYPE_OTHER => { "other" },
        _ => { "<type not known>" },
    };

    let msg_dev = match severity {
        GL_DEBUG_SEVERITY_HIGH => { "High" },
        GL_DEBUG_SEVERITY_MEDIUM => { "Medium" },
        GL_DEBUG_SEVERITY_LOW => { "Low" },
        GL_DEBUG_SEVERITY_NOTIFICATION => { "Notification" },
        _ => {"<sev not known"},
    };

	let message = unsafe { std::ffi::CStr::from_ptr(message) };
	println!("GLerror : type {} / sev {} / {:?}", msg_type, msg_dev, message);
}

pub struct DoomGl {
    pub gl: gl::Gl,
}

impl DoomGl {
    pub fn new(gl_context: &glutin::Context<PossiblyCurrent>) -> Self {
        let gl = gl::Gl::load_with(|ptr| gl_context.get_proc_address(ptr) as *const _);

        unsafe {
            gl.Enable(gl::CULL_FACE);
            gl.Enable(gl::DEPTH_TEST);
            gl.DepthFunc(gl::LESS);
            gl.FrontFace(gl::CCW);
            //gl.Enable(gl::BLEND);
            //gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            //gl.gl.Disable(gl::CULL_FACE);
            //gl.gl.PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            gl.DebugMessageCallback(Some(gl_debug_message_callback), std::ptr::null() );// use ptr to an object for
        }

        DoomGl { gl }
    }

    pub fn create_texture(&self, image: &[u8], width: i32, height: i32) -> u32 {
        let mut id = [0u32; 1];
        unsafe {
            self.gl.GenTextures(1, id.as_mut_ptr());
            assert!(self.gl.GetError() == 0);
            self.gl.BindTexture(gl::TEXTURE_2D, id[0]);
            assert!(self.gl.GetError() == 0);
            self.gl
                .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            assert!(self.gl.GetError() == 0);
            self.gl
                .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            assert!(self.gl.GetError() == 0);
            self.gl
                .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            assert!(self.gl.GetError() == 0);
            self.gl
                .TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            assert!(self.gl.GetError() == 0);
            self.gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width,
                height,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                image.as_ptr() as *const _,
            );
            assert!(self.gl.GetError() == 0);
            self.gl.GenerateMipmap(gl::TEXTURE_2D);
        }

        id[0]
    }
}
