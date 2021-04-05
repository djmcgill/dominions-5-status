use serenity::{
    framework::standard::{Args, CommandError},
    model::channel::Message,
    prelude::Context,
};

use crate::{
    commands::servers::alias_from_arg_or_channel_name,
    db::{DbConnection, DbConnectionKey},
    model::game_server::{GameServer, GameServerState, StartedState},
    server::get_game_data_async,
};
use log::*;

async fn add_server_helper(
    server_address: &str,
    game_alias: &str,
    db_connection: &DbConnection,
) -> Result<(), CommandError> {
    let game_data = get_game_data_async(server_address).await?;
    // FIXME: add to cache
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
    Ok(())
}

pub async fn add_server(
    context: &Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    info!("Adding server for {} with args {:?}", message.author, args);
    let server_address = args.single_quoted::<String>()?;

    let alias = alias_from_arg_or_channel_name(&mut args, &message, context).await?;

    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
        ));
    }

    let data = context.data.read().await;
    let db_connection = data
        .get::<DbConnectionKey>()
        .ok_or("No DbConnection was created on startup. This is a bug.")?;
    add_server_helper(&server_address, &alias, db_connection).await?;
    let text = format!("Successfully inserted with alias {}", alias);
    message
        .reply((&context.cache, context.http.as_ref()), text)
        .await?;
    Ok(())
}
