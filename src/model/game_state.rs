use crate::model::enums::{Era, NationStatus, SubmissionStatus};
use crate::model::game_data::GameData;
use crate::model::nation::{BotNationIdentifier, GameNationIdentifier};
use crate::model::player::Player;
use crate::snek::SnekGameStatus;
use chrono::{DateTime, Utc};
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
impl StartedStateDetails {
    pub fn uploaded_players(&self) -> impl Iterator<Item = UserId> + '_ {
        match self {
            StartedStateDetails::Playing(playing_state) => {
                Box::new(playing_state.players.iter().filter_map(|potential_player| {
                    if let PotentialPlayer::RegisteredAndGame(registered_player, _)
                    | PotentialPlayer::RegisteredOnly(registered_player, _) = potential_player
                    {
                        Some(registered_player.discord_user_id)
                    } else {
                        None
                    }
                })) as Box<dyn Iterator<Item = UserId>>
            }
            StartedStateDetails::Uploading(uploading_state) => Box::new(
                uploading_state
                    .uploading_players
                    .iter()
                    .filter_map(|uploading_player| {
                        if let PotentialPlayer::RegisteredAndGame(registered_player, _)
                        | PotentialPlayer::RegisteredOnly(registered_player, _) =
                            &uploading_player.potential_player
                        {
                            Some(registered_player.discord_user_id)
                        } else {
                            None
                        }
                    }),
            )
                as Box<dyn Iterator<Item = UserId>>,
        }
    }
}
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct UploadingState {
    pub uploading_players: Vec<UploadingPlayer>,
}
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PlayingState {
    pub players: Vec<PotentialPlayer>,
    pub turn: u32,
    pub turn_deadline: DateTime<Utc>,
}
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum PotentialPlayer {
    RegisteredOnly(Player, BotNationIdentifier),
    RegisteredAndGame(Player, PlayerDetails),
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
    pub fn option_player_id(&self) -> Option<&Player> {
        match &self.potential_player {
            PotentialPlayer::RegisteredOnly(user, _) => Some(user),
            PotentialPlayer::RegisteredAndGame(user, _) => Some(user),
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
