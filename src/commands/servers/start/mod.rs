use server::ServerConnection;

use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::channel::Message;

use model::*;
use model::enums::Nations;
use db::*;
use super::alias_from_arg_or_channel_name;

fn start_helper<C: ServerConnection>(
    db_conn: &DbConnection,
    address: &String,
    alias: &String,
) -> Result<(), CommandError> {
    let server = db_conn.game_for_alias(&alias)?;

    match server.state {
        GameServerState::StartedState(_, _) => {
            return Err(CommandError::from("game already started"))
        }
        GameServerState::Lobby(lobby_state) => {
            let game_data = C::get_game_data(&address)?;
            if game_data.nations.len() as i32 > lobby_state.player_count {
                return Err(CommandError::from("game has more players than the lobby"));
            }

            let started_state = StartedState {
                address: address.clone(),
                last_seen_turn: game_data.turn,
            };

            db_conn.insert_started_state(&alias, &started_state)?;
        }
    }
    Ok(())
}

pub fn start<C: ServerConnection>(
    context: &mut Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>()
        .ok_or("No DbConnection was created on startup. This is a bug.")?;
    let address = args.single_quoted::<String>()?;
    let alias = alias_from_arg_or_channel_name(&mut args, &message)?;
    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
        ));
    }
    start_helper::<C>(db_conn, &address, &alias)?;
    message.reply(&"started!")?;
    for (player, nation_id) in db_conn.players_with_nations_for_game_alias(&alias)? {
        let &(name, era) = Nations::get_nation_desc(nation_id);
        let text = format!(
            "Pretender upload has started in {}.\nServer address is {}.\nYou are registered as {} {}",
            alias,
            address,
            era,
            name,
        );
        let private_channel = player.discord_user_id.create_dm_channel()?;
        private_channel.say(&text)?;
    }
    Ok(())
}
