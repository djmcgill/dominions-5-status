use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::Message;

use server::get_game_data;
use model::player::Player;
use model::enums::nations;
use db::DbConnectionKey;

pub fn show_registered(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    println!("show_registered");
    let alias = args.single::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?;
    
    let mut text = String::new(); 
    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or_else(|| "no db connection")?;
    for (_, player, nation_id) in db_conn.players_with_nations_for_game_alias(&alias).map_err(CommandError::from)? {
        let &(nation_name, era) = nations::get_nation_desc(nation_id);
        text.push_str(&format!(
            "{}: {} ({})\n",
            player.discord_user_id.get()?.name,
            nation_name,
            era));
    }

    println!("replying with {}", text);
    let _ = message.reply(&text);
    Ok(())
}

pub fn unregister_player(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    println!("unregistering player");
    let alias = args.single::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?;

    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or("No db connection")?;
    let ref user = message.author;
    let _ = db_conn.remove_player_from_game(&alias, user.id).map_err(CommandError::from)?;
    let text = format!("Removing user {} from game {}", user.name, alias);
    println!("{}", text);
    let _ = message.reply(&text);
    Ok(())
}

pub fn register_player(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    println!("registering player");
    let arg_nation_name = args.single::<String>()?.to_lowercase();   
    let alias = args.single::<String>().or_else(|_| {
        message.channel_id.name().ok_or("")
    })?;

    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or("no db connection")?;
    let server = db_conn.game_for_alias(&alias).map_err(CommandError::from)?;
    let data = get_game_data(&server.address)?;

    let nation = data.nations.iter().find(|&nation| // TODO: more efficient algo
        nation.name.to_lowercase().starts_with(&arg_nation_name) 
    ).ok_or_else(|| {
        let err = format!("Could not find nation starting with {}", arg_nation_name);
        println!("{}", err);
        err
    })?; 

    let player = Player {
        discord_user_id: message.author.id,
    }; 

    // TODO: transaction
    db_conn.insert_player(&player).map_err(CommandError::from)?;
    println!("{} {} {}", server.alias, message.author.id, nation.id as u32);
    db_conn.insert_server_player(&server.alias, &message.author.id, nation.id as u32).map_err(CommandError::from)?;

    let text = format!("registering nation {} for user {} in game {}", nation.name, message.author.name, data.game_name);
    
    let _ = message.reply(&text);
    Ok(())
}
