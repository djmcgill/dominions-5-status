use serenity::model::channel::Message;
use serenity::prelude::Context;
use serenity::{
    framework::standard::{macros::command, Args, CommandError},
    CacheAndHttp,
};

use super::alias_from_arg_or_channel_name;
use crate::db::{DbConnection, DbConnectionKey};
use crate::model::{GameServer, GameServerState, StartedState};
use crate::server::ServerConnection;
use log::*;

#[cfg(test)]
mod tests;

fn add_server_helper<C: ServerConnection>(
    server_address: &str,
    game_alias: &str,
    db_connection: &DbConnection,
) -> Result<(), CommandError> {
    let game_data = C::get_game_data(server_address)?;

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

pub fn add_server<C: ServerConnection>(
    context: &mut Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let server_address = args.single_quoted::<String>()?;

    let alias = alias_from_arg_or_channel_name(&mut args, &message)?;

    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
        ));
    }

    let data = context.data.read();
    let db_connection = data
        .get::<DbConnectionKey>()
        .ok_or("No DbConnection was created on startup. This is a bug.")?;
    add_server_helper::<C>(&server_address, &alias, db_connection)?;
    let text = format!("Successfully inserted with alias {}", alias);
    let _ = message.reply(CacheAndHttp::default(), &text);
    info!("{}", text);
    Ok(())
}
