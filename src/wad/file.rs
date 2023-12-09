use std::cell::RefCell;
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::mem;
use std::{fs::File, path::Path};

use super::directory::WadDirectory;

pub struct WadFile {
    pub directory: WadDirectory,
    pub reader: RefCell<BufReader<File>>,
}

impl WadFile {
    pub fn new(file_name: &Path) -> Self {
        let mut file = File::open(file_name).expect("Cannot open WAD file");

        let mut content = Vec::new();
        file.read_to_end(&mut content).unwrap();
        let mut reader = io::BufReader::new(file);

        WadFile {
            directory: WadDirectory::new(&mut reader),
            reader: RefCell::new(reader),
        }
    }

    pub fn get_section(&self, name: &str) -> Vec<u8> {
        let mut buffer = Vec::new();
        let index = self
            .directory
            .find_section(name, 0)
            .expect("Couldn't find section");
        let lump = self.directory.get_lump(index);
        let mut reader = self.reader.borrow_mut();
        reader
            .seek(SeekFrom::Start(lump.file_pos as u64))
            .expect("Couldn't read section");
        buffer.resize(lump.size as usize, 0u8);
        reader
            .read_exact(&mut buffer)
            .expect("Couldn't read section");

        buffer
    }

    pub fn read_section<T: Copy>(&self, mapidx: usize, name: &str) -> Vec<T> {
        let index = self.directory.find_section(name, mapidx).unwrap();
        let lump = &self.directory.files[index];

        let mut reader = self.reader.borrow_mut();
        reader.seek(SeekFrom::Start(lump.file_pos as u64)).unwrap();

        let count = lump.size as usize / mem::size_of::<T>();

        let mut result = Vec::new();
        let mut buffer = vec![0; mem::size_of::<T>()];
        for _ in 0..count {
            if reader.read_exact(&mut buffer).is_ok() {
                let (_, body, _) = unsafe { buffer.align_to::<T>() };
                result.push(body[0]);
            }
        }
        result
    }
}
