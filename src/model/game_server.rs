use model::enums::Era;
use serenity::model::UserId;

#[derive(Debug)]
pub struct GameServer {
    pub alias: String,
    pub state: GameServerState,
}

#[derive(Debug)]
pub enum GameServerState {
    StartedState(StartedState),
    Lobby(LobbyState),
}

#[derive(Debug)]
pub struct StartedState {
    pub address: String,
    pub last_seen_turn: i32,
}

#[derive(Debug)]
pub struct LobbyState {
    pub owner: UserId,
    pub era: Era,
    pub player_count: i32,
}
