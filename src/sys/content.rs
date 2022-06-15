use super::textures::Textures;
use crate::wad::doom_textures::DoomTextures;
use crate::wad::file::WadFile;
use crate::wad::map::WadMap;

pub struct Content {
    //pub textures: RefCell<Textures>,
    pub maps: Vec<WadMap>,
    pub file: WadFile,
    textures: Textures,
}

impl Content {
    pub fn new(file: WadFile) -> Self {
        let doom_textures = DoomTextures::new(&file);

        let mut content = Content {
            maps: Vec::new(),
            file,
            textures: Textures::new(doom_textures),
        };
        content.maps.push(WadMap::new("E1M1", &content).unwrap());
        content
    }

    pub fn get_textures(&self) -> &Textures {
        &self.textures
    }
}
