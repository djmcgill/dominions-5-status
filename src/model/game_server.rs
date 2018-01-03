#[derive(Debug)]
pub struct GameServer {
    pub address: String,
    pub alias: String,
    pub last_seen_turn: i32,
}
