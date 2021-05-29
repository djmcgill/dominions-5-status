use serenity::framework::standard::{Args, CommandError};
use serenity::model::id::{ChannelId, UserId};
use serenity::prelude::Context;

use crate::commands::servers::CommandResponse;
use crate::db::*;

fn notifications_helper(
    db_conn: DbConnection,
    player_id: UserId,
    desired_turn_notifications: bool,
) -> Result<(), CommandError> {
    db_conn.set_turn_notifications(player_id, desired_turn_notifications)?;
    Ok(())
}

pub async fn notifications(
    context: &Context,
    _channel_id: ChannelId,
    user_id: UserId,
    mut args: Args,
) -> Result<CommandResponse, CommandError> {
    let desired_turn_notifications = args.single_quoted::<bool>()?;
    let db_conn = {
        let data = context.data.read().await;
        data.get::<DbConnectionKey>()
            .ok_or("no db connection")?
            .clone()
    };

    notifications_helper(db_conn, user_id, desired_turn_notifications)?;
    Ok(CommandResponse::Reply(format!(
        "Set turn notifications to {}",
        desired_turn_notifications
    )))
}
