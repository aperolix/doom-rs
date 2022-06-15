use std::collections::HashMap;

use crate::{render::doom_gl::DoomGl, wad::doom_textures::DoomTextures};

pub struct Texture {
    pub name: String,
    pub uv_start: (f32, f32),
    pub uv_end: (f32, f32),
    pub width: f32,
    pub height: f32,
    pub id: u32,
}

pub struct Textures {
    list: HashMap<String, Texture>,
}

impl Textures {
    pub fn new(doom_textures: DoomTextures) -> Self {
        let gl = DoomGl::get();

        // Convert doom textures to multiple textures of 1024x1024
        let mut textures = Vec::new();

        let mut buffer = Vec::new();
        buffer.resize(4 * 1024 * 1024, 0u8);
        textures.push(buffer);

        let mut texture_id = gl.gen_texture_id();

        let mut texture_table = HashMap::new();

        let mut tex = 0;
        let mut x = 0;
        let mut y = 0;
        let mut max_height = 0;
        for texture in doom_textures.list.iter() {
            if x + texture.width as i32 > 1024 {
                x = 0;
                y += max_height as i32;

                // Test if the texture is full
                if y + texture.height >= 1024 {
                    // Create the texture in the engine
                    gl.fill_texture_with_buffer(texture_id, 1024, 1024, textures[tex].as_slice());

                    // Gen the next texture
                    texture_id = gl.gen_texture_id();

                    tex += 1;
                    y = 0;

                    let mut buffer = Vec::new();
                    buffer.resize(4 * 1024 * 1024, 0u8);
                    textures.push(buffer);
                }
            }
            for origin_x in 0..texture.width {
                for origin_y in 0..texture.height {
                    let target_index = (((y + origin_y) * 1024 + (x + origin_x)) * 4) as usize;
                    let source_index = ((origin_y * texture.width + origin_x) * 4) as usize;
                    let buffer = textures.last_mut().unwrap();
                    buffer[target_index] = texture.buffer[source_index];
                    buffer[target_index + 1] = texture.buffer[source_index + 1];
                    buffer[target_index + 2] = texture.buffer[source_index + 2];
                    buffer[target_index + 3] = texture.buffer[source_index + 3];
                }
            }

            texture_table.insert(
                texture.name.clone(),
                Texture {
                    name: texture.name.clone(),
                    id: texture_id,
                    uv_start: (x as f32 / 1024.0, y as f32 / 1024.0),
                    uv_end: (
                        (x + texture.width) as f32 / 1024.0,
                        (y + texture.height) as f32 / 1024.0,
                    ),
                    width: texture.width as f32,
                    height: texture.height as f32,
                },
            );

            x += texture.width as i32;
            max_height = std::cmp::max(max_height, y + texture.height as i32);
        }

        // Create the texture in the engine
        gl.fill_texture_with_buffer(texture_id, 1024, 1024, textures[tex].as_slice());

        Textures {
            list: texture_table,
        }
    }

    pub fn find_texture(&self, name: &str) -> Option<&Texture> {
        self.list.get(name)
    }
}
