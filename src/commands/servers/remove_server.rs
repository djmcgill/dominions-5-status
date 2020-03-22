use serenity::framework::standard::{Args, CommandError};
use serenity::model::channel::Message;
use serenity::prelude::Context;

use super::alias_from_arg_or_channel_name;
use crate::db::*;

fn remove_server_helper(db_conn: &DbConnection, alias: &str) -> Result<(), CommandError> {
    db_conn.remove_server(&alias).map_err(CommandError::from)?;
    Ok(())
}

pub async fn remove_server(
    context: &Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let alias = alias_from_arg_or_channel_name(&mut args, &message, context).await?;
    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
        ));
    }

    let data = context.data.read().await;
    let db_conn = data.get::<DbConnectionKey>().ok_or("No DB connection")?;
    remove_server_helper(db_conn, &alias)?;
    let _ = message
        .reply(
            (&context.cache, context.http.as_ref()),
            &format!("successfully removed server {}", alias),
        )
        .await;
    Ok(())
}
