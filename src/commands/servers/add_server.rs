use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::Message;

use server::get_game_data;
use model::game_server::GameServer;
use db::DbConnectionKey;

pub fn add_server(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    let server_address = args.single_quoted::<String>()?;
    
    let game_data = get_game_data(&server_address)?;
    
    let alias = args.single_quoted::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?;

    if !args.is_empty() {
        return Err(CommandError::from("Too many arguments. TIP: spaces in arguments need to be quoted \"like this\""));
    }

    let mut data = context.data.lock();
    let db_connection = data.get_mut::<DbConnectionKey>().ok_or("No DbConnection was created on startup. This is a bug.")?;

    let server = GameServer {
        address: server_address,
        alias: alias.clone(),
        last_seen_turn: game_data.turn
    };

    db_connection.insert_game_server(&server)?;
    let text = format!("successfully inserted game {} with alias {}", game_data.game_name, alias);
    let _ = message.reply(&text);
    println!("{}", text);
    Ok(())
}
