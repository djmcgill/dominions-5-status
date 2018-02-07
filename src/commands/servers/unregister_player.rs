use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::{Message, UserId};

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
    unregister_player_helper(message.author.id, &alias, db_conn)?;

    let text = format!("Removing user {} from all nations in game {}", message.author, alias);
    info!("{}", text);
    let _ = message.reply(&text);
    Ok(())
}
