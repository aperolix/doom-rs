use std::collections::HashMap;

use super::{file::WadFile, patches::Patches};

pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub image: Vec<u8>,
}

pub struct Textures {
    pub list: HashMap<String, Texture>,
}

fn read_texture_section(
    file: &WadFile,
    section: &str,
    patches: &Patches,
) -> HashMap<String, Texture> {
    #[repr(C, packed)]
    //#[derive( Copy, Clone)]
    struct TextureInfo {
        name: [u8; 8],
        masked: i32,
        width: i16,
        height: i16,
        column_directory: i32,
        patch_count: i16,
    }

    #[repr(C, packed)]
    //#[derive(Debug, Copy, Clone)]
    struct PatchInfo {
        origin_x: i16,
        origin_y: i16,
        patch: i16,
        stepdir: i16,
        colormap: i16,
    }

    let mut result = HashMap::new();

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

            // Get the patch table
            let offset = offset + std::mem::size_of::<TextureInfo>();
            let (_, patch_info, _) = unsafe { section[offset..].align_to::<PatchInfo>() };

            // Compose texture
            let mut buffer = Vec::new();
            buffer.resize(
                4 * texture_info[0].width as usize * texture_info[0].height as usize,
                0u8,
            );
            for pinfo in patch_info.iter().take(texture_info[0].patch_count as usize) {
                let patch = patches.get_patch(pinfo.patch as usize);

                for x in 0..patch.width {
                    for y in 0..patch.height {
                        let real_x = pinfo.origin_x as i32 + x as i32;
                        let real_y =
                            texture_info[0].height as i32 - (pinfo.origin_y as i32 + y as i32) - 1;

                        let index = (real_y * texture_info[0].width as i32 + real_x) * 4;
                        if index >= 0 && index < buffer.len() as i32 {
                            let patch_index = (y as usize * patch.width + x as usize) * 4;

                            buffer[index as usize] = patch.image[patch_index];
                            buffer[index as usize + 1] = patch.image[patch_index + 1];
                            buffer[index as usize + 2] = patch.image[patch_index + 2];
                            buffer[index as usize + 3] = patch.image[patch_index + 3];
                        }
                    }
                }
            }

            result.insert(
                String::from_utf8(texture_info[0].name.to_ascii_uppercase().to_vec()).unwrap(),
                Texture {
                    width: texture_info[0].width as usize,
                    height: texture_info[0].height as usize,
                    image: buffer,
                },
            );
        }
    }

    result
}

impl Textures {
    pub fn new(file: &WadFile) -> Self {
        // First read the patches
        let patches = Patches::new(file);

        // Read the TEXTUREX
        let mut list = read_texture_section(file, "TEXTURE1", &patches);
        list.extend(read_texture_section(file, "TEXTURE2", &patches));

        Textures { list }
    }
}
