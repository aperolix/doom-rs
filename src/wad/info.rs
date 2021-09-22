use std::io::BufRead;
use std::mem;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct WadInfo {
    pub identification: [u8; 4],
    pub num_lumps: i32,
    pub into_table_ofs: i32,
}

impl WadInfo {
    pub fn from_reader(reader: &mut dyn BufRead) -> Result<Self, String> {
        let mut buffer: [u8; mem::size_of::<WadInfo>()] = [0; mem::size_of::<WadInfo>()];
        match reader.read_exact(&mut buffer) {
            Ok(_) => {
                let (head, body, _tail) = unsafe { buffer.align_to::<WadInfo>() };
                assert!(head.is_empty(), "Data was not aligned");

                let wadinfo = body[0];

                if !wadinfo.identification[1..].eq(b"WAD") {
                    return Err("Not a valid WAD file".to_string());
                } else if wadinfo.identification[0].eq(&b'P') {
                    println!("Patch WAD file detected");
                } else if !wadinfo.identification[0].eq(&b'I') {
                    return Err("Not a valid WAD file type".to_string());
                }
                Ok(wadinfo)
            }
            Err(e) => Err(e.to_string()),
        }
    }
}
