use model::nation::Nation;

pub struct GameData {
    pub game_name: String,
    pub nations: Vec<Nation>,
    pub turn: i32,
}
