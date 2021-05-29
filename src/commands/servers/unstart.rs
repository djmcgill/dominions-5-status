use serenity::{
    framework::standard::{Args, CommandError},
    prelude::Context,
};

use crate::commands::servers::CommandResponse;
use crate::{
    commands::servers::alias_from_arg_or_channel_name, db::*, model::game_server::GameServerState,
};
use serenity::model::id::{ChannelId, UserId};

fn unstart_helper(db_conn: DbConnection, alias: &str) -> Result<(), CommandError> {
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

pub async fn unstart(
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
    let alias = alias_from_arg_or_channel_name(context, channel_id, &mut args).await?;
    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
        ));
    }
    unstart_helper(db_conn, &alias)?;
    Ok(CommandResponse::Reply(format!(
        "Successfully turned '{}' back into a lobby",
        alias
    )))
}
