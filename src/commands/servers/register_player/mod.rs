use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::channel::Message;
use serenity::model::id::UserId;

use crate::server::ServerConnection;
use crate::model::{GameServerState, Player};
use crate::model::enums::*;
use crate::db::{DbConnection, DbConnectionKey};
use crate::model::Nation as StartedServerNation;
use super::alias_from_arg_or_channel_name;
use either::Either;

fn get_nation_for_started_server(
    arg_nation: Either<&str, u32>,
    game_nations: &[StartedServerNation],
    pre_game: bool
) -> Result<Nation, CommandError> {
    match arg_nation {
        Either::Left(arg_nation_name) => {
            let sanitised_name = arg_nation_name.to_lowercase().replace("'", "").replace(" ", "");
            // TODO: allow for players with registered nation but not ingame (not yet uploaded)
            let nations = game_nations
                .iter()
                .filter(|&nation| { // TODO: more efficient algo

                    let found_nation_name = nation.name.to_lowercase().replace("'", "").replace(" ", "");
                    found_nation_name.starts_with(&sanitised_name)
                })
                .map ( |game_nation| {
                    let game_nation_era =
                        // if we can't parse the db era things are messed up
                        Era::from_string(&game_nation.era).unwrap();
                    Nation {
                        id: game_nation.id as u32,
                        name: game_nation.name.to_owned(),
                        era: game_nation_era,
                    }
                })
                .collect::<Vec<_>>();

            let nations_len = nations.len();
            if nations_len > 1 {
                return Err(CommandError::from(
                    format!("ambiguous nation name: {}", arg_nation_name),
                ));
            } else if nations_len < 1 {
                let error = if pre_game {
                    format!("Could not find nation starting with {}. Make sure you've uploaded a pretender first"
                            , arg_nation_name)
                } else {
                    format!("Could not find nation starting with {}", arg_nation_name)
                };
                return Err(CommandError::from(error));
            };
            Ok(nations[0].clone())
        }
        Either::Right(arg_nation_id) =>
            if pre_game {
                let era: Era = if game_nations.is_empty() {
                    None
                } else {
                    Era::from_string(&game_nations[0].era)
                }.unwrap_or(Era::Early);
                Ok(Nations::from_id(arg_nation_id)
                    .unwrap_or(
                        Nation {
                            id: arg_nation_id,
                            name: "Unknown Nation".to_string(),
                            era,
                        }
                    )
                )
            } else {
                game_nations
                    .iter()
                    .find(|&nation| // TODO: more efficient algo
                        nation.id as u32 == arg_nation_id)
                    .map ( |game_nation| {
                        let game_nation_era =
                        // if we can't parse the db era things are messed up
                            Era::from_string(&game_nation.era).unwrap();
                        Nation {
                            id: game_nation.id as u32,
                            name: game_nation.name.to_owned(),
                            era: game_nation_era,
                        }
                    })
                    .ok_or(CommandError::from(format!("Could not find a nation with id {}", arg_nation_id)))
        }
    }
}

fn get_nation_for_lobby(
    arg_nation: Either<&str, u32>,
    era: Era,
) -> Result<Nation, CommandError> {
    match arg_nation {
        Either::Left(arg_nation_name) => {
            let nations = Nations::from_name_prefix(arg_nation_name, Some(era));
            let nations_len = nations.len();
            if nations_len > 1 {
                return Err(CommandError::from(
                    format!("ambiguous nation name: {}", arg_nation_name),
                ));
            } else if nations_len < 1 {
                return Err(CommandError::from(
                    format!("could not find nation: {}", arg_nation_name),
                ));
            };
            Ok(nations[0].clone())
        },
        Either::Right(arg_nation_id) => {
            Ok(Nations::from_id(arg_nation_id)
                .filter(|ref nation| nation.era == era)
                .unwrap_or(
                    Nation {
                        id: arg_nation_id,
                        name: "Unknown Nation".to_string(),
                        era,
                    }
                )
            )
        },
    }
}

fn register_player_helper<C: ServerConnection>(
    user_id: UserId,
    arg_nation: Either<&str, u32>,
    alias: &str,
    db_conn: &DbConnection,
    message: &Message,
) -> Result<(), CommandError> {
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
                .any(|&(_, player_nation_id)| {
                    player_nation_id == nation.id as usize
                })
            {
                return Err(CommandError::from(
                    format!("Nation {} already exists in lobby", nation.name),
                ));
            }
            let player = Player {
                discord_user_id: user_id,
                turn_notifications: true,
            };
            // TODO: transaction
            db_conn.insert_player(&player).map_err(CommandError::from)?;
            db_conn
                .insert_server_player(&server.alias, user_id, nation.id)
                .map_err(CommandError::from)?;
            message.reply(&format!(
                "registering {} {} ({}) for {}",
                nation.era,
                nation.name,
                nation.id,
                user_id.to_user()?
            ))?;
            Ok(())
        }
        GameServerState::StartedState(started_state, _) => {
            let data = C::get_game_data(&started_state.address)?;

            let nation = get_nation_for_started_server(
                arg_nation,
                &data.nations[..],
                data.turn == -1,
            )?;
            let player = Player {
                discord_user_id: user_id,
                turn_notifications: true,
            };

            // TODO: transaction
            db_conn.insert_player(&player).map_err(CommandError::from)?;
            info!("{} {} {}", server.alias, user_id, nation.id as u32);
            db_conn
                .insert_server_player(&server.alias, user_id, nation.id as u32)
                .map_err(CommandError::from)?;
            let text = format!(
                "registering nation {} ({}) for user {}",
                nation.name,
                nation.id,
                message.author
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
    let alias = alias_from_arg_or_channel_name(&mut args, &message)?;

    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or("no db connection")?;

    register_player_helper::<C>(
        message.author.id,
        Either::Right(arg_nation_id),
        &alias,
        db_conn,
        message,
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

    register_player_helper::<C>(
        message.author.id,
        Either::Left(&arg_nation_name),
        &alias,
        db_conn,
        message,
    )?;
    Ok(())
}
