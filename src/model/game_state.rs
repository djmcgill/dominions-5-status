use crate::model::game_data::GameData;
use crate::snek::SnekGameStatus;
use serenity::model::id::UserId;
use crate::model::enums::{Nations, SubmissionStatus, NationStatus, Era};
use std::cmp::Ordering;

/// We cache the call to the server (both the game itself and the snek api)
/// but NOT the db call
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct CacheEntry {
    pub game_data: GameData,
    pub option_snek_state: Option<SnekGameStatus>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct GameDetails {
    pub alias: String,
    pub owner: Option<UserId>,
    pub description: Option<String>,
    pub nations: NationDetails,
    pub cache_entry: Option<CacheEntry>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum NationDetails {
    Lobby(LobbyDetails),
    Started(StartedDetails),
}
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct StartedDetails {
    pub address: String,
    pub game_name: String,
    pub state: StartedStateDetails,
}

pub fn get_nation_string(option_snek_state: &Option<SnekGameStatus>, nation_id: u32) -> String {
    let snek_nation_details = option_snek_state
        .as_ref()
        .and_then(|snek_details| snek_details.nations.get(&nation_id));
    match snek_nation_details {
        Some(snek_nation) => snek_nation.name.clone(),
        None => {
            let &(nation_name, _) = Nations::get_nation_desc(nation_id);
            nation_name.to_owned()
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum StartedStateDetails {
    Playing(PlayingState),
    Uploading(UploadingState),
}
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct UploadingState {
    pub uploading_players: Vec<UploadingPlayer>,
}
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PlayingState {
    pub players: Vec<PotentialPlayer>,
    pub turn: u32,
    pub mins_remaining: i32,
    pub hours_remaining: i32,
}
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum PotentialPlayer {
    RegisteredOnly(UserId, u32, String),
    RegisteredAndGame(UserId, PlayerDetails),
    GameOnly(PlayerDetails),
}
impl PotentialPlayer {
    pub fn nation_name(&self) -> &String {
        match &self {
            PotentialPlayer::RegisteredOnly(_, _, nation_name) => nation_name,
            PotentialPlayer::RegisteredAndGame(_, details) => &details.nation_name,
            PotentialPlayer::GameOnly(details) => &details.nation_name,
        }
    }
    pub fn nation_id(&self) -> u32 {
        match &self {
            PotentialPlayer::RegisteredOnly(_, nation_id, _) => *nation_id,
            PotentialPlayer::RegisteredAndGame(_, details) => details.nation_id,
            PotentialPlayer::GameOnly(details) => details.nation_id,
        }
    }
    pub fn option_player_id(&self) -> Option<&UserId> {
        match &self {
            PotentialPlayer::RegisteredOnly(player_id, _, _) => Some(player_id),
            PotentialPlayer::RegisteredAndGame(player_id, _) => Some(player_id),
            PotentialPlayer::GameOnly(_) => None,
        }
    }
}
impl PartialOrd<Self> for PotentialPlayer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for PotentialPlayer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.nation_name().cmp(&other.nation_name())
    }
}
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PlayerDetails {
    pub nation_id: u32,
    pub nation_name: String,
    pub submitted: SubmissionStatus,
    pub player_status: NationStatus,
}
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct UploadingPlayer {
    pub potential_player: PotentialPlayer,
    pub uploaded: bool,
}
impl UploadingPlayer {
    pub fn nation_name(&self) -> &String {
        self.potential_player.nation_name()
    }
    pub fn nation_id(&self) -> u32 {
        self.potential_player.nation_id()
    }
    pub fn option_player_id(&self) -> Option<&UserId> {
        self.potential_player.option_player_id()
    }
}
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct LobbyDetails {
    pub players: Vec<LobbyPlayer>,
    pub era: Option<Era>,
    pub remaining_slots: u32,
}
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct LobbyPlayer {
    pub player_id: UserId,
    pub nation_id: u32,
    pub nation_name: String,
}
