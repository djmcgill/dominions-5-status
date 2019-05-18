use super::alias_from_arg_or_channel_name;
use crate::server::ServerConnection;

use serenity::builder::CreateEmbed;
use serenity::framework::standard::{Args, CommandError};
use serenity::model::channel::Message;
use serenity::prelude::Context;

use crate::db::{DbConnection, DbConnectionKey};
use crate::model::enums::{Era, NationStatus, Nations, SubmissionStatus};
use crate::model::{GameServerState, LobbyState, Nation, Player, StartedState};
use log::*;
use serenity::model::id::UserId;
use std::collections::HashMap;
use std::cmp::max;

struct GameDetails {
    alias: String,
    owner: Option<UserId>,
    description: Option<String>,
    nations: NationDetails,
}

enum NationDetails {
    Lobby(LobbyDetails),
    Started(StartedDetails),
}
struct StartedDetails {
    address: String,
    game_name: String,
    state: StartedStateDetails,
}
enum StartedStateDetails {
    Playing(PlayingState),
    Uploading(UploadingState),
}
struct UploadingState {
    uploading_players: Vec<UploadingPlayer>,
}
struct PlayingState {
    players: Vec<PotentialPlayer>,
    turn: u32,
    mins_remaining: i32,
    hours_remaining: i32,
}
enum PotentialPlayer {
    LobbyOnly(UserId, u32),
    LobbyAndGame(UserId, PlayerDetails),
    GameOnly(PlayerDetails),
}
struct PlayerDetails {
    nation_id: u32,
    submitted: SubmissionStatus,
    player_status: NationStatus,
}

struct UploadingPlayer {
    player: Option<UserId>,
    nation: u32,
    uploaded: bool,
}
struct LobbyDetails {
    players: Vec<LobbyPlayer>,
    era: Option<Era>,
    remaining_slots: u32,
}
struct LobbyPlayer {
    player: UserId,
    registered_nation: u32,
}

pub fn details_helper_2<C: ServerConnection>(
    db_conn: &DbConnection,
    alias: &str,
) -> Result<CreateEmbed, CommandError> {
    let server = db_conn.game_for_alias(&alias)?;
    info!("got server details");

    let details = match server.state {
        GameServerState::Lobby(ref lobby_state) =>
            lobby_details_2(db_conn, lobby_state, &alias)?,
        GameServerState::StartedState(ref started_state, ref option_lobby_state) => {
            started_from_lobby_details_2::<C>(
                db_conn,
                started_state,
                option_lobby_state.as_ref(),
                &alias,
            )?
        }
    };

    details_to_embed(details)
}

fn details_to_embed(details: GameDetails) -> Result<CreateEmbed, CommandError> {
    match details.nations {
        NationDetails::Started(started_details) => {
            match started_details.state {
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

                    for potential_player in playing_state.players {
                        match potential_player {
                            // If the game has started and they're not in it, too bad
                            PotentialPlayer::LobbyOnly(_, _) => {}

                            PotentialPlayer::LobbyAndGame(user_id, player_details) => {
                                let &(nation_name, era) = Nations::get_nation_desc(player_details.nation_id as usize);
                                nation_names.push_str(&format!("{} {} ({})\n", era, nation_name, player_details.nation_id));
                                player_names.push_str(&format!("**{}**\n", user_id.to_user()?));
                                if let NationStatus::Human = player_details.player_status {
                                    submitted_status.push_str(&format!("{}\n", player_details.submitted.show()));
                                } else {
                                    submitted_status.push_str(&format!("{}\n", SubmissionStatus::Submitted.show()));
                                }
                            }
                            PotentialPlayer::GameOnly(player_details) => {
                                let &(nation_name, era) = Nations::get_nation_desc(player_details.nation_id as usize);
                                nation_names.push_str(&format!("{} {} ({})\n", era, nation_name, player_details.nation_id));
                                player_names.push_str(&format!("{}\n", player_details.player_status.show()));
                                if let NationStatus::Human = player_details.player_status {
                                    submitted_status.push_str(&format!("{}\n", player_details.submitted.show()));
                                } else {
                                    submitted_status.push_str(&format!("{}\n", SubmissionStatus::Submitted.show()));
                                }
                            }
                        }
                    }

                    let mut e = CreateEmbed::default()
                        .title(embed_title)
                        .field("Nation", nation_names, true)
                        .field("Player", player_names, true)
                        .field("Submitted", submitted_status, true);

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
                StartedStateDetails::Uploading(uploading_state) => {
                    let mut nation_names = String::new();
                    let mut player_names = String::new();
                    let mut submitted_status = String::new();

                    for uploading_player in uploading_state.uploading_players {
                        let &(nation_name, era) = Nations::get_nation_desc(uploading_player.nation as usize);
                        nation_names.push_str(&format!("{} {} ({})\n", era, nation_name, uploading_player.nation));
                        let player_name = match uploading_player.player {
                            Some(user_id) => format!("**{}**\n", user_id.to_user()?),
                            None => format!("{}\n", NationStatus::Human.show())
                        };
                        player_names.push_str(&player_name);
                        let player_submitted_status = if uploading_player.uploaded {
                            format!("{}\n", SubmissionStatus::Submitted.show())
                        } else {
                            format!("{}\n", SubmissionStatus::NotSubmitted.show())
                        };
                        submitted_status.push_str(&player_submitted_status);

                    }

                    let embed_title = format!(
                        "{} ({}): Pretender uploading",
                        started_details.game_name, started_details.address,
                    );

                    let mut e = CreateEmbed::default()
                        .title(embed_title)
                        .field("Nation", nation_names, true)
                        .field("Player", player_names, true)
                        .field("Uploaded", submitted_status, true);

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
            }
        }
        NationDetails::Lobby(lobby_details) => {
            let embed_title = match lobby_details.era {
                Some(era) => format!("{} ({} Lobby)", details.alias, era),
                None => format!("{} (Lobby)", details.alias)
            };

            let mut player_names = String::new();
            let mut nation_names = String::new();

            for lobby_player in lobby_details.players {
                let discord_user = lobby_player.player.to_user()?;
                let &(nation_name, era) = Nations::get_nation_desc(lobby_player.registered_nation as usize);
                player_names.push_str(&format!("{} \n", discord_user));
                nation_names.push_str(&format!("{} {} ({})\n", era, nation_name, lobby_player.registered_nation));
            }

            for _ in 0..lobby_details.remaining_slots {
                player_names.push_str(&".\n");
                nation_names.push_str(&"OPEN\n");
            }
            let mut e = CreateEmbed::default()
                .title(embed_title)
                .field("Nation", nation_names, true)
                .field("Player", player_names, true);

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
    }
}

pub fn details_2<C: ServerConnection>(
    context: &mut Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let data = context.data.lock();
    let db_conn = data
        .get::<DbConnectionKey>()
        .ok_or("No DbConnection was created on startup. This is a bug.")?;
    let alias = alias_from_arg_or_channel_name(&mut args, &message)?;
    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
        ));
    }

    let embed_response = details_helper_2::<C>(db_conn, &alias)?;
    message
        .channel_id
        .send_message(|m| m.embed(|_| embed_response))?;
    Ok(())
}

