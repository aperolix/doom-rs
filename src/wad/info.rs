use std::{
    fs::File,
    io::{BufReader, Read, Seek},
    mem,
};

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct WadInfo {
    pub identification: [u8; 4],
    pub num_lumps: i32,
    pub into_table_ofs: i32,
}

impl WadInfo {
    pub fn new(reader: &mut BufReader<File>) -> Self {
        reader.rewind().unwrap();

        let mut buffer = [0u8; mem::size_of::<WadInfo>()];
        reader.read_exact(&mut buffer).unwrap();
        let (_head, body, _tail) = unsafe { buffer.align_to::<WadInfo>() };
        body[0]
    }
}
