use std::cell::RefCell;

use crate::wad::file::WadFile;
use crate::wad::map::WadMap;
use crate::wad::textures::{Texture, Textures};

pub struct Content {
    pub textures: RefCell<Textures>,
    pub maps: Vec<WadMap>,
    pub file: WadFile,
}

impl Content {
    pub fn new(file: WadFile) -> Self {
        let mut content = Content {
            textures: RefCell::new(Textures::new(&file)),
            maps: Vec::new(),
            file,
        };
        content.maps.push(WadMap::new("E1M1", &content).unwrap());
        content
    }

    pub fn get_texture(&self, name: &str) -> Option<Texture> {
        if name.starts_with('-') {
            return None;
        }
        let mut real_name = name;
        if real_name.starts_with("F_SKY1") {
            real_name = "SKY1";
        }
        if let Some(t) = self.textures.borrow().list.get(&real_name.to_string()) {
            return Some(*t);
        }

        // If don't exists, try to load flat
        self.textures.borrow_mut().load_flat(&self.file, real_name);

        if let Some(t) = self.textures.borrow().list.get(&real_name.to_string()) {
            return Some(*t);
        }
        None
    }
}
