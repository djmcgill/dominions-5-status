use log::*;
use serenity::framework::standard::{Args, CommandError};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::prelude::Context;
use std::str::FromStr;

use super::alias_from_arg_or_channel_name;
use crate::commands::servers::*;
use crate::db::{DbConnection, DbConnectionKey};
use crate::model::enums::*;
use crate::model::{GameServerState, Player};
use crate::server::ServerConnection;
use either::Either;

fn get_nation_for_started_server(
    arg_nation: Either<&str, u32>,
    started_state_details: &StartedStateDetails,
    era: Option<Era>,
) -> Result<Nation, CommandError> {
    match arg_nation {
        Either::Left(arg_nation_name) => {
            let sanitised_name = arg_nation_name
                .to_lowercase()
                .replace("'", "")
                .replace(" ", "");

            match started_state_details {
                StartedStateDetails::Playing(playing_state) => {
                    let mut possible_ingame_nations: Vec<&PotentialPlayer> = vec![];
                    for potential_player in &playing_state.players {
                        let sanitised_nation_name = potential_player
                            .nation_name()
                            .to_lowercase()
                            .replace("'", "")
                            .replace(" ", "");
                        if sanitised_nation_name.starts_with(&sanitised_name) {
                            possible_ingame_nations.push(potential_player);
                        }
                    }
                    let possible_ingame_nations = possible_ingame_nations;
                    match possible_ingame_nations.len() {
                        // Could not find nation. Error.
                        0 => Err(CommandError::from(format!("Could not find nation starting with \"{}\"", arg_nation_name))),
                        // Found nation!
                        1 => {
                            let found_nation = possible_ingame_nations[0];
                            let nation = Nation {
                                id: found_nation.nation_id(),
                                name: found_nation.nation_name().clone(),
                                era: None,
                            };
                            Ok(nation)
                        },
                        // Ambiguous nation. Error.
                        _ => Err(CommandError::from(format!(
                            "Found more than one nation starting with \"{}\". Consider using !register-id if the name is ambiguous.",
                            arg_nation_name
                        ))),
                    }
                }
                StartedStateDetails::Uploading(uploading_state) => {
                    let mut possible_ingame_nations: Vec<&PotentialPlayer> = vec![];
                    for uploading_player in &uploading_state.uploading_players {
                        let potential_player = &uploading_player.potential_player;
                        let sanitised_nation_name = potential_player
                            .nation_name()
                            .to_lowercase()
                            .replace("'", "")
                            .replace(" ", "");
                        if sanitised_nation_name.starts_with(&sanitised_name) {
                            possible_ingame_nations.push(potential_player);
                        }
                    }
                    let possible_ingame_nations = possible_ingame_nations;
                    match possible_ingame_nations.len() {
                        // Could not find nation. Try again with base nations.
                        0 => {
                            let possible_base_nations = Nations::from_name_prefix(arg_nation_name, era);
                            match possible_base_nations.len() {
                                0 => Err(CommandError::from(format!("Could not find nation starting with \"{}\"", arg_nation_name))),
                                1 => Ok(possible_base_nations[0].clone()),
                                _ => Err(CommandError::from(format!(
                                    "Found more than one nation starting with \"{}\". Consider using !register-id if the name is ambiguous.",
                                    arg_nation_name
                                ))),
                            }
                        },


                        // Found nation!
                        1 => {
                            let found_nation = possible_ingame_nations[0];
                            let nation = Nation {
                                id: found_nation.nation_id(),
                                name: found_nation.nation_name().clone(),
                                era: None,
                            };
                            Ok(nation)
                        },
                        // Ambiguous nation. Error.
                        _ => Err(CommandError::from(format!(
                            "Found more than one nation starting with \"{}\". Consider using !register-id if the name is ambiguous.",
                            arg_nation_name
                        ))),

                    }
                }
            }
        }
        Either::Right(arg_nation_id) => match started_state_details {
            StartedStateDetails::Uploading(uploading_state) => {
                let option_uploaded_nation = uploading_state
                    .uploading_players
                    .iter()
                    .find(|uploading_player| {
                        uploading_player.potential_player.nation_id() == arg_nation_id
                    })
                    .map(|uploading_player| Nation {
                        id: arg_nation_id,
                        name: uploading_player.nation_name().clone(),
                        era,
                    });
                option_uploaded_nation
                    .or_else(|| Nations::from_id(arg_nation_id))
                    .ok_or(CommandError::from(format!(
                        "Could not find nation with ID \"{}\"",
                        arg_nation_id
                    )))
            }
            StartedStateDetails::Playing(playing_state) => playing_state
                .players
                .iter()
                .find(|potential_player| potential_player.nation_id() == arg_nation_id)
                .map(|potential_player| Nation {
                    id: arg_nation_id,
                    name: potential_player.nation_name().clone(),
                    era,
                })
                .ok_or(CommandError::from(format!(
                    "Could not find nation in game with ID \"{}\"",
                    arg_nation_id
                ))),
        },
    }
}

