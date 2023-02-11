use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    mem,
};

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
        #[repr(C, packed)]
        #[derive(Clone, Copy)]
        pub struct WadInfo {
            pub identification: [u8; 4],
            pub num_lumps: i32,
            pub into_table_ofs: i32,
        }

        reader.rewind().unwrap();

        let mut buffer = [0u8; mem::size_of::<WadInfo>()];
        reader.read_exact(&mut buffer).unwrap();
        let (_, info, _) = unsafe { buffer.align_to::<WadInfo>() };
        let info = info[0];

        let num_lumps = usize::try_from(info.num_lumps).unwrap();
        let mut files = Vec::with_capacity(num_lumps);

        reader
            .seek(SeekFrom::Start(info.into_table_ofs as u64))
            .unwrap();

        for _ in 0..num_lumps {
            let mut buffer = [0u8; mem::size_of::<FileLump>()];
            reader.read_exact(&mut buffer).unwrap();
            let (_, lump, _) = unsafe { buffer.align_to::<FileLump>() };
            files.push(lump[0]);
        }

        WadDirectory { files }
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
