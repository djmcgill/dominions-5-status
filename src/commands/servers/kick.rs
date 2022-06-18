use log::*;
use serenity::{
    framework::standard::{Args, CommandError},
    model::id::{ChannelId, UserId},
    prelude::Context,
};

use super::alias_from_arg_or_channel_name;
use crate::{
    commands::servers::CommandResponse,
    db::{DbConnection, DbConnectionKey},
};

fn kick_helper(user_id: UserId, alias: &str, db_conn: DbConnection) -> Result<(), CommandError> {
    let rows_affected = db_conn
        .remove_player_from_game(alias, user_id)
        .map_err(CommandError::from)?;

    if rows_affected > 0 {
        Ok(())
    } else {
        return Err(format!("User is not in game {}", alias).into());
    }
}

pub async fn kick_player(
    context: &Context,
    channel_id: ChannelId,
    _user_id: UserId,
    mut args: Args,
) -> Result<CommandResponse, CommandError> {
    let target_user_id = UserId::from(args.single_quoted::<u64>()?);

    let alias = alias_from_arg_or_channel_name(context, channel_id, &mut args).await?;
    let db_conn = {
        let data = context.data.read().await;
        data.get::<DbConnectionKey>()
            .ok_or("No db connection")?
            .clone()
    };
    kick_helper(target_user_id, &alias, db_conn)?;

    let text = format!("Kicked from all nations in game {}", alias);
    info!("{}", text);
    Ok(CommandResponse::Reply(text))
}
