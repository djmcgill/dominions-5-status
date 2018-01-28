use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::{Message, UserId};

use server::get_game_data;
use model::{Player, GameServerState};
use model::enums::*;
use db::{DbConnection, DbConnectionKey};

fn unregister_player_helper(user_id: UserId, alias: &str, db_conn: &DbConnection) -> Result<(), CommandError> {
    db_conn.remove_player_from_game(&alias, user_id).map_err(CommandError::from)?;
    Ok(())
}

pub fn unregister_player(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    let alias = args.single_quoted::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?.to_lowercase();
    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or("No db connection")?;
    unregister_player_helper(message.author.id, &alias, &db_conn)?;

    let text = format!("Removing user {} from game {}", message.author.name, alias);
    info!("{}", text);
    let _ = message.reply(&text);
    Ok(())
}

fn register_player_helper(user_id: UserId, arg_nation_name: &str, alias: &str, db_conn: &DbConnection, message: &Message) -> Result<(), CommandError> {
    let server = db_conn.game_for_alias(&alias).map_err(CommandError::from)?;

    match server.state {
        GameServerState::Lobby(lobby_state) => {
            let nations = NATIONS_BY_ID.iter().filter(|&(&_id, &(name, era))| {
                let lname: String = name.to_owned().to_lowercase();
                era == lobby_state.era && lname.starts_with(&arg_nation_name)
            }).collect::<Vec<_>>();
            if nations.len() != 1 {
                return Err(CommandError::from("ambiguous nation name"));
            }
            let (nation_id, _) = nations[0];
            db_conn.insert_server_player(&server.alias, &user_id, *nation_id).map_err(CommandError::from)?;
            message.reply(&"registering")?;
            Ok(())
        }
        GameServerState::StartedState(started_state) => {
            let data = get_game_data(&started_state.address)?;

            // TODO: allow for players with registered nation but not ingame (not yet uploaded)
            let nation = data.nations.iter().find(|&nation| // TODO: more efficient algo
                nation.name.to_lowercase().starts_with(&arg_nation_name) 
            ).ok_or_else(|| {
                let err = if data.turn == -1 {
                    format!("Could not find nation starting with {}. Make sure you've uploaded a pretender first"
                        , arg_nation_name)
                } else {
                    format!("Could not find nation starting with {}", arg_nation_name)
                };
                
                info!("{}", err);
                err
            })?; 

            let player = Player {
                discord_user_id: user_id,
                turn_notifications: true,
            }; 

            // TODO: transaction
            db_conn.insert_player(&player).map_err(CommandError::from)?;
            info!("{} {} {}", server.alias, user_id, nation.id as u32);
            db_conn.insert_server_player(&server.alias, &user_id, nation.id as u32).map_err(CommandError::from)?;
            let text = format!("registering nation {} for user {}", nation.name, message.author.name);
            let _ = message.reply(&text);
            Ok(())
        }
    }  
}

pub fn register_player(context: &mut Context, message: &Message, args: Args) -> Result<(), CommandError> {
    let args = args.multiple_quoted::<String>()?;
    if args.len() > 2 {
        return Err(CommandError::from("Too many arguments. TIP: spaces in arguments need to be quoted \"like this\""));
    }

    let arg_nation_name = args[0].to_lowercase();   
    let alias = args.get(1).cloned().or_else(|| {
        message.channel_id.name()
    }).ok_or(&"Could not retrieve channel name")?.to_lowercase();

    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or("no db connection")?;

    register_player_helper(message.author.id, &arg_nation_name, &alias, &db_conn, message)?;
    Ok(())
}
