use log::*;
use serenity::framework::standard::{Args, CommandError};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::prelude::Context;

use super::alias_from_arg_or_channel_name;
use crate::db::{DbConnection, DbConnectionKey};

fn unregister_player_helper(
    user_id: UserId,
    alias: &str,
    db_conn: &DbConnection,
) -> Result<(), CommandError> {
    let rows_affected = db_conn
        .remove_player_from_game(&alias, user_id)
        .map_err(CommandError::from)?;

    if rows_affected > 0 {
        Ok(())
    } else {
        Err(format!("User is not in game {}", alias))?
    }
}

pub fn unregister_player(
    context: &mut Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let alias = alias_from_arg_or_channel_name(&mut args, &message)?;
    let data = context.data.read();
    let db_conn = data.get::<DbConnectionKey>().ok_or("No db connection")?;
    unregister_player_helper(message.author.id, &alias, db_conn)?;

    let text = format!(
        "Removing user {} from all nations in game {}",
        message.author, alias
    );
    info!("{}", text);
    let _ = message.reply(&text);
    Ok(())
}
