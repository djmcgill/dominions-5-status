use model::nation::Nation;

#[derive(Debug, Clone, PartialEq)]
pub struct GameData {
    pub game_name: String,
    pub nations: Vec<Nation>,
    pub turn: i32,
    pub turn_timer: i32,
}
