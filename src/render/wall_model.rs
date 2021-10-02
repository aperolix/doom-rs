use std::rc::Rc;

use cgmath::Matrix4;

use crate::render::{
    doom_gl::DoomGl,
    material::{MaterialValue, Stride},
};

use super::{
    doom_gl::gl,
    material::{Material, MaterialParam},
};

pub struct WallModel {
    ibuffer: Vec<u16>,
    material: Material,
    ib: u32,
    view_att: Rc<MaterialParam>,
    persp_att: Rc<MaterialParam>,
    vao: u32,
    img_att: Rc<MaterialParam>,
    texture: u32,
}

const WALL_FRAG_STR: &str = include_str!("wall.frag");
const WALL_VERT_STR: &str = include_str!("wall.vert");

impl WallModel {
    pub fn new(ibuffer: Vec<u16>, texture: u32) -> Self {
        let mut ib = unsafe { std::mem::zeroed() };
        let mut vao = unsafe { std::mem::zeroed() };

        unsafe {
            let gl = DoomGl::gl();
            // generate and bind the vao
            gl.GenVertexArrays(1, &mut vao);
            gl.BindVertexArray(vao);

            gl.GenBuffers(1, &mut ib);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ib);
            gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (ibuffer.len() * std::mem::size_of::<u16>()) as gl::types::GLsizeiptr,
                ibuffer.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );
            assert!(gl.GetError() == 0);
        }

        let mut material = Material::new(WALL_VERT_STR, WALL_FRAG_STR);
        let pos_att = MaterialParam::from_attrib("position\0", &mut material);
        let uv_att = MaterialParam::from_attrib("uv\0", &mut material);
        let light_att = MaterialParam::from_attrib("light\0", &mut material);
        let view_att = MaterialParam::from_uniform("view\0", &mut material);
        let persp_att = MaterialParam::from_uniform("proj\0", &mut material);
        let img_att = MaterialParam::from_uniform("image\0", &mut material);

        // Always bind stride after the buffer is bound
        pos_att.set_value(MaterialValue::FloatStride(Stride {
            count: 3,
            stride: 6 * std::mem::size_of::<f32>(),
            offset: 0,
        }));
        uv_att.set_value(MaterialValue::FloatStride(Stride {
            count: 2,
            stride: 6 * std::mem::size_of::<f32>(),
            offset: 3 * std::mem::size_of::<f32>(),
        }));
        light_att.set_value(MaterialValue::FloatStride(Stride {
            count: 1,
            stride: 6 * std::mem::size_of::<f32>(),
            offset: 5 * std::mem::size_of::<f32>(),
        }));

        WallModel {
            ibuffer,
            material,
            ib,
            vao,
            view_att,
            persp_att,
            img_att,
            texture,
        }
    }

    pub fn render(&self, view: &Matrix4<f32>, persp: &Matrix4<f32>) {
        self.view_att.set_value(MaterialValue::Matrix(*view));
        self.persp_att.set_value(MaterialValue::Matrix(*persp));
        self.img_att.set_value(MaterialValue::Int(0));

        let gl = DoomGl::gl();
        unsafe {
            gl.BindVertexArray(self.vao);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ib);
        }

        // Always bind material after buffer is bound
        self.material.bind();

        unsafe {
            gl.ActiveTexture(gl::TEXTURE0);
            gl.BindTexture(gl::TEXTURE_2D, self.texture);
            gl.DrawElements(
                gl::TRIANGLES,
                self.ibuffer.len() as i32,
                gl::UNSIGNED_SHORT,
                std::ptr::null(),
            );
            gl.BindVertexArray(0);
            assert!(gl.GetError() == 0);
        }
    }
}

impl Drop for WallModel {
    fn drop(&mut self) {
        unsafe {
            DoomGl::gl().DeleteBuffers(1, &self.ib);
            DoomGl::gl().DeleteVertexArrays(1, &self.vao);
        }
    }
}
