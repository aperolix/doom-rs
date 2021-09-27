use crate::wad::file::WadFile;
use crate::wad::map::WadMap;
use crate::wad::textures::{Texture, Textures};

pub struct Content {
    pub textures: Textures,
    pub maps: Vec<WadMap>,
}

impl Content {
    pub fn new(file: &mut WadFile) -> Self {
        let mut content = Content {
            textures: Textures::new(file),
            maps: Vec::new(),
        };
        content
            .maps
            .push(WadMap::new("E1M1", file, &content).unwrap());
        content
    }

    pub fn get_texture(&self, name: &str) -> Option<&Texture> {
        self.textures.list.get(&name.to_string())
    }
}
