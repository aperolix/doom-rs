use std::collections::HashMap;

use super::doom_gl::gl;

pub struct Texture {
    pub id: u32,
    pub width: i32,
    pub height: i32,
}

pub struct Textures {
    pub textures: HashMap<String, Texture>,
}

impl Textures {
    pub fn new() -> Self {
        Textures {
            textures: HashMap::new(),
        }
    }
    pub fn load_texture(&mut self, gl: &gl::Gl, name: &str, image: &[u8], width: i32, height: i32) {
        let mut id = [0u32; 1];
        unsafe {
            gl.GenTextures(1, id.as_mut_ptr());
            assert!(gl.GetError() == 0);
            gl.BindTexture(gl::TEXTURE_2D, id[0]);
            assert!(gl.GetError() == 0);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            assert!(gl.GetError() == 0);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            assert!(gl.GetError() == 0);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            assert!(gl.GetError() == 0);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            assert!(gl.GetError() == 0);
            gl.TexImage2D(
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
            assert!(gl.GetError() == 0);
            gl.GenerateMipmap(gl::TEXTURE_2D);
        }

        self.textures.insert(
            name.to_string(),
            Texture {
                id: id[0],
                width,
                height,
            },
        );
    }
}
