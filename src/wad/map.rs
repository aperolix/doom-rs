use core::str;
use std::{cell::RefCell, collections::HashMap, str::FromStr};

use crate::{
    camera::Camera,
    render::{
        doom_gl::{gl, DoomGl, GVertex},
        flat_model::FlatModel,
        material::Material,
        wall_model::WallModel,
    },
};

use super::textures::Texture;
use crate::sys::content::Content;

use bitflags::bitflags;
use cgmath::{AbsDiffEq, InnerSpace, Matrix4, Vector2, Vector3};

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
pub struct Sector {
    floor: i16,
    ceiling: i16,
    floor_tex: [u8; 8],
    ceil_tex: [u8; 8],
    lighting: i16,
    stype: SectorType,
    tag: i16,
}

pub struct WadMap {
    linedefs: Vec<LineDef>,
    sidedefs: Vec<SideDef>,
    vertexes: Vec<Vertex>,
    sectors: Vec<Sector>,

    vbuffer: RefCell<Vec<GVertex>>,
    walls: RefCell<Vec<WallModel>>,
    flats: RefCell<Vec<FlatModel>>,
}

impl WadMap {
    /// Add a wall quad
    fn add_quad(
        &self,
        wall_index: usize,
        line: (u16, u16),
        heights: (f32, f32),
        texture_size: (f32, f32),
        texture_offset: (f32, f32),
        light: f32,
    ) {
        let start = self.vertexes[line.0 as usize];
        let end = self.vertexes[line.1 as usize];

        let line = Vector3::new(end.x as f32, end.y as f32, 0.0f32)
            - Vector3::new(start.x as f32, start.y as f32, 0.0f32);
        let length = line.magnitude();

        let uv_offset = (
            texture_offset.0 / texture_size.0,
            texture_offset.1 / texture_size.1,
        );

        let mut quad_buffer = vec![
            GVertex {
                pos: Vector3::new(-start.x as f32, heights.0, start.y as f32),
                uv: Vector2::new(uv_offset.0, uv_offset.1),
                light,
            },
            GVertex {
                pos: Vector3::new(-end.x as f32, heights.0, end.y as f32),
                uv: Vector2::new(length / texture_size.0 + uv_offset.0, 0.0f32 + uv_offset.1),
                light,
            },
            GVertex {
                pos: Vector3::new(-start.x as f32, heights.1, start.y as f32),
                uv: Vector2::new(
                    uv_offset.0,
                    (heights.1 - heights.0) / texture_size.1 + uv_offset.1,
                ),
                light,
            },
            GVertex {
                pos: Vector3::new(-end.x as f32, heights.1, end.y as f32),
                uv: Vector2::new(
                    length / texture_size.0 + uv_offset.0,
                    (heights.1 - heights.0) / texture_size.1 + uv_offset.1,
                ),
                light,
            },
        ];

        let startidx = self.vbuffer.borrow().len() as u16;

        self.vbuffer.borrow_mut().append(&mut quad_buffer);
        self.walls.borrow_mut()[wall_index].append_indexes(vec![
            startidx,
            startidx + 1,
            startidx + 2,
            startidx + 2,
            startidx + 1,
            startidx + 3,
        ]);
    }

    /// Prepare wall side rendering
    fn prepare_line_render(
        &self,
        model_per_texture: &mut HashMap<u32, usize>,
        texture: &Texture,
        line: (u16, u16),
        heights: (f32, f32),
        texture_offset: (f32, f32),
        light: f32,
    ) {
        let wall_index = if let Some(i) = model_per_texture.get(&texture.id) {
            *i
        } else {
            self.walls
                .borrow_mut()
                .push(WallModel::new(texture.id, texture.sky));
            let index = self.walls.borrow().len() - 1;
            model_per_texture.insert(texture.id, index);
            index
        };

        // push vertices
        self.add_quad(
            wall_index,
            line,
            heights,
            (texture.width as f32, texture.height as f32),
            texture_offset,
            light,
        );
    }