fn get_nation_for_lobby(arg_nation: Either<&str, u32>, era: Era) -> Result<Nation, CommandError> {
    match arg_nation {
        Either::Left(arg_nation_name) => {
            let nations = Nations::from_name_prefix(arg_nation_name, Some(era));
            let nations_len = nations.len();
            if nations_len > 1 {
                return Err(CommandError::from(format!(
                    "ambiguous nation name: {}",
                    arg_nation_name
                )));
            } else if nations_len < 1 {
                // try to parse the name as a number
                let mk_err =
                    || CommandError::from(format!("could not find nation: {}", arg_nation_name));
                return u32::from_str(arg_nation_name)
                    .map_err(|_| mk_err())
                    .and_then(|arg_nation_id| Nations::from_id(arg_nation_id).ok_or_else(mk_err));
            };
            Ok(nations[0].clone())
        }
        Either::Right(arg_nation_id) => Ok(Nations::from_id(arg_nation_id).unwrap_or(Nation {
            id: arg_nation_id,
            name: "Unknown Nation".to_string(),
            era: Some(era),
        })),
    }
}

fn register_player_helper<C: ServerConnection>(
    user_id: UserId,
    arg_nation: Either<&str, u32>,
    alias: &str,
    db_conn: &DbConnection,
    message: &Message,
    details_read_handle: &crate::CacheReadHandle,
) -> Result<(), CommandError> {
    info!(
        "Registering player {} for nation {} in game {}",
        user_id, arg_nation, alias
    );
    let server = db_conn.game_for_alias(&alias).map_err(CommandError::from)?;

    match server.state {
        GameServerState::Lobby(lobby_state) => {
            let players_nations = db_conn.players_with_nations_for_game_alias(&alias)?;
            if players_nations.len() as i32 >= lobby_state.player_count {
                return Err(CommandError::from("lobby already full"));
            };

            let nation = get_nation_for_lobby(arg_nation, lobby_state.era)?;

            if players_nations
                .iter()
                .any(|&(_, player_nation_id)| player_nation_id == nation.id)
            {
                return Err(CommandError::from(format!(
                    "Nation {} already exists in lobby",
                    nation.name
                )));
            }
            let player = Player {
                discord_user_id: user_id,
                turn_notifications: true,
            };
            db_conn
                .insert_player_into_server(&player, &server.alias, nation.id)
                .map_err(CommandError::from)?;
            message.reply(&format!(
                "registering {} ({}) for {}",
                nation.name,
                nation.id,
                user_id.to_user()?
            ))?;
            Ok(())
        }
        GameServerState::StartedState(started_state, option_lobby_state) => {
            let started_details = details_read_handle
                .get_clone(alias)
                .and_then(|option| option)
                .map(|cache| {
                    let game_details: GameDetails = started_details_from_server(
                        db_conn,
                        &started_state,
                        option_lobby_state.as_ref(),
                        alias,
                        cache.game_data,
                        cache.option_snek_state,
                    )
                    .unwrap();
                    game_details
                })
                .and_then(|game_details| match game_details.nations {
                    NationDetails::Lobby(_) => None,
                    NationDetails::Started(started_details) => Some(started_details.state),
                })
                .ok_or(CommandError::from(
                    "Could not find game cache something is wrong",
                ))?;
            let option_era = option_lobby_state.map(|lobby_state| lobby_state.era);
            let nation = get_nation_for_started_server(arg_nation, &started_details, option_era)?;
            let player = Player {
                discord_user_id: user_id,
                turn_notifications: true,
            };
            db_conn
                .insert_player_into_server(&player, &server.alias, nation.id)
                .map_err(CommandError::from)?;
            let text = format!(
                "registering nation {} ({}) for user {}",
                nation.name, nation.id, message.author
            );
            let _ = message.reply(&text);
            Ok(())
        }
    }
}

pub fn register_player_id<C: ServerConnection>(
    context: &mut Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let arg_nation_id: u32 = args.single_quoted::<u32>()?;
    if arg_nation_id >= std::i32::MAX as u32 {
        return Err(format!("Nation ID {} too large. Your hilarious joke will have to be less than 2^32.", arg_nation_id).into());
    }
    let alias = alias_from_arg_or_channel_name(&mut args, &message)?;

    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or("no db connection")?;
    let details_read_handle = data
        .get::<crate::DetailsReadHandleKey>()
        .ok_or("No details cache")?;

    register_player_helper::<C>(
        message.author.id,
        Either::Right(arg_nation_id),
        &alias,
        db_conn,
        message,
        details_read_handle,
    )?;
    Ok(())
}

pub fn register_player<C: ServerConnection>(
    context: &mut Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let arg_nation_name: String = args.single_quoted::<String>()?.to_lowercase();
    let alias = alias_from_arg_or_channel_name(&mut args, &message)?;
    // FIXME: no idea why this isn't working
    //    if args.len() != 0 {
    //        return Err(CommandError::from(
    //            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
    //        ));
    //    }

    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or("no db connection")?;
    let details_read_handle = data
        .get::<crate::DetailsReadHandleKey>()
        .ok_or("No details cache")?;

    register_player_helper::<C>(
        message.author.id,
        Either::Left(&arg_nation_name),
        &alias,
        db_conn,
        message,
        details_read_handle,
    )?;
    Ok(())
}
