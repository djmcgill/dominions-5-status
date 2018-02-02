use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::{Message, UserId};

use model::{GameServer, GameServerState, LobbyState};
use model::enums::Era;
use db::*;

fn lobby_helper(db_conn: &DbConnection, era: Era, player_count: i32, alias: &String, author_id: UserId) -> Result<(), CommandError> {
    db_conn.insert_game_server(&GameServer {
        alias: alias.clone(),
        state: GameServerState::Lobby(LobbyState {
            era,
            owner: author_id,
            player_count,
        }),
    })?;
    Ok(())
}

pub fn lobby(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    let era_str = args.single_quoted::<String>()?;
    let era = Era::from_string(&era_str).ok_or("unknown era")?;
    let player_count = args.single_quoted::<i32>()?;
    let alias = args.single_quoted::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?.to_lowercase();
    
    let data = context.data.lock();
    let db_connection = data.get::<DbConnectionKey>().ok_or("No DbConnection was created on startup. This is a bug.")?;

    lobby_helper(db_connection, era, player_count, &alias, message.author.id)?;

    message.reply(&format!("Creating game lobby with name {}", alias))?;
    Ok(())
}
