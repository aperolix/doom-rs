use super::file::WadFile;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct Palette {
    pub colors: [Color; 256],
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct PlayPal {
    pub palettes: [Palette; 14],
}

impl PlayPal {
    pub fn new(file: &WadFile) -> Self {
        let content = file.get_section("PLAYPAL");
        let (_, body, _) = unsafe { content[..].align_to::<PlayPal>() };
        body[0]
    }
}
