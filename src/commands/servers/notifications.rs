use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::Message;

use db::DbConnectionKey;

pub fn notifications(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    let desired_turn_notifications = args.single_quoted::<bool>()?;
    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or("no db connection")?;
    db_conn.set_turn_notifications(message.author.id, desired_turn_notifications)?;
    message.reply(&format!("Set turn notifications to {}", desired_turn_notifications))?;
    Ok(())
}
