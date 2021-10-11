use std::cell::RefCell;
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::mem;
use std::{fs::File, path::Path};

use super::directory::WadDirectory;

pub struct WadFile {
    pub directory: WadDirectory,
    pub reader: RefCell<BufReader<File>>,
    content: Vec<u8>,
}

impl WadFile {
    pub fn new(file_name: &Path) -> Result<Self, String> {
        let mut file = match File::open(file_name) {
            Ok(file) => file,
            Err(e) => return Err(e.to_string()),
        };

        let reader = RefCell::new(io::BufReader::new(file.try_clone().unwrap()));
        let mut content = Vec::new();
        file.read_to_end(&mut content).unwrap();

        Ok(WadFile {
            directory: WadDirectory::new(&content),
            reader,
            content,
        })
    }

    pub fn get_section(&self, name: &str) -> Option<&[u8]> {
        if let Some(index) = self.directory.find_section(name, 0) {
            let lump = self.directory.get_lump(index);
            return Some(&self.content[lump.range()]);
        }
        None
    }

    pub fn read_section<T: Copy>(&self, mapidx: usize, name: &str) -> Vec<T> {
        let index = self.directory.find_section(name, mapidx).unwrap();
        let lump = &self.directory.files[index];
        self.reader
            .borrow_mut()
            .seek(SeekFrom::Start(lump.file_pos as u64))
            .unwrap();

        let count = lump.size as usize / mem::size_of::<T>();

        let mut result = Vec::new();
        let mut buffer = Vec::new();
        buffer.resize(mem::size_of::<T>(), 0u8);
        for _ in 0..count {
            if self.reader.borrow_mut().read_exact(&mut buffer).is_ok() {
                let (_, body, _) = unsafe { buffer.align_to::<T>() };
                result.push(body[0]);
            }
        }
        result
    }
}
