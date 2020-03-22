#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
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

    pub fn from_int(i: u8) -> Option<Self> {
        match i {
            0 => Some(NationStatus::Empty),
            1 => Some(NationStatus::Human),
            2 => Some(NationStatus::AI),
            3 => Some(NationStatus::Independent),
            253 => Some(NationStatus::Closed),
            254 => Some(NationStatus::DefeatedThisTurn),
            255 => Some(NationStatus::Defeated),
            _ => None,
        }
    }

    pub fn is_human(&self) -> bool {
        match self {
            NationStatus::Human | NationStatus::DefeatedThisTurn => true,
            _ => false,
        }
    }
}
