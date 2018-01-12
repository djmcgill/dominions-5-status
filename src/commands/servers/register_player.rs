use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::{Message, UserId};

use server::get_game_data;
use model::nation::Nation;
use model::player::Player;
use db::{DbConnection, DbConnectionKey};

fn unregister_player_helper(user_id: UserId, alias: &str, db_conn: &DbConnection) -> Result<(), CommandError> {
    db_conn.remove_player_from_game(&alias, user_id).map_err(CommandError::from)?;
    Ok(())
}

pub fn unregister_player(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    let alias = args.single_quoted::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?;
    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or("No db connection")?;
    unregister_player_helper(message.author.id, &alias, &db_conn)?;

    let text = format!("Removing user {} from game {}", message.author.name, alias);
    info!("{}", text);
    let _ = message.reply(&text);
    Ok(())
}

fn register_player_helper(user_id: UserId, arg_nation_name: &str, alias: &str, db_conn: &DbConnection) -> Result<Nation, CommandError> {
    let server = db_conn.game_for_alias(&alias).map_err(CommandError::from)?;
    let data = get_game_data(&server.address)?;

    let nation = data.nations.iter().find(|&nation| // TODO: more efficient algo
        nation.name.to_lowercase().starts_with(&arg_nation_name) 
    ).ok_or_else(|| {
        let err = format!("Could not find nation starting with {}", arg_nation_name);
        info!("{}", err);
        err
    })?; 

    let player = Player {
        discord_user_id: user_id,
    }; 

    // TODO: transaction
    db_conn.insert_player(&player).map_err(CommandError::from)?;
    info!("{} {} {}", server.alias, user_id, nation.id as u32);
    db_conn.insert_server_player(&server.alias, &user_id, nation.id as u32).map_err(CommandError::from)?;

    Ok(nation.clone())   
}

pub fn register_player(context: &mut Context, message: &Message, args: Args) -> Result<(), CommandError> {
    let args = args.multiple_quoted::<String>()?;
    if args.len() > 2 {
        return Err(CommandError::from("Too many arguments. TIP: spaces in arguments need to be quoted \"like this\""));
    }

    let arg_nation_name = args[0].to_lowercase();   
    let alias = args.get(1).cloned().or_else(|| {
        message.channel_id.name()
    }).ok_or(&"Could not retrieve channel name")?;

    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or("no db connection")?;

    let nation = register_player_helper(message.author.id, &arg_nation_name, &alias, &db_conn)?;
    let text = format!("registering nation {} for user {}", nation.name, message.author.name);
    let _ = message.reply(&text);
    Ok(())
}
