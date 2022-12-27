use std::collections::HashMap;

use kabal_render::opengl::OpenGl;

use crate::wad::doom_textures::DoomTextures;

pub struct Texture {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub depth: u32,
    pub id: u32,
}

pub struct Textures {
    list: HashMap<String, Texture>,
}

impl Textures {
    pub fn new(doom_textures: DoomTextures) -> Self {
        let gl = OpenGl::get();

        // Convert doom textures to multiple textures of 1024x1024
        let mut textures = Vec::new();

        let mut texture_table = HashMap::new();

        let mut current_width = 0;
        let mut current_height = 0;
        let mut current_texture_id = 0;
        let mut current_depth = 0;
        for texture in doom_textures.list.iter() {
            // Check if we need a new texture
            if texture.width != current_width || texture.height != current_height {
                if current_height != 0 || current_width != 0 {
                    // Generate texture
                    gl.fill_texture_2d_array(
                        current_texture_id,
                        current_width,
                        current_height,
                        &textures,
                    );
                }
                current_depth = 0;
                current_height = texture.height;
                current_width = texture.width;
                textures.clear();
                current_texture_id = gl.gen_texture_id();
            }

            textures.push(texture.buffer.clone());

            let current_tile = Texture {
                name: texture.name.clone(),
                width: texture.width,
                height: texture.height,
                depth: current_depth,
                id: current_texture_id,
            };

            texture_table.insert(texture.name.clone(), current_tile);

            current_depth += 1;
        }

        // for the last ones
        gl.fill_texture_2d_array(current_texture_id, current_width, current_height, &textures);

        Textures {
            list: texture_table,
        }
    }

    pub fn find_texture(&self, name: &str) -> Option<&Texture> {
        self.list.get(name)
    }
}
