use super::alias_from_arg_or_channel_name;
use crate::commands::servers::CommandResponse;
use crate::db::DbConnectionKey;
use serenity::{
    framework::standard::{Args, CommandError},
    model::id::{ChannelId, UserId},
    prelude::Context,
};

pub async fn describe(
    context: &Context,
    channel_id: ChannelId,
    _user_id: UserId,
    mut args: Args,
) -> Result<CommandResponse, CommandError> {
    let db_conn = {
        let data = context.data.read().await;
        data.get::<DbConnectionKey>()
            .ok_or("No DbConnection was created on startup. This is a bug.")?
            .clone()
    };

    let description = args.single_quoted::<String>()?;
    let alias = alias_from_arg_or_channel_name(context, channel_id, &mut args).await?;
    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: the description needs to be in quotes",
        ));
    }

    db_conn.update_lobby_with_description(&alias, &description)?;
    Ok(CommandResponse::Reply(format!(
        "Added description to {}",
        alias
    )))
}