    /// Handle the wall model creation
    fn prepare_wall_render(&self, content: &Content) {
        let mut model_per_texture = HashMap::new();

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

            let mut back_sector_is_sky = false;
            if let Some(back) = back_sector {
                let sky = [
                    'F' as u8, '_' as u8, 'S' as u8, 'K' as u8, 'Y' as u8, '1' as u8,
                ];
                if back.ceil_tex[0..6] == sky {
                    back_sector_is_sky = true;
                }
            }

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
            let texture_offset = (front_side.x_offset as f32, -front_side.y_offset as f32);

            // low
            if let Some(texture) = content.get_texture(
                &String::from_utf8(front_side.lower_tex.to_ascii_uppercase().to_vec()).unwrap(),
            ) {
                let line_offset;
                if (l.flags & LinedefFlags::LOWER_TEX_UNPEGGED) != LinedefFlags::NONE {
                    let off = (wall_extent.1 - wall_extent.0) / texture.height as f32;
                    let off = (1.0 - off % 1.0) * texture.height as f32;
                    line_offset = (texture_offset.0, texture_offset.1 + off);
                } else {
                    line_offset = texture_offset;
                }

                self.prepare_line_render(
                    &mut model_per_texture,
                    &texture,
                    line,
                    (front_floor, back_floor),
                    line_offset,
                    front_light,
                );
            }

            // mid
            if let Some(texture) = content.get_texture(
                &String::from_utf8(front_side.middle_tex.to_ascii_uppercase().to_vec()).unwrap(),
            ) {
                let line_offset;
                if (l.flags & LinedefFlags::LOWER_TEX_UNPEGGED) == LinedefFlags::NONE {
                    let off = (back_ceil - back_floor) / texture.height as f32;
                    let off = (1.0 - off % 1.0) * texture.height as f32;
                    line_offset = (texture_offset.0, texture_offset.1 + off);
                } else {
                    line_offset = texture_offset;
                }
                self.prepare_line_render(
                    &mut model_per_texture,
                    &texture,
                    line,
                    (back_floor, back_ceil),
                    line_offset,
                    front_light,
                );
            }

            // upper
            let texture_name = if back_sector_is_sky {
                String::from_str("F_SKY1").unwrap()
            } else {
                String::from_utf8(front_side.upper_tex.to_ascii_uppercase().to_vec()).unwrap()
            };
            if let Some(texture) = content.get_texture(&texture_name) {
                let line_offset;
                if (l.flags & LinedefFlags::UPPER_TEX_UNPEGGED) != LinedefFlags::NONE {
                    let off = (front_ceil - back_ceil) / texture.height as f32;
                    let off = (1.0 - off % 1.0) * texture.height as f32;
                    line_offset = (texture_offset.0, texture_offset.1 + off);
                } else {
                    line_offset = texture_offset;
                }
                self.prepare_line_render(
                    &mut model_per_texture,
                    &texture,
                    line,
                    (back_ceil, front_ceil),
                    line_offset,
                    front_light,
                );
            }

            // back
            let line = (l.end_vertex, l.start_vertex);

            if let Some(b) = back_side {
                let texture_offset = (b.x_offset as f32, b.y_offset as f32);
                // low
                if let Some(texture) = content.get_texture(
                    &String::from_utf8(b.lower_tex.to_ascii_uppercase().to_vec()).unwrap(),
                ) {
                    let line_offset;
                    if (l.flags & LinedefFlags::LOWER_TEX_UNPEGGED) != LinedefFlags::NONE {
                        let off = (wall_extent.0 - wall_extent.1) / texture.height as f32;
                        let off = (1.0 - off % 1.0) * texture.height as f32;
                        line_offset = (texture_offset.0, texture_offset.1 + off);
                    } else {
                        line_offset = texture_offset;
                    }

                    self.prepare_line_render(
                        &mut model_per_texture,
                        &texture,
                        line,
                        (back_floor, front_floor),
                        line_offset,
                        back_light,
                    );
                }

                // mid
                if let Some(texture) = content.get_texture(
                    &String::from_utf8(b.middle_tex.to_ascii_uppercase().to_vec()).unwrap(),
                ) {
                    let line_offset;
                    if (l.flags & LinedefFlags::LOWER_TEX_UNPEGGED) == LinedefFlags::NONE {
                        let off = (back_ceil - back_floor) / texture.height as f32;
                        let off = (1.0 - off % 1.0) * texture.height as f32;
                        line_offset = (texture_offset.0, texture_offset.1 + off);
                    } else {
                        line_offset = texture_offset;
                    }
                    self.prepare_line_render(
                        &mut model_per_texture,
                        &texture,
                        line,
                        (front_floor, front_ceil),
                        line_offset,
                        back_light,
                    );
                }

                // upper
                if let Some(texture) = content.get_texture(
                    &String::from_utf8(b.upper_tex.to_ascii_uppercase().to_vec()).unwrap(),
                ) {
                    let line_offset;
                    if (l.flags & LinedefFlags::UPPER_TEX_UNPEGGED) != LinedefFlags::NONE {
                        let off = (back_ceil - front_ceil) / texture.height as f32;
                        let off = (1.0 - off % 1.0) * texture.height as f32;
                        line_offset = (texture_offset.0, texture_offset.1 + off);
                    } else {
                        line_offset = texture_offset;
                    }
                    self.prepare_line_render(
                        &mut model_per_texture,
                        &texture,
                        line,
                        (front_ceil, back_ceil),
                        line_offset,
                        back_light,
                    );
                }
            }
        }
    }

    fn prepare_ground_ceil(&self, _content: &Content) {}

    /// Prepare the vbuffer & ibuffer of the map
    fn prepare_render(&self, content: &Content) {
        self.prepare_wall_render(content);
        self.prepare_ground_ceil(content);

        let mut vb = unsafe { std::mem::zeroed() };
        unsafe {
            let gl = DoomGl::gl();
            gl.GenBuffers(1, &mut vb);
            assert!(gl.GetError() == 0);
            gl.BindBuffer(gl::ARRAY_BUFFER, vb);
            assert!(gl.GetError() == 0);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (self.vbuffer.borrow().len() * std::mem::size_of::<GVertex>())
                    as gl::types::GLsizeiptr,
                self.vbuffer.borrow().as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );
            assert!(gl.GetError() == 0);
        }

        // Create actual sectors
        for wall in self.walls.borrow_mut().iter_mut() {
            wall.init();
        }
    }

    /// Render the map
    pub fn render(&self, camera: &Camera) {
        unsafe {
            let gl = DoomGl::gl();
            gl.ClearColor(0.5, 0.0, 0.5, 1.0);
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let view = Matrix4::look_at_rh(
                camera.origin,
                camera.origin + camera.direction * Vector3::unit_z(),
                Vector3::unit_y(),
            );

            for s in self.walls.borrow().iter() {
                s.render(&view, &camera.persp);
            }

            for f in self.flats.borrow().iter() {
                f.render(&view, &camera.persp);
            }
        }
    }

    /// Load the map and prepare render
    pub fn new(name: &str, content: &Content) -> Result<WadMap, String> {
        let mapidx = match content.file.directory.find_section(name, 0) {
            Some(i) => i,
            None => return Err("Map not found".to_string()),
        };

        let linedefs: Vec<LineDef> = content.file.read_section(mapidx, "LINEDEFS");
        let sidedefs: Vec<SideDef> = content.file.read_section(mapidx, "SIDEDEFS");
        let vertexes: Vec<Vertex> = content.file.read_section(mapidx, "VERTEXES");
        let sectors: Vec<Sector> = content.file.read_section(mapidx, "SECTORS");

        let mut flats = Vec::new();

        let mut sector_lines = Vec::new();
        for sector_idx in 0..sectors.len() {
            sector_lines.push(Vec::new());
            for (side_idx, sidedef) in sidedefs.as_slice().iter().enumerate() {
                if sidedef.sector == sector_idx as i16 {
                    for (linedef_idx, linedef) in linedefs.as_slice().iter().enumerate() {
                        if linedef.front_sidedef == side_idx as i16
                            || linedef.back_sidedef == side_idx as i16
                        {
                            let start_vertex =
                                &vertexes[linedefs[linedef_idx].start_vertex as usize];
                            let end_vertex = &vertexes[linedefs[linedef_idx].end_vertex as usize];
                            let start_point = [start_vertex.x as f32, start_vertex.y as f32];
                            let end_point = [end_vertex.x as f32, end_vertex.y as f32];
                            sector_lines[sector_idx].push((start_point, end_point));
                        }
                    }
                }
            }

            // Try to form polygons
            let mut polygons = Vec::new();
            let mut hole_idx = Vec::new();
            while !sector_lines[sector_idx].is_empty() {
                let line = sector_lines[sector_idx].remove(0);

                polygons.push([line.0, line.1].to_vec());

                let current_poly = polygons.last_mut().unwrap();

                loop {
                    let mut index = None;
                    for (i, line) in sector_lines[sector_idx].iter().enumerate() {
                        if line
                            .0
                            .abs_diff_eq(current_poly.first().unwrap(), f32::EPSILON)
                        {
                            current_poly.insert(0, line.1);
                            index = Some(i);
                            break;
                        } else if line
                            .1
                            .abs_diff_eq(current_poly.first().unwrap(), f32::EPSILON)
                        {
                            current_poly.insert(0, line.0);
                            index = Some(i);
                            break;
                        } else if line
                            .0
                            .abs_diff_eq(current_poly.last().unwrap(), f32::EPSILON)
                        {
                            current_poly.push(line.1);
                            index = Some(i);
                            break;
                        } else if line
                            .1
                            .abs_diff_eq(current_poly.last().unwrap(), f32::EPSILON)
                        {
                            current_poly.push(line.0);
                            index = Some(i);
                            break;
                        }
                    }
                    if let Some(i) = index {
                        sector_lines[sector_idx].remove(i);
                    } else {
                        break;
                    }
                }
                if current_poly
                    .first()
                    .unwrap()
                    .abs_diff_eq(current_poly.last().unwrap(), f32::EPSILON)
                {
                    current_poly.pop();
                }
                if hole_idx.is_empty() {
                    hole_idx.push(current_poly.len());
                } else {
                    hole_idx.push(hole_idx.last().unwrap() + current_poly.len());
                }
            }
            hole_idx.pop();
            let datas = polygons
                .into_iter()
                .flatten()
                .collect::<Vec<[f32; 2]>>()
                .into_iter()
                .flatten()
                .collect::<Vec<f32>>();
            let ib = earcutr::earcut(&datas, &hole_idx, 2);

            let ceil_texture = content.get_texture(
                &String::from_utf8(sectors[sector_idx].ceil_tex.to_ascii_uppercase().to_vec())
                    .unwrap(),
            );
            let floor_texture = content
                .get_texture(
                    &String::from_utf8(sectors[sector_idx].floor_tex.to_ascii_uppercase().to_vec())
                        .unwrap(),
                )
                .unwrap();

            let mut model = FlatModel::new(
                datas,
                ib.iter().map(|i| *i as u16).collect(),
                match ceil_texture {
                    Some(t) => t.id,
                    _ => u32::MAX,
                },
                floor_texture.id,
                ceil_texture.unwrap().sky,
            );
            model.light = sectors[sector_idx].lighting as f32 / 255.0;
            model.floor = sectors[sector_idx].floor as f32;
            model.ceil = sectors[sector_idx].ceiling as f32;

            model.init();
            flats.push(model);
        }

        let map = WadMap {
            linedefs,
            sidedefs,
            sectors,
            vertexes,
            vbuffer: RefCell::new(Vec::new()),
            walls: RefCell::new(Vec::new()),
            flats: RefCell::new(flats),
        };
        map.prepare_render(content);
        Ok(map)
    }
}
