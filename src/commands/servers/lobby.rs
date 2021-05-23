use serenity::framework::standard::{Args, CommandError};
use serenity::model::id::{ChannelId, UserId};
use serenity::prelude::Context;

use super::alias_from_arg_or_channel_name;
use crate::commands::servers::CommandResponse;
use crate::db::*;
use crate::model::enums::Era;
use crate::model::game_server::{GameServer, GameServerState, LobbyState};

fn lobby_helper(
    db_conn: DbConnection,
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

pub async fn lobby(
    context: &Context,
    channel_id: ChannelId,
    user_id: UserId,
    mut args: Args,
) -> Result<CommandResponse, CommandError> {
    let era_str = args.single_quoted::<String>()?;
    let era = Era::from_string(&era_str).ok_or("unknown era")?;
    let player_count = args.single_quoted::<i32>()?;
    let alias = alias_from_arg_or_channel_name(context, channel_id, &mut args).await?;
    let db_connection = {
        let data = context.data.read().await;
        data.get::<DbConnectionKey>()
            .ok_or("No DbConnection was created on startup. This is a bug.")?
            .clone()
    };

    lobby_helper(db_connection, era, player_count, &alias, user_id)?;
    Ok(CommandResponse::Reply(format!(
        "Creating game lobby with name {}",
        alias
    )))
}
