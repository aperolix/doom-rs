use std::rc::Rc;

use cgmath::Matrix4;
use kabal_render::opengl::{gl, OpenGl};

use crate::sys::textures::Texture;

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
    ceil_depth: u32,
    floor_depth: u32,

    height_att: Rc<MaterialParam>,
    light_att: Rc<MaterialParam>,
    persp_att: Rc<MaterialParam>,
    view_att: Rc<MaterialParam>,
    img_att: Rc<MaterialParam>,
    sky_att: Rc<MaterialParam>,
    depth_att: Rc<MaterialParam>,

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
        ceil_texture: Option<&Texture>,
        floor_texture: &Texture,
    ) -> Self {
        unsafe { OpenGl::gl().Disable(gl::CULL_FACE) };
        let mut material = Material::new(FLAT_VERT_STR, FLAT_FRAG_STR);

        let light_att = MaterialParam::from_uniform("light\0", &mut material);
        let height_att = MaterialParam::from_uniform("height\0", &mut material);
        let view_att = MaterialParam::from_uniform("view\0", &mut material);
        let persp_att = MaterialParam::from_uniform("proj\0", &mut material);
        let img_att = MaterialParam::from_uniform("image\0", &mut material);
        let sky_att = MaterialParam::from_uniform("sky\0", &mut material);
        let depth_att = MaterialParam::from_uniform("depth\0", &mut material);

        height_att.set_value(MaterialValue::Float(0.0));

        FlatModel {
            material,
            vbuffer,
            ibuffer,
            persp_att,
            view_att,
            img_att,
            sky_att,
            height_att,
            light_att,
            depth_att,
            light: 1.0,
            ceil: 64.0,
            floor: 0.0,
            vao: 0,
            ib: 0,
            vb: 0,
            ceil_texture: match ceil_texture {
                Some(t) => t.id,
                _ => u32::MAX,
            },
            ceil_depth: match ceil_texture {
                Some(t) => t.depth,
                _ => u32::MAX,
            },
            floor_texture: floor_texture.id,
            floor_depth: floor_texture.depth,
        }
    }

    pub fn init(&mut self) {
        let mut ib = unsafe { std::mem::zeroed() };
        let mut vao = unsafe { std::mem::zeroed() };
        let mut vb = unsafe { std::mem::zeroed() };

        unsafe {
            let gl = OpenGl::gl();
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
        self.sky_att.set_value(MaterialValue::Int(0));
        self.depth_att.set_value(MaterialValue::Float(0.0f32));

        let gl = OpenGl::gl();
        unsafe {
            OpenGl::gl().Disable(gl::CULL_FACE);

            gl.BindVertexArray(self.vao);
            gl.BindBuffer(gl::ARRAY_BUFFER, self.vb);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ib);
        }

        // Always bind material after buffer is bound
        self.material.bind();

        unsafe {
            // floor
            if self.floor_texture != u32::MAX {
                self.height_att.set_value(MaterialValue::Float(self.floor));
                self.height_att.bind();
                self.depth_att
                    .set_value(MaterialValue::Float(self.floor_depth as f32));
                self.depth_att.bind();

                gl.ActiveTexture(gl::TEXTURE0);
                gl.BindTexture(gl::TEXTURE_2D_ARRAY, self.floor_texture);
                gl.DrawElements(
                    gl::TRIANGLES,
                    self.ibuffer.len() as i32,
                    gl::UNSIGNED_SHORT,
                    std::ptr::null(),
                );
            }

            self.sky_att.set_value(MaterialValue::Int(0));
            self.material.bind();

            // Ceil
            if self.ceil_texture != u32::MAX {
                self.height_att.set_value(MaterialValue::Float(self.ceil));
                self.height_att.bind();
                self.depth_att
                    .set_value(MaterialValue::Float(self.ceil_depth as f32));
                self.depth_att.bind();

                gl.ActiveTexture(gl::TEXTURE0);
                gl.BindTexture(gl::TEXTURE_2D_ARRAY, self.ceil_texture);
                gl.DrawElements(
                    gl::TRIANGLES,
                    self.ibuffer.len() as i32,
                    gl::UNSIGNED_SHORT,
                    std::ptr::null(),
                );
            }

            gl.BindVertexArray(0);
            assert!(gl.GetError() == 0);
        }
    }
}
