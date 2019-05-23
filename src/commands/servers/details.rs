use super::alias_from_arg_or_channel_name;
use crate::server::ServerConnection;

use serenity::builder::CreateEmbed;
use serenity::framework::standard::{Args, CommandError};
use serenity::model::channel::Message;
use serenity::prelude::Context;

use crate::db::{DbConnection, DbConnectionKey};
use crate::model::enums::{Era, NationStatus, Nations, SubmissionStatus};
use crate::model::{GameServerState, LobbyState, Nation, Player, StartedState};
use crate::snek::SnekGameStatus;
use log::*;
use serenity::model::id::UserId;
use std::cmp::max;
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Clone)]
pub struct GameDetails {
    pub alias: String,
    pub owner: Option<UserId>,
    pub description: Option<String>,
    pub nations: NationDetails,
}

#[derive(PartialEq, Eq, Clone)]
pub enum NationDetails {
    Lobby(LobbyDetails),
    Started(StartedDetails),
}
#[derive(PartialEq, Eq, Clone)]
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

#[derive(PartialEq, Eq, Clone)]
pub enum StartedStateDetails {
    Playing(PlayingState),
    Uploading(UploadingState),
}
#[derive(PartialEq, Eq, Clone)]
pub struct UploadingState {
    pub uploading_players: Vec<UploadingPlayer>,
}
#[derive(PartialEq, Eq, Clone)]
pub struct PlayingState {
    pub players: Vec<PotentialPlayer>,
    pub turn: u32,
    pub mins_remaining: i32,
    pub hours_remaining: i32,
}
#[derive(PartialEq, Eq, Clone)]
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
#[derive(PartialEq, Eq, Clone)]
pub struct PlayerDetails {
    pub nation_id: u32,
    pub nation_name: String,
    pub submitted: SubmissionStatus,
    pub player_status: NationStatus,
}
#[derive(PartialEq, Eq, Clone)]
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
#[derive(PartialEq, Eq, Clone)]
pub struct LobbyDetails {
    pub players: Vec<LobbyPlayer>,
    pub era: Option<Era>,
    pub remaining_slots: u32,
}
#[derive(PartialEq, Eq, Clone)]
pub struct LobbyPlayer {
    pub player_id: UserId,
    pub nation_id: u32,
    pub nation_name: String,
}

pub fn details<C: ServerConnection>(
    context: &mut Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let data = context.data.lock();
    let db_conn = data
        .get::<DbConnectionKey>()
        .ok_or("No DbConnection was created on startup. This is a bug.")?;

    let read_handle = data
        .get::<crate::DetailsReadHandleKey>()
        .ok_or("No ReadHandle was created on startup. This is a bug.")?;

    let alias = alias_from_arg_or_channel_name(&mut args, &message)?;
    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
        ));
    }
    let embed_response = details_helper(&alias, db_conn, read_handle)?;

    message
        .channel_id
        .send_message(|m| m.embed(|_| embed_response))?;
    Ok(())
}

