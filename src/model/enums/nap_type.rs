#[derive(PartialEq, Debug, Clone, Copy)]
pub enum NapType {
    Fixed { end_turn: u32 },
    Rolling { notice_length: u32 },
}
