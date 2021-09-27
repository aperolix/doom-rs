use super::doom_gl::{gl, DoomGl};
use std::io::Read;
use std::{fs::File, path::Path};

pub struct Material {
    vs: u32,
    fs: u32,
    program: u32,
}

pub fn create_shader(name: &Path, shader_type: gl::types::GLenum) -> u32 {
    let mut file = File::open(name).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let length = content.len() as i32;

    let vs;
    unsafe {
        let gl = DoomGl::gl();
        vs = gl.CreateShader(shader_type);
        gl.ShaderSource(vs, 1, [content.as_ptr() as *const _].as_ptr(), &length);
        gl.CompileShader(vs);

        let mut status = 0;
        gl.GetShaderiv(vs, gl::COMPILE_STATUS, &mut status);
        if status != 1 {
            let mut message = [0u8; 1024];
            let mut length = 0;
            gl.GetShaderInfoLog(vs, 1024, &mut length, message.as_mut_ptr() as *mut _);

            let str = String::from_raw_parts(message.as_mut_ptr(), length as usize, 1024);
            panic!("{}", str);
        }
    }
    vs
}

impl Material {
    pub fn new(vs: &str, fs: &str) -> Self {
        let vs = create_shader(Path::new(vs), gl::VERTEX_SHADER);
        let fs = create_shader(Path::new(fs), gl::FRAGMENT_SHADER);

        let gl = DoomGl::gl();
        let program = unsafe { gl.CreateProgram() };
        unsafe {
            gl.AttachShader(program, vs);
            gl.AttachShader(program, fs);
            gl.LinkProgram(program);
        }

        Material { vs, fs, program }
    }

    pub fn get_uniform_location(&self, name: &str) -> i32 {
        let gl = DoomGl::gl();
        let location = unsafe { gl.GetUniformLocation(self.program, name.as_ptr() as *const _) };
        unsafe { assert!(gl.GetError() == 0) };
        location
    }

    pub fn get_attrib_location(&self, name: &str) -> i32 {
        let gl = DoomGl::gl();
        let location = unsafe { gl.GetAttribLocation(self.program, name.as_ptr() as *const _) };
        unsafe { assert!(gl.GetError() == 0) };
        location
    }

    pub fn bind(&self) {
        unsafe { DoomGl::gl().UseProgram(self.program) };
    }
}

impl Drop for Material {
    fn drop(&mut self) {
        unsafe {
            let gl = DoomGl::gl();
            gl.DetachShader(self.program, self.vs);
            gl.DetachShader(self.program, self.fs);
            gl.DeleteProgram(self.program);
            gl.DeleteShader(self.vs);
            gl.DeleteShader(self.fs);
        }
    }
}
