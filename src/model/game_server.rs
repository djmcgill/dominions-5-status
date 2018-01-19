#[derive(Debug)]
pub struct GameServer {
    pub alias: String,
    pub state: GameServerState,
}

#[derive(Debug)]
pub enum GameServerState {
    StartedState(StartedState),
    Lobby
}

#[derive(Debug)]
pub struct StartedState {
    pub address: String,
    pub last_seen_turn: i32,
}
