#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct RawGameData {
    pub a: [u8; 6], // 6
    pub game_name: String, // null terminated
    pub c: [u8; 6], // 4
    pub d: i32, // 4
    // pub e: u8,  // 1
    pub f: Vec<u8>, // ; 750],
    pub g: u8,  // 1
    pub h: u32, // 4
    pub i: u32, // 4
    pub j: u8,  // 1
}
