use crate::render::doom_gl::gl;
use crate::render::textures::Textures;
use crate::wad::info::WadInfo;

use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::mem;
use std::{fs::File, path::Path};

use super::directory::WadDirectory;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct MapTexture {
    name: [u8; 8],
    masked: i32,
    width: i16,
    height: i16,
    column_directory: i32,
    patch_count: i16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct MapPatch {
    origin_x: i16,
    origin_y: i16,
    patch: i16,
    stepdir: i16,
    colormap: i16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct PatchHeader {
    width: i16,
    height: i16,
    left: i16,
    top: i16,
}

struct TexturePatch {
    header: PatchHeader,
    image: Vec<u8>,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct PalColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct PlayPal {
    pub colors: [PalColor; 256],
}

pub struct WadFile {
    pub directory: WadDirectory,
    pub reader: BufReader<File>,
    patches: Vec<TexturePatch>,
    pub textures: Textures,
    playpal: [PlayPal; 14],
}

impl WadFile {
    pub fn new(file_name: &Path) -> Result<Self, String> {
        let file = match File::open(file_name) {
            Ok(file) => file,
            Err(e) => return Err(e.to_string()),
        };

        let mut reader = io::BufReader::new(file);

        let info = WadInfo::from_reader(&mut reader)?;
        let directory = WadDirectory::new(&info, &mut reader)?;

        Ok(WadFile {
            directory,
            reader,
            patches: Vec::new(),
            textures: Textures::new(),
            playpal: [PlayPal {
                colors: [PalColor {
                    r: 0u8,
                    g: 0u8,
                    b: 0u8,
                }; 256],
            }; 14],
        })
    }

    pub fn read_section<T: Copy>(&mut self, mapidx: usize, name: &str) -> Vec<T> {
        let index = self.directory.find_section(name, mapidx).unwrap();
        let lump = &self.directory.files[index];
        self.reader
            .seek(SeekFrom::Start(lump.file_pos as u64))
            .unwrap();

        let count = lump.size as usize / mem::size_of::<T>();

        let mut result = Vec::new();
        let mut buffer = Vec::new();
        buffer.resize(mem::size_of::<T>(), 0u8);
        for _ in 0..count {
            if self.reader.read_exact(&mut buffer).is_ok() {
                let (_, body, _) = unsafe { buffer.align_to::<T>() };
                result.push(body[0]);
            }
        }
        result
    }

    pub fn read_playpal(&mut self) {
        let index = self.directory.find_section("PLAYPAL", 0).unwrap();
        let lump = &self.directory.files[index];

        self.reader
            .seek(SeekFrom::Start(lump.file_pos as u64))
            .unwrap();

        let mut buffer = Vec::new();
        buffer.resize(mem::size_of::<PlayPal>() * 14, 0u8);
        if self.reader.read_exact(&mut buffer).is_ok() {
            let (_, playpal, _) = unsafe { buffer.align_to::<PlayPal>() };
            self.playpal.copy_from_slice(playpal);
        }
    }

    pub fn read_pnames(&mut self) {
        let index = self.directory.find_section("PNAMES", 0).unwrap();
        let lump = &self.directory.files[index];

        self.reader
            .seek(SeekFrom::Start(lump.file_pos as u64))
            .unwrap();

        let mut buffer = Vec::new();
        buffer.resize(mem::size_of::<i32>(), 0u8);
        if self.reader.read_exact(&mut buffer).is_ok() {
            let (_, count, _) = unsafe { buffer.align_to::<i32>() };

            let mut buffer = Vec::new();
            buffer.resize(mem::size_of::<[u8; 8]>() * count[0] as usize, 0u8);

            if self.reader.read_exact(&mut buffer).is_ok() {
                let (_, names, _) = unsafe { buffer.align_to::<[u8; 8]>() };

                for n in names {
                    let name = String::from_utf8(n.to_vec()).unwrap();

                    self.read_image(name.as_str());
                }
            }
        }
    }

    fn read_image(&mut self, name: &str) {
        let index = self
            .directory
            .find_section(name.to_uppercase().as_str(), 0)
            .unwrap();
        let lump = &self.directory.files[index];

        self.reader
            .seek(SeekFrom::Start(lump.file_pos as u64))
            .unwrap();

        let mut buffer = Vec::new();
        buffer.resize(mem::size_of::<PatchHeader>(), 0u8);

        let mut image = Vec::new();

        if self.reader.read_exact(&mut buffer).is_ok() {
            let (_, header, _) = unsafe { buffer.align_to::<PatchHeader>() };

            image.resize(header[0].width as usize * header[0].height as usize, 0u8);

            let mut buffer = Vec::new();
            buffer.resize(mem::size_of::<i32>() * header[0].width as usize, 0u8);
            if self.reader.read_exact(&mut buffer).is_ok() {
                let (_, columns, _) = unsafe { buffer.align_to::<i32>() };

                for i in 0..header[0].width {
                    self.reader
                        .seek(SeekFrom::Start(
                            lump.file_pos as u64 + columns[i as usize] as u64,
                        ))
                        .unwrap();

                    let mut rowstart = 0u8;
                    while rowstart != 255 {
                        let mut one_byte: [u8; 1] = [0u8];
                        self.reader.read_exact(&mut one_byte).unwrap();
                        rowstart = one_byte[0];

                        if rowstart == 255 {
                            break;
                        }

                        self.reader.read_exact(&mut one_byte).unwrap();
                        let pixel_count = one_byte[0] as i16;
                        self.reader.read_exact(&mut one_byte).unwrap();

                        for j in 0..pixel_count {
                            self.reader.read_exact(&mut one_byte).unwrap();
                            let pixel = one_byte[0];

                            image[(j as usize + rowstart as usize) * header[0].width as usize
                                + i as usize] = pixel;
                        }
                        self.reader.read_exact(&mut one_byte).unwrap();
                    }
                }
            }

            let patch = TexturePatch {
                header: header[0],
                image,
            };

            self.patches.push(patch);
        }
    }

    pub fn read_textures(&mut self, lump: &str, gl: &gl::Gl) {
        let index = self.directory.find_section(lump, 0).unwrap();
        let lump = &self.directory.files[index];

        self.reader
            .seek(SeekFrom::Start(lump.file_pos as u64))
            .unwrap();

        let mut buffer = Vec::new();
        buffer.resize(mem::size_of::<u32>(), 0u8);
        if self.reader.read_exact(&mut buffer).is_ok() {
            let (_, count, _) = unsafe { buffer.align_to::<i32>() };

            let mut buffer = Vec::new();
            buffer.resize(4 * count[0] as usize, 0u8);
            if self.reader.read_exact(&mut buffer).is_ok() {
                let (_, offsets, _) = unsafe { buffer.align_to::<i32>() };

                for offset in offsets {
                    self.reader
                        .seek(SeekFrom::Start((lump.file_pos + offset) as u64))
                        .unwrap();

                    let mut buffer = Vec::new();
                    buffer.resize(mem::size_of::<MapTexture>() as usize, 0u8);
                    if self.reader.read_exact(&mut buffer).is_ok() {
                        let (_, map_texture, _) = unsafe { buffer.align_to::<MapTexture>() };

                        let mut buffer = Vec::new();
                        buffer.resize(
                            mem::size_of::<MapPatch>() * map_texture[0].patch_count as usize,
                            0u8,
                        );
                        if self.reader.read_exact(&mut buffer).is_ok() {
                            let (_, patches, _) = unsafe { buffer.align_to::<MapPatch>() };

                            let mut texture_buffer = Vec::new();
                            let texture_dimension =
                                (map_texture[0].width as i32, map_texture[0].height as i32);

                            texture_buffer.resize(
                                4 * (texture_dimension.0 * texture_dimension.1) as usize,
                                0u8,
                            );

                            for p in patches {
                                let patch = &self.patches[p.patch as usize];
                                let patch_width = patch.header.width;
                                let patch_height = patch.header.height;

                                for x in 0..patch_width {
                                    for y in 0..patch_height {
                                        let real_x = p.origin_x as i32 + x as i32;
                                        let real_y = texture_dimension.1
                                            - (p.origin_y as i32 + y as i32)
                                            - 1;

                                        let index = (real_y * texture_dimension.0 + real_x) * 4;
                                        if index >= 0 && index < texture_buffer.len() as i32 {
                                            let palette_idx = patch.image
                                                [y as usize * patch_width as usize + x as usize];

                                            /*if palette_idx == 247 {
                                                texture_buffer[index as usize] = 0u8;
                                                texture_buffer[index as usize + 1] = 0u8;
                                                texture_buffer[index as usize + 2] = 0u8;
                                                texture_buffer[index as usize + 3] = 0u8;
                                            } else {*/
                                            let color =
                                                self.playpal[0_usize].colors[palette_idx as usize];

                                            texture_buffer[index as usize] = color.r;
                                            texture_buffer[index as usize + 1] = color.g;
                                            texture_buffer[index as usize + 2] = color.b;
                                            texture_buffer[index as usize + 3] = 255u8;
                                            //}
                                        }
                                    }
                                }
                            }

                            self.textures.load_texture(
                                gl,
                                &String::from_utf8(map_texture[0].name.to_vec()).unwrap(),
                                &texture_buffer,
                                texture_dimension.0,
                                texture_dimension.1,
                            )
                        }
                    }
                }
            }
        }
    }
}
