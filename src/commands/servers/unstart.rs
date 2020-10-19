use serenity::{CacheAndHttp, framework::standard::{Args, CommandError}};
use serenity::model::channel::Message;
use serenity::prelude::Context;

use super::alias_from_arg_or_channel_name;
use crate::db::*;
use crate::model::*;

#[cfg(test)]
mod unstart_tests;

fn unstart_helper(db_conn: &DbConnection, alias: &str) -> Result<(), CommandError> {
    let server = db_conn.game_for_alias(&alias)?;

    match server.state {
        GameServerState::StartedState(_, _) => {
            db_conn.remove_started_state(&alias)?;
        }
        GameServerState::Lobby(_) => {
            return Err(CommandError::from("cannot use this command on a lobby"))
        }
    }
    Ok(())
}

pub fn unstart(
    context: &mut Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    // TODO: make sure this is a suitable replacement for data.lock()
    // let data = context.data.read();
    let data = context.data.read();
    let db_conn = data
        .get::<DbConnectionKey>()
        .ok_or("No DbConnection was created on startup. This is a bug.")?;
    let alias = alias_from_arg_or_channel_name(&mut args, &message)?;
    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
        ));
    }
    unstart_helper(db_conn, &alias)?;
    message.reply(CacheAndHttp::default(), &format!(
        "Successfully turned '{}' back into a lobby",
        alias
    ))?;
    Ok(())
}
