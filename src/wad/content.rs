pub use super::file::WadFile;
use super::textures::Textures;

pub struct Content {
    pub textures: Textures,
}

impl Content {
    pub fn new(file: &WadFile) -> Self {
        Content {
            textures: Textures::new(file),
        }
    }
}
