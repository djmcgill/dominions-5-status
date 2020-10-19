use super::alias_from_arg_or_channel_name;

use serenity::model::channel::Message;
use serenity::prelude::Context;
use serenity::{
    framework::standard::{Args, CommandError},
    CacheAndHttp,
};

use crate::db::DbConnectionKey;

pub fn describe(
    context: &mut Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let data = context.data.read();
    let db_conn = data
        .get::<DbConnectionKey>()
        .ok_or("No DbConnection was created on startup. This is a bug.")?;

    let description = args.single_quoted::<String>()?;
    let alias = alias_from_arg_or_channel_name(&mut args, &message)?;
    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: the description needs to be in quotes",
        ));
    }

    db_conn.update_lobby_with_description(&alias, &description)?;
    message.reply(
        CacheAndHttp::default(),
        &format!("added description to {}", alias),
    )?;
    Ok(())
}
