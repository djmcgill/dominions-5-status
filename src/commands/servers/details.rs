use crate::server::ServerConnection;

use serenity::framework::standard::CommandError;

use crate::db::DbConnection;
use crate::model::enums::{Era, NationStatus, SubmissionStatus};
use crate::model::{
    BotNationIdentifier, GameData, GameNationIdentifier, GameServerState, LobbyState, Nation,
    Player, StartedState,
};
use crate::snek::SnekGameStatus;
use log::*;
use serenity::model::id::UserId;
use std::borrow::Cow;
use std::cmp::max;
use std::collections::HashMap;

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
        match self {
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

pub fn get_details_for_alias<C: ServerConnection>(
    db_conn: &DbConnection,
    alias: &str,
) -> Result<GameDetails, CommandError> {
    let server = db_conn.game_for_alias(&alias)?;
    info!("got server details");

    let details = match server.state {
        GameServerState::Lobby(ref lobby_state) => lobby_details(db_conn, lobby_state, &alias)?,
        GameServerState::StartedState(ref started_state, ref option_lobby_state) => {
            started_details::<C>(db_conn, started_state, option_lobby_state.as_ref(), &alias)?
        }
    };

    Ok(details)
}

pub fn lobby_details(
    db_conn: &DbConnection,
    lobby_state: &LobbyState,
    alias: &str,
) -> Result<GameDetails, CommandError> {
    let players_nations = db_conn.players_with_nations_for_game_alias(&alias)?;

    let mut player_nation_details: Vec<LobbyPlayer> = players_nations
        .into_iter()
        .map(|(player, nation_identifier)| {
            let name = nation_identifier.name(None);
            let lobby_player = LobbyPlayer {
                player_id: player.discord_user_id,
                nation_identifier,
                cached_name: name,
            };
            lobby_player
        })
        .collect();

    player_nation_details.sort_by(|n1, n2| n1.cached_name.cmp(&n2.cached_name));

    let remaining_slots = max(
        0,
        (lobby_state.player_count - player_nation_details.len() as i32) as u32,
    );

    let lobby_details = LobbyDetails {
        players: player_nation_details,
        era: Some(lobby_state.era),
        remaining_slots,
    };

    Ok(GameDetails {
        alias: alias.to_owned(),
        owner: Some(lobby_state.owner),
        description: lobby_state.description.clone(),
        nations: NationDetails::Lobby(lobby_details),
        cache_entry: None, // lobbies have no cache entry
    })
}

fn started_details<C: ServerConnection>(
    db_conn: &DbConnection,
    started_state: &StartedState,
    option_lobby_state: Option<&LobbyState>,
    alias: &str,
) -> Result<GameDetails, CommandError> {
    let server_address = &started_state.address;
    let game_data = C::get_game_data(&server_address)?;
    let option_snek_details = C::get_snek_data(server_address)?;

    started_details_from_server(
        db_conn,
        started_state,
        option_lobby_state,
        alias,
        game_data,
        option_snek_details,
    )
}

pub fn started_details_from_server(
    db_conn: &DbConnection,
    started_state: &StartedState,
    option_lobby_state: Option<&LobbyState>,
    alias: &str,
    game_data: GameData,
    option_snek_details: Option<SnekGameStatus>,
) -> Result<GameDetails, CommandError> {
    let id_player_nations = db_conn.players_with_nations_for_game_alias(&alias)?;
    let player_details = join_players_with_nations(&game_data.nations, &id_player_nations)?;

    let state_details = if game_data.turn < 0 {
        let uploaded_players_detail: Vec<UploadingPlayer> = player_details
            .into_iter()
            .map(|potential_player_detail| {
                match potential_player_detail {
                    potential_player @ PotentialPlayer::GameOnly(_) => {
                        UploadingPlayer {
                            potential_player,
                            uploaded: true, // all players we can see have uploaded
                        }
                    }
                    potential_player @ PotentialPlayer::RegisteredAndGame(_, _) => {
                        UploadingPlayer {
                            potential_player,
                            uploaded: true, // all players we can see have uploaded
                        }
                    }
                    potential_player @ PotentialPlayer::RegisteredOnly(_, _) => {
                        UploadingPlayer {
                            potential_player,
                            uploaded: false, // all players we can't see have not uploaded
                        }
                    }
                }
            })
            .collect();

        StartedStateDetails::Uploading(UploadingState {
            uploading_players: uploaded_players_detail,
        })
    } else {
        let total_mins_remaining = game_data.turn_timer / (1000 * 60);
        let hours_remaining = total_mins_remaining / 60;
        let mins_remaining = total_mins_remaining - hours_remaining * 60;
        StartedStateDetails::Playing(PlayingState {
            players: player_details,
            mins_remaining,
            hours_remaining,
            turn: game_data.turn as u32, // game_data >= 0 checked above
        })
    };

    let started_details = StartedDetails {
        address: started_state.address.clone(),
        game_name: game_data.game_name.clone(),
        state: state_details,
    };

    Ok(GameDetails {
        alias: alias.to_owned(),
        owner: option_lobby_state.map(|lobby_state| lobby_state.owner.clone()),
        description: option_lobby_state.and_then(|lobby_state| lobby_state.description.clone()),
        nations: NationDetails::Started(started_details),
        cache_entry: Some(CacheEntry {
            game_data: game_data.clone(),
            option_snek_state: option_snek_details.clone(),
        }),
    })
}

/// Takes data from the three possible sources, and matches them together until we know
/// 1) who is in the game and the bot's record
/// 2) who is in the game but not the bot
/// 3) who is NOT in the game but is in the bot
fn join_players_with_nations(
    // from game
    nations: &Vec<Nation>,
    // from db
    players_nations: &Vec<(Player, BotNationIdentifier)>,
) -> Result<Vec<PotentialPlayer>, CommandError> {
    let mut potential_players = vec![];

    // Any players registered in the bot with a number go here
    let mut players_by_nation_id = HashMap::new();
    for (player, nation_identifier) in players_nations {
        if let Some(nation_id) = nation_identifier.id() {
            players_by_nation_id.insert(nation_id, (player, nation_identifier));
        } else {
            // The nation_identifier has no ID so it's a custom name so it's bot only
            potential_players.push(PotentialPlayer::RegisteredOnly(
                player.discord_user_id,
                nation_identifier.clone(),
            ));
        }
    }
    for nation in nations {
        match players_by_nation_id.remove(&nation.identifier.id()) {
            // Lobby and game
            Some((player, _)) => {
                let player_details = PlayerDetails {
                    nation_identifier: nation.identifier.clone(),
                    submitted: nation.submitted,
                    player_status: nation.status,
                };
                potential_players.push(PotentialPlayer::RegisteredAndGame(
                    player.discord_user_id,
                    player_details,
                ))
            }
            // Game only
            None => potential_players.push(PotentialPlayer::GameOnly(PlayerDetails {
                nation_identifier: nation.identifier.clone(),
                submitted: nation.submitted,
                player_status: nation.status,
            })),
        }
    }
    // Lobby only RegisteredOnly(UserId, BotNationIdentifier),
    for (_, (player, bot_nation_identifier)) in players_by_nation_id {
        potential_players.push(PotentialPlayer::RegisteredOnly(
            player.discord_user_id,
            bot_nation_identifier.clone(),
        ));
    }
    Ok(potential_players)
}
