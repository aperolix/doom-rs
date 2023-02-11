use super::{
    file::WadFile,
    playpal::{Palette, PlayPal},
};

pub struct Patch {
    pub width: usize,
    pub height: usize,
    pub image: Vec<u8>,
    pub name: String,
}

pub struct Patches {
    patches: Vec<Patch>,
}

fn load_image(image: &[u8], pal: &Palette) -> Patch {
    #[repr(C, packed)]
    #[derive(Copy, Clone)]
    struct Header {
        width: i16,
        height: i16,
        left: i16,
        top: i16,
    }
    let (_, header, _) = unsafe { image[..].align_to::<Header>() };
    let size = header[0].width as usize * header[0].height as usize;
    let mut buffer = Vec::with_capacity(size);
    buffer.resize(size * 4, 0u8);

    let (_, columns, _) = unsafe { image[std::mem::size_of::<Header>()..].align_to::<i32>() };

    for (i, offset) in columns.iter().take(header[0].width as usize).enumerate() {
        let mut offset = *offset as usize;

        let mut rowstart = 0u8;
        while rowstart != 255 {
            rowstart = image[offset];
            offset += 1;

            if rowstart == 255 {
                break;
            }

            let pixel_count = image[offset] as usize;
            offset += 2; // skip one dummy byte
            let end_offset = offset + pixel_count;
            for (j, pixel) in image[offset..end_offset].iter().enumerate() {
                let color = &pal.colors[*pixel as usize];
                let index = ((j + rowstart as usize) * header[0].width as usize + i) * 4;
                buffer[index] = color.r;
                buffer[index + 1] = color.g;
                buffer[index + 2] = color.b;
                buffer[index + 3] = 255;
            }
            offset = end_offset + 1; // skip one dummy byte
        }
    }
    Patch {
        width: header[0].width as usize,
        height: header[0].height as usize,
        image: buffer,
        name: String::new(),
    }
}

impl Patches {
    pub fn new(file: &WadFile) -> Self {
        // Read palettes
        let playpal = PlayPal::new(file);

        let content = file.get_section("PNAMES");
        let (_, num_patches, _) = unsafe { content[0..4].align_to::<i32>() };

        let (_, body, _) = unsafe { content[4..].align_to::<[u8; 8]>() };

        let mut patches = Vec::new();
        for name in body.iter().take(num_patches[0] as usize) {
            let mut name = String::from_utf8(name.to_vec()).unwrap();
            name.make_ascii_uppercase();

            let image = file.get_section(name.as_str());
            let mut patch = load_image(image.as_slice(), &playpal.palettes[0]);
            patch.name = name;
            patches.push(patch);
        }

        Patches { patches }
    }

    pub fn get_patch(&self, index: usize) -> &Patch {
        &self.patches[index]
    }

    pub fn get_patch_by_name(&self, name: &String) -> Option<&Patch> {
        self.patches.iter().find(|&p| p.name.starts_with(name))
    }
}
