use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::Message;

use model::{GameServer, GameServerState};
use db::DbConnectionKey;

pub fn lobby(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    let alias = args.single_quoted::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?;
    
    let data = context.data.lock();
    let db_connection = data.get::<DbConnectionKey>().ok_or("No DbConnection was created on startup. This is a bug.")?;
    db_connection.insert_game_server(&GameServer {
        alias: alias.clone(),
        state: GameServerState::Lobby, 
    })?;
    message.reply(&format!("Creating game lobby with name {}", alias))?;
    Ok(())
}
