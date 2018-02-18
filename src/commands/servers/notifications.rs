use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::{Message, UserId};

use db::*;

fn notifications_helper(
    db_conn: &DbConnection,
    player_id: UserId,
    desired_turn_notifications: bool,
) -> Result<(), CommandError> {
    db_conn.set_turn_notifications(player_id, desired_turn_notifications)?;
    Ok(())
}

pub fn notifications(
    context: &mut Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let desired_turn_notifications = args.single_quoted::<bool>()?;
    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or("no db connection")?;

    notifications_helper(db_conn, message.author.id, desired_turn_notifications)?;
    message.reply(&format!(
        "Set turn notifications to {}",
        desired_turn_notifications
    ))?;
    Ok(())
}
