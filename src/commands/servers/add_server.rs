use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::Message;

use server::get_game_data;
use model::GameServer;
use db::{DbConnection, DbConnectionKey};

fn add_server_helper(server_address: &str, game_alias: &str, db_connection: &mut DbConnection) -> Result<(), CommandError> {
    let game_data = get_game_data(server_address)?;

    let server = GameServer {
        address: server_address.to_string(),
        alias: game_alias.to_string(),
        last_seen_turn: game_data.turn
    };

    db_connection.insert_game_server(&server)?;
    Ok(())
}

pub fn add_server(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    let server_address = args.single_quoted::<String>()?;
    
    let alias = args.single_quoted::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?;

    if !args.is_empty() {
        return Err(CommandError::from("Too many arguments. TIP: spaces in arguments need to be quoted \"like this\""));
    }

    let mut data = context.data.lock();
    let mut db_connection = data.get_mut::<DbConnectionKey>().ok_or("No DbConnection was created on startup. This is a bug.")?;
    add_server_helper(&server_address, &alias, &mut db_connection)?;
    let text = format!("Successfully inserted with alias {}", alias);
    let _ = message.reply(&text);
    info!("{}", text);
    Ok(())
}
