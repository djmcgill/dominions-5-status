use super::alias_from_arg_or_channel_name;

use serenity::framework::standard::{Args, CommandError};
use serenity::model::channel::Message;
use serenity::prelude::Context;

use crate::db::DbConnectionKey;

pub async fn describe(
    context: &Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let db_conn = {
        let data = context.data.read().await;
        data.get::<DbConnectionKey>()
            .ok_or("No DbConnection was created on startup. This is a bug.")?
            .clone()
    };

    let description = args.single_quoted::<String>()?;
    let alias = alias_from_arg_or_channel_name(&mut args, &message, context).await?;
    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: the description needs to be in quotes",
        ));
    }

    db_conn.update_lobby_with_description(&alias, &description)?;
    message
        .reply(
            (&context.cache, context.http.as_ref()),
            &format!("added description to {}", alias),
        )
        .await?;
    Ok(())
}
