use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::Message;

use db::*;

#[cfg(test)]
mod tests;

fn remove_server_helper(db_conn: &DbConnection, alias: &str) -> Result<(), CommandError> {
    db_conn.remove_server(&alias).map_err(CommandError::from)?;
    Ok(())
}

pub fn remove_server(
    context: &mut Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let alias = args.single_quoted::<String>()
        .or_else(|_| {
            message.channel_id.name().ok_or(format!(
                "Could not find channel name for channel {}",
                message.channel_id
            ))
        })?
        .to_lowercase();
    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
        ));
    }

    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or("No DB connection")?;
    remove_server_helper(db_conn, &alias)?;
    let _ = message.reply(&format!("successfully removed server {}", alias));
    Ok(())
}
