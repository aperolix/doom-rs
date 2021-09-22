use std::io::{Seek, SeekFrom};
use std::{io::BufRead, mem};

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
    pub fn new<T: BufRead + Seek>(info: &WadInfo, reader: &mut T) -> Result<Self, String> {
        if let Err(e) = reader.seek(SeekFrom::Start(info.into_table_ofs as u64)) {
            return Err(e.to_string());
        }

        let mut directory = WadDirectory { files: Vec::new() };

        for _ in 0..info.num_lumps {
            let mut buffer: [u8; mem::size_of::<FileLump>()] = [0; mem::size_of::<FileLump>()];
            match reader.read_exact(&mut buffer) {
                Ok(_) => {
                    let (head, body, _tail) = unsafe { buffer.align_to::<FileLump>() };
                    assert!(head.is_empty(), "Data was not aligned");
                    let file_lump = body[0];
                    directory.files.push(file_lump);
                }
                Err(e) => return Err(e.to_string()),
            }
        }

        Ok(directory)
    }

    pub fn find_section(&self, name: &str, from_index: usize) -> Option<usize> {
        if name.len() > 8 {
            panic!("Section name must be 8 char or less");
        }
        let name = &name.as_bytes()[0..name.len()];
        for i in from_index..self.files.len() {
            if self.files[i].name[0..name.len()].eq(name) {
                return Some(i);
            }
        }
        None
    }
}
