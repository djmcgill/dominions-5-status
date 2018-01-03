#[repr(C)]
#[derive(Debug)]
pub struct RawGameData {
    pub a: [u8; 6], // 6
    pub game_name: String,
    pub c: [u8; 6], // 6
    pub d: u32, // 4
    pub e: u8,  // 1
    pub f: Vec<u8>, // ; 750],
    pub g: u8,  // 1
    pub h: u32, // 4
    pub i: u32, // 4
    pub j: u8,  // 1
}
