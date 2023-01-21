use std::path::Path;

use super::textures::Textures;
use crate::wad::doom_textures::DoomTextures;
use crate::wad::file::WadFile;
use crate::wad::map::WadMap;

pub struct Content {
    map: Option<WadMap>,
    pub file: WadFile,
    textures: Textures,
}

impl Content {
    pub fn new(file_name: &Path) -> Self {
        let file = WadFile::new(file_name).expect("File not found");
        let doom_textures = DoomTextures::new(&file);

        Content {
            map: None,
            file,
            textures: Textures::new(doom_textures),
        }
    }

    pub fn load_map(&mut self, map_name: &str) {
        let map = WadMap::new(map_name, self);
        if let Ok(map) = map {
            self.map = Some(map);
        }
    }

    pub fn get_map(&self) -> &Option<WadMap> {
        &self.map
    }

    pub fn get_textures(&self) -> &Textures {
        &self.textures
    }
}
