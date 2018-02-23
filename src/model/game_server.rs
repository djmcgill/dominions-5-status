use model::enums::Era;
use serenity::model::id::UserId;

#[derive(Debug, Clone, PartialEq)]
pub struct GameServer {
    pub alias: String,
    pub state: GameServerState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameServerState {
    StartedState(StartedState, Option<LobbyState>),
    Lobby(LobbyState),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StartedState {
    pub address: String,
    pub last_seen_turn: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LobbyState {
    pub owner: UserId,
    pub era: Era,
    pub player_count: i32,
}
