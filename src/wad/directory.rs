use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    mem,
};

use super::info::WadInfo;
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct FileLump {
    pub file_pos: i32,
    pub size: i32,
    pub name: [u8; 8],
}

pub struct WadDirectory {
    pub files: Vec<FileLump>,
}

impl WadDirectory {
    pub fn new(reader: &mut BufReader<File>) -> Self {
        let info = WadInfo::new(reader);

        let mut directory = WadDirectory {
            files: Vec::with_capacity(info.num_lumps as usize),
        };

        reader
            .seek(SeekFrom::Start(info.into_table_ofs as u64))
            .unwrap();

        for _ in 0..info.num_lumps {
            let mut buffer = [0u8; mem::size_of::<FileLump>()];
            reader.read_exact(&mut buffer).unwrap();
            let (_, lump, _) = unsafe { buffer.align_to::<FileLump>() };
            directory.files.push(lump[0]);
        }

        directory
    }

    pub fn find_section(&self, name: &str, from_index: usize) -> Option<usize> {
        if name.len() > 8 {
            panic!("Section name must be 8 char or less");
        }
        let name = &name.as_bytes()[0..name.len()];
        (from_index..self.files.len()).find(|&i| self.files[i].name[0..name.len()].eq(name))
    }

    pub fn get_lump(&self, index: usize) -> &FileLump {
        &self.files[index]
    }

    pub fn get_lump_index(&self, name: &str) -> Option<usize> {
        let name = &name.as_bytes()[0..name.len()];
        (0..self.files.len()).find(|&i| self.files[i].name[0..name.len()].eq(name))
    }
}
