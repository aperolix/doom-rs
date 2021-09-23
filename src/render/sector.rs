use cgmath::{BaseNum, Matrix4};

use super::{doom_gl::gl, material::Material};

pub struct SectorModel {
    wall_ibuffer: Vec<u16>,
    wall_material: Material,
    ib: u32,
    view_att: i32,
    persp_att: i32,
    pos_att: i32,
    uv_att: i32,
    light_att: i32,
    img_att: i32,
    texture: u32,
}

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

impl SectorModel {
    pub fn new(gl: &gl::Gl, ibuffer: Vec<u16>, texture: u32) -> Self {
        let wall_material = Material::new(gl, "./src/render/wall.vert", "./src/render/wall.frag");

        let mut ib = unsafe { std::mem::zeroed() };
        let pos_att = wall_material.get_attrib_location(gl, "position\0");
        let uv_att = wall_material.get_attrib_location(gl, "uv\0");
        let light_att = wall_material.get_attrib_location(gl, "light\0");
        let view_att = wall_material.get_uniform_location(gl, "view\0");
        let persp_att = wall_material.get_uniform_location(gl, "proj\0");
        let img_att = wall_material.get_uniform_location(gl, "image\0");
        unsafe {
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
            view_att,
            persp_att,
            pos_att,
            uv_att,
            light_att,
            img_att,
            texture,
        }
    }

    pub fn render(&self, view: &Matrix4<f32>, persp: &Matrix4<f32>, gl: &gl::Gl) {
        unsafe {
            self.wall_material.bind(gl);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ib);
            assert!(gl.GetError() == 0);
            gl.EnableVertexAttribArray(self.pos_att as gl::types::GLuint);
            assert!(gl.GetError() == 0);
            gl.EnableVertexAttribArray(self.uv_att as gl::types::GLuint);
            assert!(gl.GetError() == 0);
            gl.EnableVertexAttribArray(self.light_att as gl::types::GLuint);
            assert!(gl.GetError() == 0);
            gl.UniformMatrix4fv(
                self.view_att,
                1,
                gl::FALSE,
                view.to_arr().as_ptr() as *const _,
            );
            assert!(gl.GetError() == 0);
            gl.UniformMatrix4fv(
                self.persp_att,
                1,
                gl::FALSE,
                persp.to_arr().as_ptr() as *const _,
            );
            assert!(gl.GetError() == 0);
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
/*
impl<'a> Drop for SectorModel<'a> {
    fn drop(&mut self) {
        /*unsafe {
            self.gl.DeleteBuffers(1, &self.ib);
        }*/
    }
}
*/