fn lobby_details_2(
    db_conn: &DbConnection,
    lobby_state: &LobbyState,
    alias: &str,
) -> Result<GameDetails, CommandError> {
    let players_nations = db_conn.players_with_nations_for_game_alias(&alias)?;

    let player_nation_details: Vec<LobbyPlayer> = players_nations
        .into_iter()
        .map(|(player, nation_id)| -> LobbyPlayer {
            LobbyPlayer {
                player: player.discord_user_id,
                registered_nation: nation_id as u32,
            }
        })
        .collect();

    let remaining_slots = max(0, (lobby_state.player_count - player_nation_details.len() as i32) as u32);

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

fn started_from_lobby_details_2<C: ServerConnection>(
    db_conn: &DbConnection,
    started_state: &StartedState,
    option_lobby_state: Option<&LobbyState>,
    alias: &str,
) -> Result<GameDetails, CommandError> {
    let server_address = &started_state.address;
    let mut game_data = C::get_game_data(&server_address)?;
    game_data
        .nations
        .sort_unstable_by(|a, b| a.name.cmp(&b.name));

    let id_player_nations = db_conn.players_with_nations_for_game_alias(&alias)?;
    let player_details = join_players_with_nations(&game_data.nations, &id_player_nations)?;

    let state_details = if game_data.turn < 0 {
        let uploaded_players_detail: Vec<UploadingPlayer> = player_details
            .into_iter()
            .map(|potential_player_detail| {
                match potential_player_detail {
                    PotentialPlayer::GameOnly(player_detail) => {
                        UploadingPlayer {
                            player: None,
                            nation: player_detail.nation_id,
                            uploaded: true, // all players we can see have uploaded
                        }
                    }
                    PotentialPlayer::LobbyAndGame(user_id, player_detail) => {
                        UploadingPlayer {
                            player: Some(user_id),
                            nation: player_detail.nation_id,
                            uploaded: true, // all players we can see have uploaded
                        }
                    }
                    PotentialPlayer::LobbyOnly(user_id, nation_id) => {
                        UploadingPlayer {
                            player: Some(user_id),
                            nation: nation_id as u32,
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
    players_nations: &Vec<(Player, usize)>,
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
                    nation_id: nation.id as u32,
                    submitted: nation.submitted,
                    player_status: nation.status,
                };
                potential_players.push(PotentialPlayer::LobbyAndGame(
                    player.discord_user_id,
                    player_details,
                ))
            }
            // Game only
            None => potential_players.push(PotentialPlayer::GameOnly(PlayerDetails {
                nation_id: nation.id as u32,
                submitted: nation.submitted,
                player_status: nation.status,
            })),
        }
    }
    // Lobby only
    for (nation_id, player) in players_by_nation_id {
        potential_players.push(PotentialPlayer::LobbyOnly(
            player.discord_user_id,
            nation_id as u32,
        ));
    }
    Ok(potential_players)
}
