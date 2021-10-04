use std::rc::Rc;

use cgmath::Matrix4;

use crate::render::doom_gl::{gl, DoomGl};

use super::material::{Material, MaterialParam, MaterialValue, Stride};

pub struct FlatModel {
    vbuffer: Vec<f32>,
    ibuffer: Vec<u16>,
    pub light: f32,
    pub ceil: f32,
    pub floor: f32,
    material: Material,
    ceil_texture: u32,
    floor_texture: u32,

    height_att: Rc<MaterialParam>,
    light_att: Rc<MaterialParam>,
    persp_att: Rc<MaterialParam>,
    view_att: Rc<MaterialParam>,
    img_att: Rc<MaterialParam>,

    vao: u32,
    ib: u32,
    vb: u32,
}

const FLAT_FRAG_STR: &str = include_str!("flat.frag");
const FLAT_VERT_STR: &str = include_str!("flat.vert");

impl FlatModel {
    pub fn new(
        vbuffer: Vec<f32>,
        ibuffer: Vec<u16>,
        ceil_texture: u32,
        floor_texture: u32,
    ) -> Self {
        unsafe { DoomGl::gl().Disable(gl::CULL_FACE) };
        let mut material = Material::new(FLAT_VERT_STR, FLAT_FRAG_STR);

        let light_att = MaterialParam::from_uniform("light\0", &mut material);
        let height_att = MaterialParam::from_uniform("height\0", &mut material);
        let view_att = MaterialParam::from_uniform("view\0", &mut material);
        let persp_att = MaterialParam::from_uniform("proj\0", &mut material);
        let img_att = MaterialParam::from_uniform("image\0", &mut material);

        height_att.set_value(MaterialValue::Float(0.0));

        FlatModel {
            material,
            vbuffer,
            ibuffer,
            persp_att,
            view_att,
            img_att,
            height_att,
            light_att,
            light: 1.0,
            ceil: 64.0,
            floor: 0.0,
            vao: 0,
            ib: 0,
            vb: 0,
            ceil_texture,
            floor_texture,
        }
    }

    pub fn init(&mut self) {
        let mut ib = unsafe { std::mem::zeroed() };
        let mut vao = unsafe { std::mem::zeroed() };
        let mut vb = unsafe { std::mem::zeroed() };

        unsafe {
            let gl = DoomGl::gl();
            gl.GenBuffers(1, &mut vb);
            gl.BindBuffer(gl::ARRAY_BUFFER, vb);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (self.vbuffer.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                self.vbuffer.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );

            // generate and bind the vao
            gl.GenVertexArrays(1, &mut vao);
            gl.BindVertexArray(vao);

            gl.GenBuffers(1, &mut ib);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ib);
            gl.BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.ibuffer.len() * std::mem::size_of::<u16>()) as gl::types::GLsizeiptr,
                self.ibuffer.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );
            self.vao = vao;
            self.ib = ib;

            assert!(gl.GetError() == 0);
        }

        let pos_att = MaterialParam::from_attrib("position\0", &mut self.material);

        // Always bind stride after the buffer is bound
        pos_att.set_value(MaterialValue::FloatStride(Stride {
            count: 2,
            stride: 2 * std::mem::size_of::<f32>(),
            offset: 0,
        }));
    }

    pub fn render(&self, view: &Matrix4<f32>, persp: &Matrix4<f32>) {
        self.view_att.set_value(MaterialValue::Matrix(*view));
        self.persp_att.set_value(MaterialValue::Matrix(*persp));
        self.img_att.set_value(MaterialValue::Int(0));
        self.light_att.set_value(MaterialValue::Float(self.light));

        let gl = DoomGl::gl();
        unsafe {
            DoomGl::gl().Disable(gl::CULL_FACE);

            gl.BindVertexArray(self.vao);
            gl.BindBuffer(gl::ARRAY_BUFFER, self.vb);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ib);
        }

        // Always bind material after buffer is bound
        self.material.bind();

        unsafe {
            // floor
            self.height_att.set_value(MaterialValue::Float(self.floor));
            self.height_att.bind();

            gl.ActiveTexture(gl::TEXTURE0);
            gl.BindTexture(gl::TEXTURE_2D, self.floor_texture);
            gl.DrawElements(
                gl::TRIANGLES,
                self.ibuffer.len() as i32,
                gl::UNSIGNED_SHORT,
                std::ptr::null(),
            );

            // Ceil
            self.height_att.set_value(MaterialValue::Float(self.ceil));
            self.height_att.bind();

            gl.ActiveTexture(gl::TEXTURE0);
            gl.BindTexture(gl::TEXTURE_2D, self.ceil_texture);
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
