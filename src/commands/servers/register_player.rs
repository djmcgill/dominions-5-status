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
use crate::model::{BotNationIdentifier, GameNationIdentifier, GameServerState, Player};
use crate::snek::SnekGameStatus;
use either::Either;

// Find an uploaded/playing nation
fn get_nation_for_started_server(
    arg_nation: Either<&str, u32>,
    started_state_details: &StartedStateDetails,
    era: Option<Era>,
    option_snek_state: Option<&SnekGameStatus>,
) -> Result<GameNationIdentifier, CommandError> {
    match arg_nation {
        Either::Left(arg_nation_name) => {
            let sanitised_name = arg_nation_name
                .to_lowercase()
                .replace("'", "")
                .replace(" ", "");

            match started_state_details {
                StartedStateDetails::Playing(playing_state) => {
                    // FIXME: should this be PlayerDetails instead?
                    let mut possible_ingame_nations: Vec<&PotentialPlayer> = vec![];

                    for potential_player in &playing_state.players {
                        let nation_name = potential_player.nation_name(option_snek_state);

                        let sanitised_nation_name =
                            nation_name.to_lowercase().replace("'", "").replace(" ", "");
                        if sanitised_nation_name.starts_with(&sanitised_name) {
                            possible_ingame_nations.push(potential_player);
                        }
                    }
                    let possible_ingame_nations = possible_ingame_nations;
                    match possible_ingame_nations.len() {
                        // Could not find nation. Check if it's a number.
                        0 => {
                            match u32::from_str(arg_nation_name) {
                                Ok(nation_id) => {
                                    if let Some(nation) = &playing_state.players.iter().find(|playing_nation| {
                                        match playing_nation.nation_id() {
                                            Some(playing_id) => playing_id == nation_id,
                                            None => false,
                                        }
                                    }) {
                                        match nation {
                                            PotentialPlayer::GameOnly(player_details) =>
                                                Ok(player_details.nation_identifier.clone()),
                                            PotentialPlayer::RegisteredAndGame(_, _) =>
                                                Err("Nation already registered".into()),
                                            PotentialPlayer::RegisteredOnly(_, _) =>
                                                Err("Nation already registered".into()),
                                        }
                                    } else {
                                        Err(CommandError::from(format!("Could not find nation starting with \"{}\"", arg_nation_name)))
                                    }
                                }
                                Err(_) => {
                                    Err(CommandError::from(format!("Could not find nation starting with \"{}\"", arg_nation_name)))
                                }
                            }
                        }
                        // Found nation!
                        1 => {
                            match possible_ingame_nations[0] {
                                PotentialPlayer::GameOnly(player_details) => Ok(player_details.nation_identifier.clone()),
                                PotentialPlayer::RegisteredAndGame(_, _) =>
                                    Err("Nation already registered".into()),
                                PotentialPlayer::RegisteredOnly(_, _) =>
                                    Err("Nation already registered".into()),
                            }
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
                            .nation_name(option_snek_state)
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
                                1 => Ok(GameNationIdentifier::Existing(possible_base_nations[0].clone())),
                                _ => Err(CommandError::from(format!(
                                    "Found more than one nation starting with \"{}\". Consider using !register-id if the name is ambiguous.",
                                    arg_nation_name
                                ))),
                            }
                        },


                        // Found nation!
                        1 => {
                            let found_nation = possible_ingame_nations[0];
                            match found_nation {
                                PotentialPlayer::GameOnly(player_details) => Ok(player_details.nation_identifier.clone()),
                                PotentialPlayer::RegisteredAndGame(_, _) => Err("Nation already registered".into()),
                                PotentialPlayer::RegisteredOnly(_, _) => Err("Nation already registered".into()),
                            }
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
                let potential_nation = GameNationIdentifier::from_id(arg_nation_id);

                let already_registered =
                    uploading_state
                        .uploading_players
                        .iter()
                        .find(|uploading_player| {
                            let unregistered = if let PotentialPlayer::GameOnly(_) =
                                uploading_player.potential_player
                            {
                                true
                            } else {
                                false
                            };

                            !unregistered
                                && uploading_player.potential_player.nation_id()
                                    == Some(arg_nation_id)
                        });
                if already_registered.is_some() {
                    Err("ID already registered".into())
                } else {
                    Ok(potential_nation)
                }
            }
            // Find a nation that is uploaded but not already registered
            StartedStateDetails::Playing(playing_state) => playing_state
                .players
                .iter()
                .find(|potential_player| potential_player.nation_id() == Some(arg_nation_id))
                .map(|potential_player| match potential_player {
                    PotentialPlayer::GameOnly(player_details) => {
                        Ok(player_details.nation_identifier.clone())
                    }
                    _ => Err("ID already registered".into()),
                })
                .ok_or_else(|| {
                    CommandError::from(format!(
                        "Could not find nation in game with ID \"{}\"",
                        arg_nation_id
                    ))
                })
                .and_then(|i| i),
        },
    }
}

fn get_nation_for_lobby(
    arg_nation: Either<&str, u32>,
    era: Era,
) -> Result<GameNationIdentifier, CommandError> {
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
                let mk_err = || {
                    CommandError::from(format!("Could not find nation: {}. Use register-custom or register-id for mod nations", arg_nation_name))
                };
                return u32::from_str(arg_nation_name)
                    .map_err(|_| mk_err())
                    .map(|arg_nation_id| GameNationIdentifier::from_id(arg_nation_id));
            };
            Ok(GameNationIdentifier::Existing(nations[0].clone()))
        }
        Either::Right(arg_nation_id) => Ok(GameNationIdentifier::from_id(arg_nation_id)),
    }
}

