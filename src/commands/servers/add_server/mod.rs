use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::Message;

use server::ServerConnection;
use model::{GameServer, GameServerState, StartedState};
use db::{DbConnection, DbConnectionKey};

#[cfg(test)]
mod tests;

fn add_server_helper<C: ServerConnection>(server_address: &str, game_alias: &str, db_connection: &DbConnection) -> Result<(), CommandError> {
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

    db_connection.insert_game_server(&server)?;
    Ok(())
}

pub fn add_server<C: ServerConnection>(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    let server_address = args.single_quoted::<String>()?;
    
    let alias = args.single_quoted::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?.to_lowercase();

    if !args.is_empty() {
        return Err(CommandError::from("Too many arguments. TIP: spaces in arguments need to be quoted \"like this\""));
    }

    let data = context.data.lock();
    let db_connection = data.get::<DbConnectionKey>().ok_or("No DbConnection was created on startup. This is a bug.")?;
    add_server_helper::<C>(&server_address, &alias, db_connection)?;
    let text = format!("Successfully inserted with alias {}", alias);
    let _ = message.reply(&text);
    info!("{}", text);
    Ok(())
}
