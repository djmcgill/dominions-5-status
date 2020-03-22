use serenity::{
    framework::standard::{Args, CommandError},
    model::channel::Message,
    prelude::Context,
};

use crate::{
    commands::servers::alias_from_arg_or_channel_name, db::*, model::game_server::GameServerState,
};

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

pub async fn unstart(
    context: &Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let data = context.data.read().await;
    let db_conn = data
        .get::<DbConnectionKey>()
        .ok_or("No DbConnection was created on startup. This is a bug.")?;
    let alias = alias_from_arg_or_channel_name(&mut args, &message, context).await?;
    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
        ));
    }
    unstart_helper(db_conn, &alias)?;
    message
        .reply(
            (&context.cache, context.http.as_ref()),
            &format!("Successfully turned '{}' back into a lobby", alias),
        )
        .await?;
    Ok(())
}
