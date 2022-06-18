use crate::{
    commands::servers::{alias_from_arg_or_channel_name, CommandResponse},
    db::DbConnectionKey,
    DetailsCacheKey,
};
use serenity::{
    framework::standard::{Args, CommandError},
    model::id::{ChannelId, UserId},
    prelude::Context,
};

pub async fn server_set_alias(
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

    let old_alias = args.single_quoted::<String>()?;
    let new_alias = alias_from_arg_or_channel_name(context, channel_id, &mut args).await?;
    if !args.is_empty() {
        return Err(CommandError::from("Too many arguments."));
    }

    db_conn.update_lobby_with_alias(&old_alias, &new_alias)?;

    let reply = format!("Updated alias from {} to {}", old_alias, new_alias,);

    {
        let mut data = context.data.write().await;
        let cache_handle = data
            .get_mut::<DetailsCacheKey>()
            .ok_or("No DetailsCache was created on startup. This is a bug.")?;

        if let Some(old_entry) = cache_handle.remove(&old_alias) {
            if let Some(_overwritten_entry) = cache_handle.insert(new_alias, old_entry) {
                return Err(CommandError::from(
                    "Overwrote existing game entry, this should never happen due to db unique constraint",
                ));
            }
        }
    }

    Ok(CommandResponse::Reply(reply))
}
