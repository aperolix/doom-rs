use std::str::FromStr;

use super::{file::WadFile, patches::Patches, playpal::PlayPal};

pub struct DoomTexture {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub buffer: Vec<u8>,
}

pub struct DoomTextures {
    pub list: Vec<DoomTexture>,
}

fn read_texture_section(file: &WadFile, section: &str, patches: &Patches) -> Vec<DoomTexture> {
    #[repr(C, packed)]
    struct TextureInfo {
        name: [u8; 8],
        masked: i32,
        width: i16,
        height: i16,
        column_directory: i32,
        patch_count: i16,
    }

    #[repr(C, packed)]
    struct PatchInfo {
        origin_x: i16,
        origin_y: i16,
        patch: i16,
        stepdir: i16,
        colormap: i16,
    }

    let mut result = Vec::new();

    if let Some(section) = file.get_section(section) {
        // Get count
        let (_, count, _) = unsafe { section[..4].align_to::<i32>() };
        let count = count[0] as usize;

        // Get offset table
        let (_, offsets, _) = unsafe { section[4..].align_to::<i32>() };

        for offset in offsets.iter().take(count) {
            let offset = *offset as usize;

            // Get the texture header
            let (_, texture_info, _) = unsafe { section[offset..].align_to::<TextureInfo>() };

            let width = texture_info[0].width as usize;
            let height = texture_info[0].height as usize;

            // Get the patch table
            let offset = offset + std::mem::size_of::<TextureInfo>();
            let (_, patch_info, _) = unsafe { section[offset..].align_to::<PatchInfo>() };

            // Compose texture
            let mut buffer = Vec::new();
            buffer.resize(4 * width * height, 1u8);
            for pinfo in patch_info.iter().take(texture_info[0].patch_count as usize) {
                let patch = patches.get_patch(pinfo.patch as usize);

                for x in 0..patch.width {
                    for y in 0..patch.height {
                        let real_x = pinfo.origin_x as i32 + x as i32;
                        let real_y =
                            texture_info[0].height as i32 - (pinfo.origin_y as i32 + y as i32) - 1;

                        let index = (real_y * texture_info[0].width as i32 + real_x) * 4;
                        if index >= 0 && index < buffer.len() as i32 {
                            let patch_index = (y * patch.width + x) * 4;

                            let dest_alpha = buffer[index as usize + 3];
                            let src_alpha = patch.image[patch_index + 3];

                            if src_alpha == 0 {
                                // Don't rewrite above existing color
                                if dest_alpha <= 1u8 {
                                    buffer[index as usize + 3] = src_alpha;
                                }
                            } else {
                                buffer[index as usize] = patch.image[patch_index];
                                buffer[index as usize + 1] = patch.image[patch_index + 1];
                                buffer[index as usize + 2] = patch.image[patch_index + 2];
                                buffer[index as usize + 3] = 255u8;
                            }
                        }
                    }
                }
            }

            let name =
                String::from_utf8(texture_info[0].name.to_ascii_uppercase().to_vec()).unwrap();

            result.push(DoomTexture {
                name,
                width: width as i32,
                height: height as i32,
                buffer,
            });
        }
    }

    result
}

fn read_sky(patches: &Patches) -> Vec<DoomTexture> {
    let mut sky_num = 1;
    let mut result = Vec::new();
    loop {
        let sky_name = format!("SKY{sky_num}");
        if let Some(p) = patches.get_patch_by_name(&sky_name) {
            // Compose texture
            let mut buffer = Vec::new();
            buffer.resize(4 * p.width * p.height, 1u8);

            for x in 0..p.width {
                for y in 0..p.height {
                    let index = (y * p.width + x) * 4;

                    buffer[index] = p.image[index];
                    buffer[index + 1] = p.image[index + 1];
                    buffer[index + 2] = p.image[index + 2];
                    buffer[index + 3] = 255u8;
                }
            }
            result.push(DoomTexture {
                name: sky_name,
                width: p.width as i32,
                height: p.height as i32,
                buffer,
            });
        } else {
            break;
        }
        sky_num += 1;
    }
    result
}

pub fn load_flat(file: &WadFile, name: &str) -> DoomTexture {
    let section = file.get_section(name).unwrap();

    // Read palettes
    let playpal = PlayPal::new(file);

    let mut buffer = Vec::new();
    buffer.resize(4 * 64 * 64_usize, 0u8);

    for (i, pixel) in section.iter().enumerate() {
        let color = &playpal.palettes[0].colors[*pixel as usize];

        buffer[i * 4] = color.r;
        buffer[i * 4 + 1] = color.g;
        buffer[i * 4 + 2] = color.b;
        buffer[i * 4 + 3] = 255;
    }

    DoomTexture {
        name: String::from_str(name).unwrap(),
        width: 64,
        height: 64,
        buffer,
    }
}

fn read_flats(file: &WadFile) -> Vec<DoomTexture> {
    let start_index = file.directory.get_lump_index("F_START").unwrap();
    let end_index = file.directory.get_lump_index("F_END").unwrap();
    let mut textures = Vec::new();

    for i in start_index..end_index {
        let lump = file.directory.get_lump(i);
        if lump.size != 0 {
            let name = lump.name.to_ascii_uppercase();
            let name = String::from_utf8(name.to_vec()).unwrap();
            textures.push(load_flat(file, name.as_str()));
        }
    }

    textures
}

impl DoomTextures {
    pub fn new(file: &WadFile) -> Self {
        // First read the patches
        let patches = Patches::new(file);

        // Read the TEXTUREX
        let mut list = read_texture_section(file, "TEXTURE1", &patches);
        list.extend(read_texture_section(file, "TEXTURE2", &patches));
        list.extend(read_sky(&patches));
        list.extend(read_flats(file));

        list.sort_by(|a, b| a.height.cmp(&b.height).then(a.width.cmp(&b.width)));

        DoomTextures { list }
    }
}
