use crate::server::{ServerConnection, RealServerConnection};

use serenity::framework::standard::{Args, CommandError};
use serenity::model::channel::Message;
use serenity::prelude::Context;

use super::alias_from_arg_or_channel_name;
use crate::db::*;
use crate::model::*;
use crate::commands::servers::{get_details_for_alias, StartedStateDetails, PotentialPlayer};
use crate::commands::servers::NationDetails;
use crate::commands::servers::turn_check::{create_messages_for_new_turn, notify_player_for_new_turn, NewTurnNation};

fn start_helper<C: ServerConnection>(
    db_conn: &DbConnection,
    address: &str,
    alias: &str,
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
                address: address.to_string(),
                last_seen_turn: game_data.turn,
            };

            db_conn.insert_started_state(&alias, &started_state)?;

            // This is a bit of a hack, the turncheck should take care of it
            let started_details = get_details_for_alias::<RealServerConnection>(db_conn, alias)?;
            let mut new_turn_messages = vec![];
            if let NationDetails::Started(started_details) = started_details.nations {
                if let StartedStateDetails::Uploading(uploading_details) = started_details.state {
                    for player in uploading_details.uploading_players {
                        if let PotentialPlayer::RegisteredOnly(user_id, _, nation_name) = player.potential_player {
                            new_turn_messages.push(NewTurnNation {
                                user_id,
                                message: format!(
                                    "Uploading has started in {}! You registered as {}. Server address is '{}'.",
                                    alias, nation_name, started_details.address
                                ),
                            });
                        }
                    }
                }
            }
            for new_turn_message in &new_turn_messages {
                let _ = notify_player_for_new_turn(new_turn_message);
            }
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
    let db_conn = data
        .get::<DbConnectionKey>()
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
    Ok(())
}
