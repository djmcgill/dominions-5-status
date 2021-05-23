use log::*;
use serenity::framework::standard::{Args, CommandError};
use serenity::model::id::{ChannelId, UserId};
use serenity::prelude::Context;

use super::alias_from_arg_or_channel_name;
use crate::commands::servers::CommandResponse;
use crate::db::{DbConnection, DbConnectionKey};

fn unregister_player_helper(
    user_id: UserId,
    alias: &str,
    db_conn: DbConnection,
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

pub async fn unregister_player(
    context: &Context,
    channel_id: ChannelId,
    user_id: UserId,
    mut args: Args,
) -> Result<CommandResponse, CommandError> {
    let alias = alias_from_arg_or_channel_name(context, channel_id, &mut args).await?;
    let db_conn = {
        let data = context.data.read().await;
        data.get::<DbConnectionKey>()
            .ok_or("No db connection")?
            .clone()
    };
    unregister_player_helper(user_id, &alias, db_conn)?;

    let text = format!("Removed from all nations in game {}", alias);
    info!("{}", text);
    Ok(CommandResponse::Reply(text))
}
