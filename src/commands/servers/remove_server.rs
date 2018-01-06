use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::Message;

use db::DbConnectionKey;
pub fn remove_server(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    let alias = args.single::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?;

    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().unwrap();
    db_conn.remove_server(&alias).unwrap();
    let _ = message.reply(&format!("successfully removed server {}", alias));
    Ok(())
}
