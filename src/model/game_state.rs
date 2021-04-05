use crate::model::enums::{Era, NationStatus, SubmissionStatus};
use crate::model::game_data::GameData;
use crate::model::nation::{BotNationIdentifier, GameNationIdentifier};
use crate::snek::SnekGameStatus;
use serenity::model::id::UserId;
use std::borrow::Cow;

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
    RegisteredOnly(UserId, BotNationIdentifier),
    RegisteredAndGame(UserId, PlayerDetails),
    GameOnly(PlayerDetails),
}
impl PotentialPlayer {
    pub fn nation_name(&self, option_snek_state: Option<&SnekGameStatus>) -> Cow<'static, str> {
        match &self {
            PotentialPlayer::GameOnly(player_details) => {
                player_details.nation_identifier.name(option_snek_state)
            }
            PotentialPlayer::RegisteredAndGame(_, player_details) => {
                player_details.nation_identifier.name(option_snek_state)
            }
            PotentialPlayer::RegisteredOnly(_, bot_nation_identifier) => {
                bot_nation_identifier.name(option_snek_state)
            }
        }
    }
    pub fn nation_id(&self) -> Option<u32> {
        match self {
            PotentialPlayer::GameOnly(player_details) => {
                Some(player_details.nation_identifier.id())
            }
            PotentialPlayer::RegisteredAndGame(_, player_details) => {
                Some(player_details.nation_identifier.id())
            }
            PotentialPlayer::RegisteredOnly(_, bot_nation_identifier) => bot_nation_identifier.id(),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PlayerDetails {
    pub nation_identifier: GameNationIdentifier,
    pub submitted: SubmissionStatus,
    pub player_status: NationStatus,
}
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct UploadingPlayer {
    pub potential_player: PotentialPlayer,
    pub uploaded: bool,
}
impl UploadingPlayer {
    pub fn nation_name(&self, option_snek_state: Option<&SnekGameStatus>) -> Cow<'static, str> {
        match self.potential_player {
            PotentialPlayer::RegisteredOnly(_, ref bot_nation_identifier) => {
                bot_nation_identifier.name(option_snek_state)
            }
            PotentialPlayer::RegisteredAndGame(_, ref player_details) => {
                player_details.nation_identifier.name(option_snek_state)
            }
            PotentialPlayer::GameOnly(ref player_details) => {
                player_details.nation_identifier.name(option_snek_state)
            }
        }
    }
    pub fn option_player_id(&self) -> Option<&UserId> {
        match self.potential_player {
            PotentialPlayer::RegisteredOnly(ref user, _) => Some(&user),
            PotentialPlayer::RegisteredAndGame(ref user, _) => Some(&user),
            _ => None,
        }
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
    pub nation_identifier: BotNationIdentifier,
    pub cached_name: Cow<'static, str>,
}
