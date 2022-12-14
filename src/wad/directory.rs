use std::ops::Range;

use super::info::WadInfo;
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct FileLump {
    pub file_pos: i32,
    pub size: i32,
    pub name: [u8; 8],
}

impl FileLump {
    pub fn range(&self) -> Range<usize> {
        Range {
            start: self.file_pos as usize,
            end: self.file_pos as usize + self.size as usize,
        }
    }
}

pub struct WadDirectory {
    pub files: Vec<FileLump>,
}

impl WadDirectory {
    pub fn new(content: &[u8]) -> Self {
        let info = WadInfo::new(content);

        let mut directory = WadDirectory {
            files: Vec::with_capacity(info.num_lumps as usize),
        };

        let offset = info.into_table_ofs as usize;

        let body = unsafe {
            directory.files.set_len(info.num_lumps as usize);
            let (_head, body, _tail) = content[offset..].align_to::<FileLump>();
            body
        };
        directory.files.copy_from_slice(body);

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
