use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;

use super::alias_from_arg_or_channel_name;
use crate::commands::servers::CommandResponse;
use crate::db::*;
use crate::{DetailsCacheHandle, DetailsCacheKey};
use log::error;
use serenity::model::id::{ChannelId, UserId};
use std::sync::Arc;

async fn remove_server_helper(
    details_cache_handle: DetailsCacheHandle,
    db_conn: DbConnection,
    alias: &str,
) -> Result<(), CommandError> {
    // Okay there is a bit of a race condition here, where if the turn check gets the alias,
    // then this runs and deletes it from the db and cache, and then the turn check finishes
    // it'll re-add it to the cache BUT that only means that there's a now-useless cache entry
    // that is ignored so I'm going to go with: don't care.
    db_conn.remove_server(alias).map_err(CommandError::from)?;
    let mut guard = details_cache_handle.0.write().await;
    match guard.get_mut::<DetailsCacheKey>() {
        Some(handle) => {
            handle.remove(alias);
            Ok(())
        }
        None => {
            error!("Cache somehow not initialised, this should never happen!!");
            Err("Cache somehow not initialised, this should never happen!!".into())
        }
    }
}

pub async fn remove_server(
    context: &Context,
    channel_id: ChannelId,
    _user_id: UserId,
    mut args: Args,
) -> Result<CommandResponse, CommandError> {
    let alias = alias_from_arg_or_channel_name(context, channel_id, &mut args).await?;
    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
        ));
    }

    let write_handle_mutex = DetailsCacheHandle(Arc::clone(&context.data));
    let db_conn = {
        let data = context.data.read().await;
        data.get::<DbConnectionKey>()
            .ok_or("No DB connection")?
            .clone()
    };
    remove_server_helper(write_handle_mutex, db_conn, &alias).await?;
    Ok(CommandResponse::Reply(format!(
        "successfully removed server {}",
        alias
    )))
}
