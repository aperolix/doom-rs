use crate::render::doom_gl::DoomGl;
use crate::wad::file::WadFile;
use crate::wad::textures::{Texture, Textures};

pub struct Content {
    pub textures: Textures,
}

impl Content {
    pub fn new(file: &WadFile, gl: &DoomGl) -> Self {
        Content {
            textures: Textures::new(file, gl),
        }
    }

    pub fn get_texture(&self, name: &str) -> Option<&Texture> {
        self.textures.list.get(&name.to_string())
    }
}
