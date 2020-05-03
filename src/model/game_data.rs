use crate::model::nation::Nation;
use crate::model::enums::Era;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameData {
    pub game_name: String,
    pub nations: Vec<Nation>,
    pub turn: i32,
    pub turn_timer: i32,
    pub era: Era,
}
