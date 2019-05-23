#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
pub enum NationStatus {
    Empty = 0,
    Human = 1,
    AI = 2,
    Independent = 3,
    Closed = 253,
    DefeatedThisTurn = 254,
    Defeated = 255,
}
impl NationStatus {
    pub fn show(self) -> &'static str {
        match self {
            NationStatus::Empty => "Empty",
            NationStatus::Human => "Human",
            NationStatus::AI => "AI",
            NationStatus::Independent => "Independent",
            NationStatus::Closed => "Closed",
            NationStatus::DefeatedThisTurn => "Defeated this turn",
            NationStatus::Defeated => "Defeated",
        }
    }

    pub fn from_int(i: u8) -> Self {
        match i {
            0 => NationStatus::Empty,
            1 => NationStatus::Human,
            2 => NationStatus::AI,
            3 => NationStatus::Independent,
            253 => NationStatus::Closed,
            254 => NationStatus::DefeatedThisTurn,
            255 => NationStatus::Defeated,
            _ => panic!(),
        }
    }
}
