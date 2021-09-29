use std::{cell::RefCell, rc::Rc};

use cgmath::Matrix4;

use crate::render::doom_gl::DoomGl;

use super::{
    doom_gl::gl,
    material::{Material, MaterialBindableParam, MaterialParm},
};

pub struct SectorModel {
    wall_ibuffer: Vec<u16>,
    wall_material: Material,
    ib: u32,
    view_att: Rc<RefCell<MaterialParm<Matrix4<f32>>>>,
    persp_att: Rc<RefCell<MaterialParm<Matrix4<f32>>>>,
    vao: u32,
    pos_att: i32,
    uv_att: i32,
    light_att: i32,
    img_att: i32,
    texture: u32,
}

impl SectorModel {
    pub fn new(ibuffer: Vec<u16>, texture: u32) -> Self {
        let mut wall_material = Material::new("./src/render/wall.vert", "./src/render/wall.frag");

        let mut ib = unsafe { std::mem::zeroed() };
        let mut vao = unsafe { std::mem::zeroed() };
        let pos_att = wall_material.get_attrib_location("position\0");
        let uv_att = wall_material.get_attrib_location("uv\0");
        let light_att = wall_material.get_attrib_location("light\0");
        let view_att = MaterialBindableParam::new("view\0", &mut wall_material);
        let persp_att = MaterialBindableParam::new("proj\0", &mut wall_material);
        let img_att = wall_material.get_uniform_location("image\0");
        unsafe {
            let gl = DoomGl::gl();
            // generate and bind the vao
            gl.GenVertexArrays(1, &mut vao);
            gl.BindVertexArray(vao);

            gl.GenBuffers(1, &mut ib);
            assert!(gl.GetError() == 0);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ib);
            assert!(gl.GetError() == 0);
            gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (ibuffer.len() * std::mem::size_of::<u16>()) as gl::types::GLsizeiptr,
                ibuffer.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );
            assert!(gl.GetError() == 0);
            gl.VertexAttribPointer(
                pos_att as gl::types::GLuint,
                3,
                gl::FLOAT,
                0,
                (6 * std::mem::size_of::<f32>()) as gl::types::GLsizei,
                std::ptr::null(),
            );
            assert!(gl.GetError() == 0);
            gl.VertexAttribPointer(
                uv_att as gl::types::GLuint,
                2,
                gl::FLOAT,
                0,
                (6 * std::mem::size_of::<f32>()) as gl::types::GLsizei,
                (3 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            assert!(gl.GetError() == 0);
            gl.VertexAttribPointer(
                light_att as gl::types::GLuint,
                1,
                gl::FLOAT,
                0,
                (6 * std::mem::size_of::<f32>()) as gl::types::GLsizei,
                (5 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            assert!(gl.GetError() == 0);
        }

        SectorModel {
            wall_ibuffer: ibuffer,
            wall_material,
            ib,
            vao,
            view_att,
            persp_att,
            pos_att,
            uv_att,
            light_att,
            img_att,
            texture,
        }
    }

    pub fn render(&self, view: &Matrix4<f32>, persp: &Matrix4<f32>) {
        unsafe {
            let gl = DoomGl::gl();
            self.wall_material.bind();
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ib);
            gl.BindVertexArray(self.vao);
            assert!(gl.GetError() == 0);
            gl.EnableVertexAttribArray(self.pos_att as gl::types::GLuint);
            assert!(gl.GetError() == 0);
            gl.EnableVertexAttribArray(self.uv_att as gl::types::GLuint);
            assert!(gl.GetError() == 0);
            gl.EnableVertexAttribArray(self.light_att as gl::types::GLuint);
            assert!(gl.GetError() == 0);
            self.view_att.borrow_mut().set_value(*view);
            self.persp_att.borrow_mut().set_value(*persp);
            gl.Uniform1i(self.img_att, 0);
            assert!(gl.GetError() == 0);
            gl.ActiveTexture(gl::TEXTURE0);
            assert!(gl.GetError() == 0);
            gl.BindTexture(gl::TEXTURE_2D, self.texture);
            assert!(gl.GetError() == 0);
            gl.DrawElements(
                gl::TRIANGLES,
                self.wall_ibuffer.len() as i32,
                gl::UNSIGNED_SHORT,
                std::ptr::null(),
            );
            assert!(gl.GetError() == 0);
        }
    }
}

impl Drop for SectorModel {
    fn drop(&mut self) {
        unsafe {
            DoomGl::gl().DeleteBuffers(1, &self.ib);
            DoomGl::gl().DeleteVertexArrays(1, &self.vao);
        }
    }
}
