use cgmath::{BaseNum, Matrix4, SquareMatrix};

use super::doom_gl::{gl, DoomGl};
use std::cell::RefCell;
use std::rc::Rc;

pub trait ToArr {
    type Output;
    fn to_arr(&self) -> Self::Output;
}

impl<T: BaseNum> ToArr for Matrix4<T> {
    type Output = [[T; 4]; 4];
    fn to_arr(&self) -> Self::Output {
        (*self).into()
    }
}

pub trait MaterialBindableParam {
    fn new(name: &'static str, material: &mut Material) -> Rc<RefCell<Self>>
    where
        Self: Sized;
    fn bind(&self);
}

pub struct MaterialParm<T> {
    id: i32,
    value: T,
}

impl<T> MaterialParm<T> {
    pub fn set_value(&mut self, value: T) {
        self.value = value;
    }
}

impl MaterialBindableParam for MaterialParm<Matrix4<f32>> {
    fn new(name: &'static str, material: &mut Material) -> Rc<RefCell<Self>>
    where
        Self: Sized,
    {
        let id = material.get_uniform_location(name);
        let result = Rc::new(RefCell::new(MaterialParm {
            id,
            value: Matrix4::identity(),
        }));

        material.register_parm(result.clone() as Rc<RefCell<dyn MaterialBindableParam>>);
        result
    }

    fn bind(&self) {
        let gl = DoomGl::gl();
        unsafe {
            gl.UniformMatrix4fv(
                self.id,
                1,
                gl::FALSE,
                self.value.to_arr().as_ptr() as *const _,
            );
            assert!(gl.GetError() == 0);
        }
    }
}

pub struct Material {
    vs: u32,
    fs: u32,
    program: u32,
    parms: Vec<Rc<RefCell<dyn MaterialBindableParam>>>,
}

pub fn create_shader(content: &str, shader_type: gl::types::GLenum) -> u32 {
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
        let vs = create_shader(vs, gl::VERTEX_SHADER);
        let fs = create_shader(fs, gl::FRAGMENT_SHADER);

        let gl = DoomGl::gl();
        let program = unsafe { gl.CreateProgram() };
        unsafe {
            gl.AttachShader(program, vs);
            gl.AttachShader(program, fs);
            gl.LinkProgram(program);
        }

        Material {
            vs,
            fs,
            program,
            parms: Vec::new(),
        }
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

    pub fn register_parm(&mut self, parm: Rc<RefCell<dyn MaterialBindableParam>>) {
        self.parms.push(parm);
    }

    pub fn bind(&self) {
        unsafe { DoomGl::gl().UseProgram(self.program) };

        // Bind all current params
        for parm in &self.parms {
            parm.borrow().bind();
        }
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
