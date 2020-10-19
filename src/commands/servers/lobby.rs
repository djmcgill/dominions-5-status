use serenity::framework::standard::{Args, CommandError};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::prelude::Context;

use super::alias_from_arg_or_channel_name;
use crate::db::*;
use crate::model::enums::Era;
use crate::model::{GameServer, GameServerState, LobbyState};

#[cfg(test)]
mod tests;

fn lobby_helper(
    db_conn: &DbConnection,
    era: Era,
    player_count: i32,
    alias: &str,
    author_id: UserId,
) -> Result<(), CommandError> {
    db_conn.insert_game_server(&GameServer {
        alias: alias.to_owned(),
        state: GameServerState::Lobby(LobbyState {
            era,
            owner: author_id,
            player_count,
            description: None,
        }),
    })?;
    Ok(())
}

pub fn lobby(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    let era_str = args.single_quoted::<String>()?;
    let era = Era::from_string(&era_str).ok_or("unknown era")?;
    let player_count = args.single_quoted::<i32>()?;
    let alias = alias_from_arg_or_channel_name(&mut args, &message)?;
    let data = context.data.read();
    let db_connection = data
        .get::<DbConnectionKey>()
        .ok_or("No DbConnection was created on startup. This is a bug.")?;

    lobby_helper(db_connection, era, player_count, &alias, message.author.id)?;

    message.reply(&format!("Creating game lobby with name {}", alias))?;
    Ok(())
}
