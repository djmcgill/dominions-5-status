use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::channel::Message;
use serenity::model::id::UserId;

use server::ServerConnection;
use model::{GameServerState, Player};
use model::enums::*;
use db::{DbConnection, DbConnectionKey};
use model::Nation as StartedServerNation;
use super::alias_from_arg_or_channel_name;

fn get_nation_for_started_server(
    option_arg_nation_name: Option<&str>, // should be an either
    option_arg_nation_id: Option<u32>,
    game_nations: &[StartedServerNation],
    pre_game: bool
) -> Result<Nation, CommandError> {
    match (option_arg_nation_name, option_arg_nation_id) {
        (Some(arg_nation_name), None) => {
            // TODO: allow for players with registered nation but not ingame (not yet uploaded)
            let nations = game_nations
                .iter()
                .filter(|&nation| // TODO: more efficient algo
                    nation.name.to_lowercase().starts_with(arg_nation_name))
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
        (None, Some(arg_nation_id)) => {
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
        _ => Err(CommandError::from("Internal server error: get_nation_for_lobby malformed args")),
    }
}

fn get_nation_for_lobby(
    option_arg_nation_name: Option<&str>, // should be an either
    option_arg_nation_id: Option<u32>,
    era: Era,
) -> Result<Nation, CommandError> {
    match (option_arg_nation_name, option_arg_nation_id) {
        (Some(arg_nation_name), None) => {
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
        (None, Some(arg_nation_id)) => {
            Nations::from_id(arg_nation_id)
                .filter(|ref nation| nation.era == era)
                .ok_or(CommandError::from(
                    format!("Could not find nation with id: {} and era: {}",
                        arg_nation_id, era
                    )
                ))
        },
        _ => Err(CommandError::from("Internal server error: get_nation_for_lobby malformed args")),
    }
}

fn register_player_helper<C: ServerConnection>(
    user_id: UserId,
    arg_nation_name: Option<&str>,
    arg_nation_id: Option<u32>,
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

            let nation = get_nation_for_lobby(arg_nation_name, arg_nation_id, lobby_state.era)?;

           if players_nations
                .iter()
                .find(|&&(_, player_nation_id)| {
                    player_nation_id == nation.id as usize
                })
                .is_some()
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
                .insert_server_player(&server.alias, &user_id, nation.id)
                .map_err(CommandError::from)?;
            message.reply(&format!(
                "registering {} {} ({}) for {}",
                nation.era,
                nation.name,
                nation.id,
                user_id.get()?
            ))?;
            Ok(())
        }
        GameServerState::StartedState(started_state, _) => {
            let data = C::get_game_data(&started_state.address)?;

            let nation = get_nation_for_started_server(
                arg_nation_name,
                arg_nation_id,
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
                .insert_server_player(&server.alias, &user_id, nation.id as u32)
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
        None,
        Some(arg_nation_id),
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
        Some(&arg_nation_name),
        None,
        &alias,
        db_conn,
        message,
    )?;
    Ok(())
}
