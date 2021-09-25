use core::str;
use std::collections::HashMap;

use crate::{
    camera::Camera,
    render::{
        doom_gl::{gl, GVertex},
        sector::SectorModel,
        textures::{Texture, Textures},
    },
};

use super::file::WadFile;

use bitflags::bitflags;
use cgmath::{InnerSpace, Matrix4, Vector2, Vector3};

bitflags! {
    struct LinedefFlags: i16 {
        const NONE = 0;
        const BLOCK_ALL = 0x0001;
        const BLOCK_MONSTERS = 0x0002;
        const TWO_SIDED = 0x0004;
        const UPPER_TEX_UNPEGGED = 0x0008;
        const LOWER_TEX_UNPEGGED = 0x0010;
        const SECRET = 0x0020;
        const BLOCK_SOUND = 0x0040;
        const AUTO_MAP_NEVER = 0x0080;
        const AUTO_MAP_ALWAYS = 0x0100;
    }
}

#[repr(i16)]
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
enum SectorType {
    Normal = 0,
    BlinkRandom = 1,
    BlinkHalf = 2,
    BlinkFull = 3,
    DamageBlink = 4,
    MediumDamage = 5,
    LightDamage = 7,
    Oscillate = 8,
    Secret = 9,
    CeilDoorOpen = 10,
    KillEnd = 11,
    SyncBlinkHalf = 12,
    SyncBlinkFull = 13,
    CeilDoorClose = 14,
    HeavyDamage = 16,
    Flicker = 17,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct LineDef {
    start_vertex: u16,
    end_vertex: u16,
    flags: LinedefFlags,
    special_type: i16,
    sector_tag: i16,
    front_sidedef: i16,
    back_sidedef: i16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct SideDef {
    x_offset: i16,
    y_offset: i16,
    upper_tex: [u8; 8],
    lower_tex: [u8; 8],
    middle_tex: [u8; 8],
    sector: i16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    x: i16,
    y: i16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct Segment {
    start_vertex: i16,
    end_vertex: i16,
    angle: i16,
    linedef: i16,
    direction: i16,
    offset: i16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct Sector {
    floor: i16,
    ceiling: i16,
    floor_tex: [u8; 8],
    ceil_tex: [u8; 8],
    lighting: i16,
    stype: SectorType,
    tag: i16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct SubSector {
    seg_count: i16,
    first_seg: i16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct BoundingBox {
    top: i16,
    bottom: i16,
    left: i16,
    right: i16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct Nodes {
    partition_x: i16,
    partition_y: i16,
    change_x: i16,
    change_y: i16,
    right_bbox: BoundingBox,
    left_bbox: BoundingBox,
    right_child: i16,
    left_child: i16,
}

#[allow(dead_code)]
pub struct WadMap {
    linedefs: Vec<LineDef>,
    sidedefs: Vec<SideDef>,
    vertexes: Vec<Vertex>,
    segs: Vec<Segment>,
    sectors: Vec<Sector>,
    ssectors: Vec<SubSector>,
    nodes: Vec<Nodes>,

    vbuffer: Vec<GVertex>,
    wall_ibuffer: Vec<Vec<u16>>,

    model_sectors: Vec<SectorModel>,

    textures: Textures,

    gl: gl::Gl,
}

pub struct PerMaterial {
    pub ibuffer: Vec<u16>,
}

impl WadMap {
    fn add_quad(
        &self,
        vbuffer: &mut Vec<GVertex>,
        material: &mut PerMaterial,
        line: (u16, u16),
        heights: (f32, f32),
        texture_size: (i32, i32),
        texture_offset: (i16, i16),
        light: f32,
    ) {
        let start = self.vertexes[line.0 as usize];
        let end = self.vertexes[line.1 as usize];

        let line = Vector3::new(end.x as f32, end.y as f32, 0.0f32)
            - Vector3::new(start.x as f32, start.y as f32, 0.0f32);
        let length = line.magnitude();

        let uv_offset = (
            texture_offset.0 as f32 / texture_size.0 as f32,
            texture_offset.1 as f32 / texture_size.1 as f32,
        );

        let mut quad_buffer = vec![
            GVertex {
                pos: Vector3::new(-start.x as f32, heights.0, start.y as f32),
                uv: Vector2::new(uv_offset.0, uv_offset.1),
                light,
            },
            GVertex {
                pos: Vector3::new(-end.x as f32, heights.0, end.y as f32),
                uv: Vector2::new(
                    length / texture_size.0 as f32 + uv_offset.0,
                    0.0f32 + uv_offset.1,
                ),
                light,
            },
            GVertex {
                pos: Vector3::new(-start.x as f32, heights.1, start.y as f32),
                uv: Vector2::new(
                    uv_offset.0,
                    (heights.1 - heights.0) / texture_size.1 as f32 + uv_offset.1,
                ),
                light,
            },
            GVertex {
                pos: Vector3::new(-end.x as f32, heights.1, end.y as f32),
                uv: Vector2::new(
                    length / texture_size.0 as f32 + uv_offset.0,
                    (heights.1 - heights.0) / texture_size.1 as f32 + uv_offset.1,
                ),
                light,
            },
        ];

        let startidx = vbuffer.len() as u16;

        vbuffer.append(&mut quad_buffer);
        material.ibuffer.append(&mut vec![
            startidx,
            startidx + 1,
            startidx + 2,
            startidx + 2,
            startidx + 1,
            startidx + 3,
        ]);
    }

    fn get_texture(&self, texture_name: [u8; 8]) -> Option<&Texture> {
        if texture_name[0] != b'-' {
            if let Some(texture) = self
                .textures
                .textures
                .get(&String::from_utf8(texture_name.to_ascii_uppercase().to_vec()).unwrap())
            {
                return Some(texture);
            }
        }
        None
    }

    fn prepare_line_render(
        &self,
        per_material: &mut HashMap<u32, PerMaterial>,
        vbuffer: &mut Vec<GVertex>,
        texture: &Texture,
        line: (u16, u16),
        heights: (f32, f32),
        texture_offset: (i16, i16),
        light: f32,
    ) {
        let material = if let Some(m) = per_material.get_mut(&texture.id) {
            m
        } else {
            per_material.insert(
                texture.id,
                PerMaterial {
                    ibuffer: Vec::new(),
                },
            );
            per_material.get_mut(&texture.id).unwrap()
        };

        // push vertices
        self.add_quad(
            vbuffer,
            material,
            line,
            heights,
            (texture.width, texture.height),
            texture_offset,
            light,
        );
    }

    pub fn prepare_render(&self) -> (HashMap<u32, PerMaterial>, Vec<GVertex>) {
        let mut vbuffer = Vec::new();

        let mut per_material = HashMap::new();

        // Create walls buffers
        for l in &self.linedefs {
            // front
            assert!(l.front_sidedef != -1);

            let front_side = self.sidedefs.get(l.front_sidedef as usize).unwrap();
            let front_sector = self.sectors.get(front_side.sector as usize).unwrap();

            let (back_side, back_sector) = if l.back_sidedef != -1 {
                let side = self.sidedefs.get(l.back_sidedef as usize).unwrap();
                let sector = self.sectors.get(side.sector as usize).unwrap();
                (Some(side), Some(sector))
            } else {
                (None, None)
            };

            let front_floor = front_sector.floor as f32;
            let front_ceil = front_sector.ceiling as f32;
            let front_light = front_sector.lighting as f32 / 255.0;
            let back_floor = if let Some(s) = back_sector {
                s.floor as f32
            } else {
                front_floor
            };
            let back_ceil = if let Some(s) = back_sector {
                s.ceiling as f32
            } else {
                front_ceil
            };
            let back_light = if let Some(s) = back_sector {
                s.lighting as f32 / 255.0
            } else {
                0.0
            };
            let mut wall_extent = (back_floor, back_ceil);
            if front_floor < back_floor {
                wall_extent.0 = front_floor;
            }
            if front_ceil > back_ceil {
                wall_extent.1 = front_ceil;
            }

            let line = (l.start_vertex, l.end_vertex);
            let texture_offset = (front_side.x_offset, -front_side.y_offset);

            // low
            if let Some(texture) = self.get_texture(front_side.lower_tex) {
                let line_offset;
                if (l.flags & LinedefFlags::LOWER_TEX_UNPEGGED) != LinedefFlags::NONE {
                    let off = (wall_extent.1 - wall_extent.0) / texture.height as f32;
                    let off = ((1.0 - off % 1.0) * texture.height as f32) as i16;
                    line_offset = (texture_offset.0, texture_offset.1 + off);
                } else {
                    line_offset = texture_offset;
                }

                self.prepare_line_render(
                    &mut per_material,
                    &mut vbuffer,
                    texture,
                    line,
                    (front_floor, back_floor),
                    line_offset,
                    front_light,
                );
            }

            // mid
            if let Some(texture) = self.get_texture(front_side.middle_tex) {
                let line_offset;
                if (l.flags & LinedefFlags::LOWER_TEX_UNPEGGED) == LinedefFlags::NONE {
                    let off = (back_ceil - back_floor) / texture.height as f32;
                    let off = ((1.0 - off % 1.0) * texture.height as f32) as i16;
                    line_offset = (texture_offset.0, texture_offset.1 + off);
                } else {
                    line_offset = texture_offset;
                }
                self.prepare_line_render(
                    &mut per_material,
                    &mut vbuffer,
                    texture,
                    line,
                    (back_floor, back_ceil),
                    line_offset,
                    front_light,
                );
            }

            // upper
            if let Some(texture) = self.get_texture(front_side.upper_tex) {
                let line_offset;
                if (l.flags & LinedefFlags::UPPER_TEX_UNPEGGED) != LinedefFlags::NONE {
                    let off = (front_ceil - back_ceil) / texture.height as f32;
                    let off = ((1.0 - off % 1.0) * texture.height as f32) as i16;
                    line_offset = (texture_offset.0, texture_offset.1 + off);
                } else {
                    line_offset = texture_offset;
                }
                self.prepare_line_render(
                    &mut per_material,
                    &mut vbuffer,
                    texture,
                    line,
                    (back_ceil, front_ceil),
                    line_offset,
                    front_light,
                );
            }

            // back
            let line = (l.end_vertex, l.start_vertex);

            if let Some(b) = back_side {
                let texture_offset = (b.x_offset, b.y_offset);
                // low
                if let Some(texture) = self.get_texture(b.lower_tex) {
                    let line_offset;
                    if (l.flags & LinedefFlags::LOWER_TEX_UNPEGGED) != LinedefFlags::NONE {
                        let off = (wall_extent.0 - wall_extent.1) / texture.height as f32;
                        let off = ((1.0 - off % 1.0) * texture.height as f32) as i16;
                        line_offset = (texture_offset.0, texture_offset.1 + off);
                    } else {
                        line_offset = texture_offset;
                    }

                    self.prepare_line_render(
                        &mut per_material,
                        &mut vbuffer,
                        texture,
                        line,
                        (back_floor, front_floor),
                        line_offset,
                        back_light,
                    );
                }

                // mid
                if let Some(texture) = self.get_texture(b.middle_tex) {
                    let line_offset;
                    if (l.flags & LinedefFlags::LOWER_TEX_UNPEGGED) == LinedefFlags::NONE {
                        let off = (back_ceil - back_floor) / texture.height as f32;
                        let off = ((1.0 - off % 1.0) * texture.height as f32) as i16;
                        line_offset = (texture_offset.0, texture_offset.1 + off);
                    } else {
                        line_offset = texture_offset;
                    }
                    self.prepare_line_render(
                        &mut per_material,
                        &mut vbuffer,
                        texture,
                        line,
                        (front_floor, front_ceil),
                        line_offset,
                        back_light,
                    );
                }

                // upper
                if let Some(texture) = self.get_texture(b.upper_tex) {
                    let line_offset;
                    if (l.flags & LinedefFlags::UPPER_TEX_UNPEGGED) != LinedefFlags::NONE {
                        let off = (back_ceil - front_ceil) / texture.height as f32;
                        let off = ((1.0 - off % 1.0) * texture.height as f32) as i16;
                        line_offset = (texture_offset.0, texture_offset.1 + off);
                    } else {
                        line_offset = texture_offset;
                    }
                    self.prepare_line_render(
                        &mut per_material,
                        &mut vbuffer,
                        texture,
                        line,
                        (front_ceil, back_ceil),
                        line_offset,
                        back_light,
                    );
                }
            }
        }

        (per_material, vbuffer)
    }

    pub fn prepare_render_finalize(&mut self, input: (HashMap<u32, PerMaterial>, Vec<GVertex>)) {
        self.vbuffer = input.1;
        let mut vb = unsafe { std::mem::zeroed() };
        unsafe {
            self.gl.GenBuffers(1, &mut vb);
            assert!(self.gl.GetError() == 0);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, vb);
            assert!(self.gl.GetError() == 0);
            self.gl.BufferData(
                gl::ARRAY_BUFFER,
                (self.vbuffer.len() * std::mem::size_of::<GVertex>()) as gl::types::GLsizeiptr,
                self.vbuffer.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );
            assert!(self.gl.GetError() == 0);
        }

        // Create actual sectors
        for mat in input.0.iter() {
            self.model_sectors
                .push(SectorModel::new(&self.gl, mat.1.ibuffer.clone(), *mat.0));
        }
    }

    pub fn render(&mut self, camera: &Camera) {
        unsafe {
            self.gl.ClearColor(0.5, 0.0, 0.5, 1.0);
            self.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let view = Matrix4::look_at_rh(
                camera.origin,
                camera.origin + camera.direction * Vector3::unit_z(),
                Vector3::unit_y(),
            );

            for s in &self.model_sectors {
                s.render(&view, &camera.persp, &self.gl);
            }
        }
    }

    pub fn load_map(name: &str, mut wad: WadFile, gl: gl::Gl) -> Result<WadMap, String> {
        let mapidx = match wad.directory.find_section(name, 0) {
            Some(i) => i,
            None => return Err("Map not found".to_string()),
        };

        let linedefs = wad.read_section(mapidx, "LINEDEFS");
        let sidedefs = wad.read_section(mapidx, "SIDEDEFS");
        let vertexes = wad.read_section(mapidx, "VERTEXES");
        let segs = wad.read_section(mapidx, "SEGS");
        let sectors = wad.read_section(mapidx, "SECTORS");
        let ssectors = wad.read_section(mapidx, "SSECTORS");
        let nodes = wad.read_section(mapidx, "NODES");
        wad.read_playpal();
        wad.read_pnames();
        wad.read_textures("TEXTURE1", &gl);
        wad.read_textures("TEXTURE2", &gl);

        Ok(WadMap {
            linedefs,
            sidedefs,
            segs,
            sectors,
            ssectors,
            nodes,
            vertexes,
            vbuffer: Vec::new(),
            wall_ibuffer: Vec::new(),
            model_sectors: Vec::new(),
            textures: wad.textures,
            gl,
        })
    }
}
