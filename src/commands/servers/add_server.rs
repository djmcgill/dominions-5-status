use crate::commands::servers::CommandResponse;
use crate::{
    commands::servers::alias_from_arg_or_channel_name,
    db::{DbConnection, DbConnectionKey},
    model::{
        game_server::{GameServer, GameServerState, StartedState},
        game_state::CacheEntry,
    },
    server::get_game_data_async,
    snek::snek_details_async,
    DetailsCacheHandle, DetailsCacheKey,
};
use chrono::Utc;
use log::*;
use serenity::{
    framework::standard::{Args, CommandError},
    model::id::{ChannelId, UserId},
    prelude::Context,
};
use std::sync::Arc;

async fn add_server_helper(
    server_address: &str,
    game_alias: &str,
    db_connection: DbConnection,
    write_handle_mutex: DetailsCacheHandle,
) -> Result<(), CommandError> {
    let game_data = get_game_data_async(server_address).await?;
    let option_snek_state = snek_details_async(server_address).await?;
    let server = GameServer {
        alias: game_alias.to_string(),
        state: GameServerState::StartedState(
            StartedState {
                address: server_address.to_string(),
                last_seen_turn: game_data.turn,
            },
            None,
        ),
    };

    db_connection.insert_game_server(&server).map_err(|e| {
        if e.to_string()
            .contains("UNIQUE constraint failed: game_servers.alias")
        {
            CommandError::from(format!(
                "A game called '{}' already exists, if you are starting a lobby use !start",
                game_alias
            ))
        } else {
            CommandError::from(e)
        }
    })?;

    let cache_entry = CacheEntry {
        game_data,
        option_snek_state,
    };
    let mut guard = write_handle_mutex.0.write().await;
    match guard.get_mut::<DetailsCacheKey>() {
        Some(write_handle) => {
            write_handle.insert(
                game_alias.to_owned(),
                Box::new((Utc::now(), Some(cache_entry))),
            );
        }
        None => {
            error!("Cache somehow not initialised, this should never happen!!");
        }
    }

    Ok(())
}

pub async fn add_server(
    context: &Context,
    channel_id: ChannelId,
    user_id: UserId,
    mut args: Args,
) -> Result<CommandResponse, CommandError> {
    info!("Adding server for {} with args {:?}", user_id, args);
    let server_address = args.single_quoted::<String>()?;

    let alias = alias_from_arg_or_channel_name(context, channel_id, &mut args).await?;

    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
        ));
    }

    let write_handle_mutex = DetailsCacheHandle(Arc::clone(&context.data));
    let db_connection = {
        let data = context.data.read().await;
        data.get::<DbConnectionKey>()
            .ok_or("No DbConnection was created on startup. This is a bug.")?
            .clone()
    };
    add_server_helper(&server_address, &alias, db_connection, write_handle_mutex).await?;
    let text = format!("Successfully inserted with alias {}", alias);
    Ok(CommandResponse::Reply(text))
}