fn register_custom_helper(
    user_id: UserId,
    arg_custom_nation: String,
    alias: String,
    db_conn: &DbConnection,
    message: &Message,
) -> Result<(), CommandError> {
    info!(
        "Registering player {} for custom nation {} in game {}",
        user_id, arg_custom_nation, alias
    );

    if arg_custom_nation.len() >= 100 {
        return Err("Come now, let's not be silly. Please use a shorter nation name.".into());
    }

    let server = db_conn.game_for_alias(&alias).map_err(CommandError::from)?;

    match server.state {
        GameServerState::Lobby(lobby_state) => {
            let players_nations = db_conn.players_with_nations_for_game_alias(&alias)?;
            if players_nations.len() as i32 >= lobby_state.player_count {
                return Err(CommandError::from("lobby already full"));
            };

            let register_message = format!(
                "registering {} for {}. You will have to reregister after uploading.",
                arg_custom_nation,
                user_id.to_user()?
            );
            let nation = BotNationIdentifier::CustomName(arg_custom_nation);
            let player = Player {
                discord_user_id: user_id,
                turn_notifications: true,
            };
            db_conn
                .insert_player_into_server(&player, &server.alias, nation)
                .map_err(CommandError::from)?;
            message.reply(&register_message)?;
            Ok(())
        }
        GameServerState::StartedState(_, _) => {
            Err("You cannot use \"register-custom\" during after uploads have started!".into())
        }
    }
}

fn register_player_helper(
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

            if players_nations.iter().any(|&(_, ref player_nation_id)| {
                let nation_id: BotNationIdentifier = nation.clone().into();
                player_nation_id == &nation_id
            }) {
                return Err(CommandError::from("Nation already exists in lobby"));
            }
            let player = Player {
                discord_user_id: user_id,
                turn_notifications: true,
            };
            db_conn
                .insert_player_into_server(&player, &server.alias, nation.clone().into())
                .map_err(CommandError::from)?;
            message.reply(&format!(
                "registering {} for {}",
                nation.name(None),
                user_id.to_user()?
            ))?;
            Ok(())
        }
        GameServerState::StartedState(started_state, option_lobby_state) => {
            let (started_details, option_snek_state) = details_read_handle
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
                    NationDetails::Started(started_details) => Some((
                        started_details.state,
                        game_details.cache_entry.and_then(|i| i.option_snek_state),
                    )),
                })
                .ok_or(CommandError::from(
                    "Could not find game cache something is wrong",
                ))?;
            let option_era = option_lobby_state.map(|lobby_state| lobby_state.era);
            let nation = get_nation_for_started_server(
                arg_nation,
                &started_details,
                option_era,
                option_snek_state.as_ref(),
            )?;
            let player = Player {
                discord_user_id: user_id,
                turn_notifications: true,
            };
            db_conn
                .insert_player_into_server(&player, &server.alias, nation.clone().into())
                .map_err(CommandError::from)?;
            let text = format!(
                "registering nation {} for user {}",
                nation.name(option_snek_state.as_ref()),
                message.author
            );
            let _ = message.reply(&text);
            Ok(())
        }
    }
}

pub fn register_player_id(
    context: &mut Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let arg_nation_id: u32 = args.single_quoted::<u32>()?;
    if arg_nation_id >= std::i32::MAX as u32 {
        return Err(format!(
            "Nation ID {} too large. Your hilarious joke will have to be less than 2^32.",
            arg_nation_id
        )
        .into());
    }
    let alias = alias_from_arg_or_channel_name(&mut args, &message)?;

    let data = context.data.read();
    let db_conn = data.get::<DbConnectionKey>().ok_or("no db connection")?;
    let details_read_handle = data
        .get::<crate::DetailsReadHandleKey>()
        .ok_or("No details cache")?;

    register_player_helper(
        message.author.id,
        Either::Right(arg_nation_id),
        &alias,
        db_conn,
        message,
        details_read_handle,
    )?;
    Ok(())
}

pub fn register_custom(
    context: &mut Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let arg_nation_name: String = args.single_quoted::<String>()?;
    let alias = alias_from_arg_or_channel_name(&mut args, &message)?;

    let data = context.data.read();
    let db_conn = data.get::<DbConnectionKey>().ok_or("no db connection")?;

    register_custom_helper(message.author.id, arg_nation_name, alias, db_conn, message)?;
    Ok(())
}

pub fn register_player(
    context: &mut Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let arg_nation_name: String = args.single_quoted::<String>()?.to_lowercase();
    let alias = alias_from_arg_or_channel_name(&mut args, &message)?;

    let data = context.data.read();
    let db_conn = data.get::<DbConnectionKey>().ok_or("no db connection")?;
    let details_read_handle = data
        .get::<crate::DetailsReadHandleKey>()
        .ok_or("No details cache")?;

    register_player_helper(
        message.author.id,
        Either::Left(&arg_nation_name),
        &alias,
        db_conn,
        message,
        details_read_handle,
    )?;
    Ok(())
}