fn details_helper(
    alias: &str,
    db_conn: &DbConnection,
    read_handle: &crate::ReadHandle,
) -> Result<CreateEmbed, CommandError> {
    let option_option_game_details = read_handle.handle().get_and(alias, |values| {
        if values.len() != 1 {
            panic!()
        } else {
            (*values[0]).1.clone()
        }
    });

    match option_option_game_details {
        Some(Some(details)) => details_to_embed(details),
        Some(None) => Err(CommandError::from("Failed to connect to the server.")),
        None => {
            if db_conn
                .retrieve_all_servers()?
                .into_iter()
                .find(|server| &server.alias == alias)
                .is_some()
            {
                Err(CommandError::from(
                    "Server starting up, please try again in 1 min.",
                ))
            } else {
                Err(CommandError::from(format!(
                    "Game with alias '{}' not found.",
                    alias
                )))
            }
        }
    }
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

fn details_to_embed(details: GameDetails) -> Result<CreateEmbed, CommandError> {
    let mut e = match details.nations {
        NationDetails::Started(started_details) => {
            match &started_details.state {
                StartedStateDetails::Playing(playing_state) => {
                    let embed_title = format!(
                        "{} ({}): turn {}, {}h {}m remaining",
                        started_details.game_name,
                        started_details.address,
                        playing_state.turn,
                        playing_state.hours_remaining,
                        playing_state.mins_remaining
                    );

                    let mut nation_names = String::new();
                    let mut player_names = String::new();
                    let mut submitted_status = String::new();

                    for potential_player in &playing_state.players {
                        let (option_user_id, player_details) = match potential_player {
                            // If the game has started and they're not in it, too bad
                            PotentialPlayer::RegisteredOnly(_, _, _) => continue,
                            PotentialPlayer::RegisteredAndGame(user_id, player_details) => {
                                (Some(user_id), player_details)
                            }
                            PotentialPlayer::GameOnly(player_details) => (None, player_details),
                        };

                        nation_names.push_str(&format!(
                            "{} ({})\n",
                            player_details.nation_name, player_details.nation_id
                        ));

                        if let NationStatus::Human = player_details.player_status {
                            match option_user_id {
                                Some(user_id) => {
                                    player_names.push_str(&format!("**{}**\n", user_id.to_user()?))
                                }
                                None => player_names.push_str(&format!(
                                    "{}\n",
                                    player_details.player_status.show()
                                )),
                            }
                            submitted_status
                                .push_str(&format!("{}\n", player_details.submitted.show()));
                        } else {
                            player_names
                                .push_str(&format!("{}\n", player_details.player_status.show()));
                            submitted_status
                                .push_str(&format!("{}\n", SubmissionStatus::Submitted.show()));
                        }
                    }

                    CreateEmbed::default()
                        .title(embed_title)
                        .field("Nation", nation_names, true)
                        .field("Player", player_names, true)
                        .field("Submitted", submitted_status, true)
                }
                StartedStateDetails::Uploading(uploading_state) => {
                    let embed_title = format!(
                        "{} ({}): Pretender uploading",
                        started_details.game_name, started_details.address,
                    );

                    let mut nation_names = String::new();
                    let mut player_names = String::new();
                    let mut submitted_status = String::new();

                    for uploading_player in &uploading_state.uploading_players {
                        nation_names.push_str(&format!(
                            "{} ({})\n",
                            uploading_player.nation_name(),
                            uploading_player.nation_id()
                        ));

                        let player_name = match uploading_player.option_player_id() {
                            Some(user_id) => format!("**{}**\n", user_id.to_user()?),
                            None => format!("{}\n", NationStatus::Human.show()),
                        };
                        player_names.push_str(&player_name);
                        let player_submitted_status = if uploading_player.uploaded {
                            format!("{}\n", SubmissionStatus::Submitted.show())
                        } else {
                            format!("{}\n", SubmissionStatus::NotSubmitted.show())
                        };
                        submitted_status.push_str(&player_submitted_status);
                    }

                    CreateEmbed::default()
                        .title(embed_title)
                        .field("Nation", nation_names, true)
                        .field("Player", player_names, true)
                        .field("Uploaded", submitted_status, true)
                }
            }
        }
        NationDetails::Lobby(lobby_details) => {
            let embed_title = match lobby_details.era {
                Some(era) => format!("{} ({} Lobby)", details.alias, era),
                None => format!("{} (Lobby)", details.alias),
            };

            let mut player_names = String::new();
            let mut nation_names = String::new();

            for lobby_player in lobby_details.players {
                let discord_user = lobby_player.player_id.to_user()?;
                player_names.push_str(&format!("{} \n", discord_user));
                nation_names.push_str(&format!(
                    "{} ({})\n",
                    lobby_player.nation_name, lobby_player.nation_id
                ));
            }

            for _ in 0..lobby_details.remaining_slots {
                player_names.push_str(&".\n");
                nation_names.push_str(&"OPEN\n");
            }
            CreateEmbed::default()
                .title(embed_title)
                .field("Nation", nation_names, true)
                .field("Player", player_names, true)
        }
    };
    for owner in details.owner {
        e = e.field("Owner", owner.to_user()?.to_string(), false);
    }

    for description in details.description {
        if !description.is_empty() {
            e = e.field("Description", description, false);
        }
    }
    Ok(e)
}

fn lobby_details(
    db_conn: &DbConnection,
    lobby_state: &LobbyState,
    alias: &str,
) -> Result<GameDetails, CommandError> {
    let players_nations = db_conn.players_with_nations_for_game_alias(&alias)?;

    let mut player_nation_details: Vec<LobbyPlayer> = players_nations
        .into_iter()
        .map(|(player, nation_id)| -> LobbyPlayer {
            let &(nation_name, _) = Nations::get_nation_desc(nation_id);
            LobbyPlayer {
                player_id: player.discord_user_id,
                nation_id,
                nation_name: nation_name.to_owned(),
            }
        })
        .collect();
    player_nation_details.sort_by(|n1, n2| n1.nation_name.cmp(&n2.nation_name));

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

    let id_player_nations = db_conn.players_with_nations_for_game_alias(&alias)?;
    let option_snek_details = C::get_snek_data(server_address)?;
    let player_details =
        join_players_with_nations(&game_data.nations, &id_player_nations, &option_snek_details)?;

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
                    potential_player @ PotentialPlayer::RegisteredOnly(_, _, _) => {
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
        address: server_address.clone(),
        game_name: game_data.game_name.clone(),
        state: state_details,
    };

    Ok(GameDetails {
        alias: alias.to_owned(),
        owner: option_lobby_state.map(|lobby_state| lobby_state.owner.clone()),
        description: option_lobby_state.and_then(|lobby_state| lobby_state.description.clone()),
        nations: NationDetails::Started(started_details),
    })
}

fn join_players_with_nations(
    nations: &Vec<Nation>,
    players_nations: &Vec<(Player, u32)>,
    option_snek_details: &Option<SnekGameStatus>,
) -> Result<Vec<PotentialPlayer>, CommandError> {
    let mut potential_players = vec![];

    let mut players_by_nation_id = HashMap::new();
    for (player, nation_id) in players_nations {
        players_by_nation_id.insert(*nation_id, player);
    }
    for nation in nations {
        match players_by_nation_id.remove(&nation.id) {
            // Lobby and game
            Some(player) => {
                let player_details = PlayerDetails {
                    nation_id: nation.id,
                    nation_name: get_nation_string(option_snek_details, nation.id),
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
                nation_id: nation.id,
                nation_name: get_nation_string(option_snek_details, nation.id),
                submitted: nation.submitted,
                player_status: nation.status,
            })),
        }
    }
    // Lobby only
    for (nation_id, player) in players_by_nation_id {
        let &(nation_name, _) = Nations::get_nation_desc(nation_id);
        potential_players.push(PotentialPlayer::RegisteredOnly(
            player.discord_user_id,
            nation_id,
            nation_name.to_owned(),
        ));
    }
    potential_players.sort_unstable();
    Ok(potential_players)
}
