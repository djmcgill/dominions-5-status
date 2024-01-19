use anyhow::anyhow;
use serenity::{
    framework::standard::{Args, CommandError},
    prelude::Context,
};
use std::sync::Arc;

use crate::commands::servers::CommandResponse;
use crate::{
    commands::servers::alias_from_arg_or_channel_name, db::*, model::game_server::GameServerState,
    DetailsCacheHandle, DetailsCacheKey,
};
use serenity::model::id::{ChannelId, UserId};

async fn unstart_helper(
    db_conn: DbConnection,
    handle: DetailsCacheHandle,
    alias: &str,
) -> Result<(), CommandError> {
    let server = db_conn.game_for_alias(alias)?;

    match server.state {
        GameServerState::StartedState(_, _) => {
            db_conn.remove_started_state(alias)?;
            let mut guard = handle.0.write().await;
            let write_handle = guard.get_mut::<DetailsCacheKey>().ok_or_else(|| {
                anyhow!("Cache somehow not initialised, this should never happen!!")
            })?;
            let _ = write_handle.remove(alias);
        }
        GameServerState::Lobby(_) => {
            return Err(CommandError::from("cannot use this command on a lobby"))
        }
    }
    Ok(())
}

pub async fn unstart(
    context: &Context,
    channel_id: ChannelId,
    _user_id: UserId,
    mut args: Args,
) -> Result<CommandResponse, CommandError> {
    let data_handle = DetailsCacheHandle(Arc::clone(&context.data));

    let db_conn = {
        let data = context.data.read().await;
        data.get::<DbConnectionKey>()
            .ok_or("No DbConnection was created on startup. This is a bug.")?
            .clone()
    };
    let alias = alias_from_arg_or_channel_name(context, channel_id, &mut args).await?;
    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
        ));
    }
    unstart_helper(db_conn, data_handle, &alias).await?;
    Ok(CommandResponse::Reply(format!(
        "Successfully turned '{}' back into a lobby",
        alias
    )))
}
