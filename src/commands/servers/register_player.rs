use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::{Message, UserId};

use server::get_game_data;
use model::{Player, GameServerState};
use model::enums::*;
use db::{DbConnection, DbConnectionKey};

fn register_player_helper(user_id: UserId, arg_nation_name: &str, alias: &str, db_conn: &DbConnection, message: &Message) -> Result<(), CommandError> {
    let server = db_conn.game_for_alias(&alias).map_err(CommandError::from)?;

    match server.state {
        GameServerState::Lobby(lobby_state) => {
            let players_nations = db_conn.players_with_nations_for_game_alias(&alias)?;
            if players_nations.len() as i32 >= lobby_state.player_count {
                return Err(CommandError::from("lobby already full"));
            };

            let nations = NATIONS_BY_ID.iter().filter(|&(&_id, &(name, era))| {
                let lname: String = name.to_owned().to_lowercase();
                era == lobby_state.era && lname.starts_with(&arg_nation_name)
            }).collect::<Vec<_>>();
            let nations_len = nations.len();
            if nations_len > 1 {
                return Err(CommandError::from("ambiguous nation name"));
            } else if nations_len < 1 {
                return Err(CommandError::from("could not find nation"));
            };
            let (&nation_id, &(nation_name, nation_era)) = nations[0];
            if players_nations.iter().find(|&&(_, player_nation_id)| player_nation_id == nation_id as usize).is_some() {
                return Err(CommandError::from(format!("Nation {} already exists in lobby", nation_name)));
            }
            db_conn.insert_server_player(&server.alias, &user_id, nation_id).map_err(CommandError::from)?;
            message.reply(&format!("registering {} {} for {}", nation_era, nation_name, user_id.get()?))?;
            Ok(())
        }
        GameServerState::StartedState(started_state, _) => {
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
            let text = format!("registering nation {} for user {}", nation.name, message.author);
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
