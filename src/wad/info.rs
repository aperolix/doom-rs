#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct WadInfo {
    pub identification: [u8; 4],
    pub num_lumps: i32,
    pub into_table_ofs: i32,
}

impl WadInfo {
    pub fn new(content: &[u8]) -> Self {
        let (_head, body, _tail) = unsafe { content.align_to::<WadInfo>() };
        body[0]
    }
}
