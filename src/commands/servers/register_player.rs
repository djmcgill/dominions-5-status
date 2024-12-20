use crate::commands::servers::CommandResponse;
use crate::{
    commands::servers::{alias_from_arg_or_channel_name, details::started_details_from_server},
    db::{DbConnection, DbConnectionKey},
    model::{
        enums::*,
        game_server::GameServerState,
        game_state::*,
        nation::{BotNationIdentifier, GameNationIdentifier},
        player::Player,
    },
    snek::SnekGameStatus,
    DetailsCacheHandle,
};
use anyhow::anyhow;
use either::Either;
use log::*;
use serenity::model::id::ChannelId;
use serenity::{
    framework::standard::{Args, CommandError},
    model::id::UserId,
    prelude::Context,
};
use std::{borrow::Cow, cmp::Ordering, str::FromStr, sync::Arc};

// Find an uploaded/playing nation
fn get_nation_for_started_server(
    arg_nation: Either<&str, u32>,
    started_state_details: &StartedStateDetails,
    era: Option<Era>,
    option_snek_state: Option<&SnekGameStatus>,
    dom_version: u8,
) -> Result<GameNationIdentifier, CommandError> {
    match arg_nation {
        Either::Left(arg_nation_name) => {
            let sanitised_name = sanitise_text(Cow::Owned(arg_nation_name.to_lowercase()));

            match started_state_details {
                StartedStateDetails::Playing(playing_state) => {
                    // FIXME: should this be PlayerDetails instead?
                    let mut possible_ingame_nations: Vec<&PotentialPlayer> = vec![];

                    for potential_player in &playing_state.players {
                        let nation_name = potential_player.nation_name(option_snek_state);

                        let sanitised_nation_name =
                            sanitise_text(Cow::Owned(nation_name.to_lowercase()));
                        if sanitised_nation_name.starts_with(sanitised_name.as_ref()) {
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
                        let sanitised_nation_name = sanitise_text(Cow::Owned(
                            potential_player
                                .nation_name(option_snek_state)
                                .to_lowercase(),
                        ));
                        if sanitised_nation_name.starts_with(sanitised_name.as_ref()) {
                            possible_ingame_nations.push(potential_player);
                        }
                    }
                    let possible_ingame_nations = possible_ingame_nations;
                    match possible_ingame_nations.len() {
                        // Could not find nation. Try again with base nations.
                        0 => {
                            let possible_base_nations = match dom_version {
                                5 => Nations::from_name_prefix(arg_nation_name, era),
                                6 => Nations::from_name_prefix_6(arg_nation_name, era),
                                _ => return Err(CommandError::from(format!("Dom {} lol", dom_version))),
                            };
                            match possible_base_nations.len() {
                                0 => Err(CommandError::from(format!("Could not find nation starting with \"{}\"", arg_nation_name))),
                                1 => Ok(GameNationIdentifier::Existing(possible_base_nations[0])),
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
                let potential_nation = match dom_version {
                    5 => GameNationIdentifier::from_id(arg_nation_id),
                    6 => GameNationIdentifier::from_id_6(arg_nation_id),
                    _ => return Err(CommandError::from(format!("Dom {} lol", dom_version))),
                };

                let already_registered =
                    uploading_state
                        .uploading_players
                        .iter()
                        .find(|uploading_player| {
                            let unregistered = matches!(
                                uploading_player.potential_player,
                                PotentialPlayer::GameOnly(_)
                            );

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
    dom_version: u8,
) -> Result<GameNationIdentifier, CommandError> {
    match arg_nation {
        Either::Left(arg_nation_name) => {
            let nations = match dom_version {
                5 => Nations::from_name_prefix(arg_nation_name, Some(era)),
                6 => Nations::from_name_prefix_6(arg_nation_name, Some(era)),
                _ => return Err(CommandError::from(format!("Dom {} lol", dom_version))),
            };
            match nations.len().cmp(&1) {
                Ordering::Greater => Err(CommandError::from(format!(
                    "ambiguous nation name: {}",
                    arg_nation_name
                ))),
                Ordering::Less => {
                    // try to parse the name as a number
                    let mk_err = || {
                        CommandError::from(format!("Could not find nation: {}. Use register-custom or register-id for mod nations", arg_nation_name))
                    };
                    u32::from_str(arg_nation_name)
                        .map_err(|_| mk_err())
                        .and_then(|id| match dom_version {
                            5 => Ok(GameNationIdentifier::from_id(id)),
                            6 => Ok(GameNationIdentifier::from_id_6(id)),
                            _ => Err(CommandError::from(format!("Dom {} lol", dom_version))),
                        })
                }
                Ordering::Equal => Ok(GameNationIdentifier::Existing(
                    *nations.first().ok_or_else(|| anyhow!("nation vec empty"))?,
                )),
            }
        }
        Either::Right(arg_nation_id) => Ok(match dom_version {
            5 => GameNationIdentifier::from_id(arg_nation_id),
            6 => GameNationIdentifier::from_id_6(arg_nation_id),
            _ => return Err(CommandError::from(format!("Dom {} lol", dom_version))),
        }),
    }
}

async fn register_custom_helper(
    user_id: UserId,
    arg_custom_nation: String,
    alias: String,
    db_conn: DbConnection,
) -> Result<String, CommandError> {
    info!(
        "Registering player {} for custom nation {} in game {}",
        user_id, arg_custom_nation, alias
    );

    if arg_custom_nation.len() >= 100 {
        return Err("Come now, let's not be silly. Use a shorter nation name.".into());
    }

    let server = db_conn.game_for_alias(&alias).map_err(CommandError::from)?;

    match server.state {
        GameServerState::Lobby(lobby_state)
        | GameServerState::StartedState(_, Some(lobby_state)) => {
            let players_nations = db_conn.players_with_nations_for_game_alias(&alias)?;
            if players_nations.len() as i32 >= lobby_state.player_count {
                return Err(CommandError::from("lobby already full"));
            };
            if players_nations
                .iter()
                .any(|(p, _)| p.discord_user_id == user_id)
            {
                return Err(CommandError::from(
                    "you are already registered for this game",
                ));
            }

            let register_message = format!(
                "Registered {}. Until Illwinter fix modded nation status, you will always show up as having not submitted and will still get pings when you go AI unless you unregister.",
                arg_custom_nation,
            );
            let nation = BotNationIdentifier::CustomName(arg_custom_nation);
            let player = Player {
                discord_user_id: user_id,
                turn_notifications: true,
            };
            db_conn
                .insert_player_into_server(&player, &server.alias, nation)
                .map_err(CommandError::from)?;
            Ok(register_message)
        }
        GameServerState::StartedState(_, None) => {
            Err("You cannot use \"register-custom\" during after uploads have started unless the game was a lobby".into())
        }
    }
}

async fn register_player_helper(
    user_id: UserId,
    arg_nation: Either<&str, u32>,
    alias: &str,
    db_conn: DbConnection,
    details_read_handle: DetailsCacheHandle,
) -> Result<String, CommandError> {
    info!(
        "Registering player {} for nation {} in game {}",
        user_id, arg_nation, alias
    );
    let server = db_conn.game_for_alias(alias).map_err(CommandError::from)?;

    match server.state {
        GameServerState::Lobby(lobby_state) => {
            let players_nations = db_conn.players_with_nations_for_game_alias(alias)?;
            if players_nations.len() as i32 >= lobby_state.player_count {
                return Err(CommandError::from("lobby already full"));
            };
            if players_nations
                .iter()
                .any(|(p, _)| p.discord_user_id == user_id)
            {
                return Err(CommandError::from(
                    "you are already registered for this game",
                ));
            }

            let nation = get_nation_for_lobby(arg_nation, lobby_state.era, server.dom_version)?;

            if players_nations.iter().any(|(_, player_nation_id)| {
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
            Ok(format!("Registered {}", nation.name(None),))
        }
        GameServerState::StartedState(started_state, option_lobby_state) => {
            let option_lobby_state_ref = &option_lobby_state;
            let started_db_conn = db_conn.clone();
            let (started_details, option_snek_state) = details_read_handle
                .get_clone(alias)
                .await
                .map_err(CommandError::from)
                .and_then(move |cache| {
                    let game_details: GameDetails = started_details_from_server(
                        started_db_conn,
                        &started_state,
                        option_lobby_state_ref.as_ref(),
                        alias,
                        &cache.game_data,
                        cache.option_snek_state.as_ref(),
                    )?;
                    Ok(game_details)
                })
                .map_err(CommandError::from)
                .and_then(|game_details| match game_details.nations {
                    NationDetails::Lobby(_) => Err(CommandError::from("Somehow found lobby details in a started server? This should never happen!!!")),
                    NationDetails::Started(started_details) => Ok((
                        started_details.state,
                        game_details.cache_entry.and_then(|i| i.option_snek_state),
                    )),
                })?;

            if started_details
                .uploaded_players()
                .any(|registered_user_id| registered_user_id == user_id)
            {
                return Err(CommandError::from(
                    "you are already registered for this game",
                ));
            }

            let option_era = option_lobby_state_ref
                .as_ref()
                .map(|lobby_state| lobby_state.era);
            let nation = get_nation_for_started_server(
                arg_nation,
                &started_details,
                option_era,
                option_snek_state.as_ref(),
                server.dom_version,
            )?;
            let player = Player {
                discord_user_id: user_id,
                turn_notifications: true,
            };
            db_conn
                .insert_player_into_server(&player, &server.alias, nation.clone().into())
                .map_err(CommandError::from)?;
            let text = format!("Registered {}", nation.name(option_snek_state.as_ref()),);
            Ok(text)
        }
    }
}

pub async fn register_player_id(
    context: &Context,
    channel_id: ChannelId,
    user_id: UserId,
    mut args: Args,
) -> Result<CommandResponse, CommandError> {
    let arg_nation_id: u32 = args.single_quoted::<u32>()?;
    if arg_nation_id >= i32::MAX as u32 {
        return Err(format!(
            "Nation ID {} too large. Your hilarious joke will have to be less than 2^31-1.",
            arg_nation_id
        )
        .into());
    }
    let alias = alias_from_arg_or_channel_name(context, channel_id, &mut args).await?;
    let details_read_handle = DetailsCacheHandle(Arc::clone(&context.data));

    let db_conn = {
        let data = context.data.read().await;
        data.get::<DbConnectionKey>()
            .ok_or("no db connection")?
            .clone()
    };

    let reply = register_player_helper(
        user_id,
        Either::Right(arg_nation_id),
        &alias,
        db_conn,
        details_read_handle,
    )
    .await?;
    Ok(CommandResponse::Reply(reply))
}

pub async fn register_player_custom(
    context: &Context,
    channel_id: ChannelId,
    user_id: UserId,
    mut args: Args,
) -> Result<CommandResponse, CommandError> {
    let arg_nation_name: String = args.single_quoted::<String>()?;
    let alias = alias_from_arg_or_channel_name(context, channel_id, &mut args).await?;

    let db_conn = {
        let data = context.data.read().await;
        data.get::<DbConnectionKey>()
            .ok_or("no db connection")?
            .clone()
    };

    let reply = register_custom_helper(user_id, arg_nation_name, alias, db_conn).await?;
    Ok(CommandResponse::Reply(reply))
}

pub async fn register_player(
    context: &Context,
    channel_id: ChannelId,
    user_id: UserId,
    mut args: Args,
) -> Result<CommandResponse, CommandError> {
    let arg_nation_name: String = args.single_quoted::<String>()?.to_lowercase();
    let alias = alias_from_arg_or_channel_name(context, channel_id, &mut args).await?;

    let details_read_handle = DetailsCacheHandle(Arc::clone(&context.data));
    let db_conn = {
        let data = context.data.read().await;
        data.get::<DbConnectionKey>()
            .ok_or("no db connection")?
            .clone()
    };

    let reply = register_player_helper(
        user_id,
        Either::Left(&arg_nation_name),
        &alias,
        db_conn,
        details_read_handle,
    )
    .await?;
    Ok(CommandResponse::Reply(reply))
}
