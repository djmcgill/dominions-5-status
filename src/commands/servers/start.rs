use crate::commands::servers::turn_check::{notify_player_for_new_turn, NewTurnNation};
use crate::commands::servers::CommandResponse;
use crate::{
    commands::servers::{
        alias_from_arg_or_channel_name,
        details::get_details_for_alias,
        // turn_check::{notify_player_for_new_turn, NewTurnNation},
    },
    db::*,
    model::{
        game_server::{GameServerState, StartedState},
        game_state::{NationDetails, PotentialPlayer, StartedStateDetails},
    },
    server::get_game_data_async,
    snek::snek_details_async,
};
use serenity::model::id::{ChannelId, UserId};
use serenity::{
    framework::standard::{Args, CommandError},
    prelude::Context,
};

async fn start_helper(
    db_conn: DbConnection,
    address: &str,
    alias: &str,
    context: &Context,
) -> Result<(), CommandError> {
    let server = db_conn.game_for_alias(alias)?;

    match server.state {
        GameServerState::StartedState(_, _) => {
            return Err(CommandError::from("game already started"))
        }
        GameServerState::Lobby(lobby_state) => {
            let game_data = get_game_data_async(address).await?;
            if game_data.nations.len() as i32 > lobby_state.player_count {
                return Err(CommandError::from("game has more players than the lobby"));
            }

            let started_state = StartedState {
                address: address.to_string(),
                last_seen_turn: game_data.turn,
            };

            db_conn.insert_started_state(alias, &started_state)?;

            let started_details = get_details_for_alias(db_conn, alias).await?;
            let option_snek_state = snek_details_async(address).await.ok().and_then(|i| i);

            if let NationDetails::Started(started_details) = started_details.nations {
                if let StartedStateDetails::Uploading(uploading_details) = started_details.state {
                    for player in uploading_details.uploading_players {
                        if let PotentialPlayer::RegisteredOnly(user_id, nation_id) =
                            player.potential_player
                        {
                            let message = NewTurnNation {
                                    user_id,
                                    message: format!(
                                        "Uploading has started in {}! You registered as {}. Server address is '{}'.",
                                        alias, nation_id.name(option_snek_state.as_ref()), started_details.address
                                    ),
                                };

                            let cache = context.cache.clone();
                            let http = context.http.clone();
                            let _ = tokio::spawn(async move {
                                // fixme: error handling
                                let _ =
                                    notify_player_for_new_turn(message, (&cache, http.as_ref()))
                                        .await;
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub async fn start(
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
    let address = args.single_quoted::<String>()?;
    let alias = alias_from_arg_or_channel_name(context, channel_id, &mut args).await?;
    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
        ));
    }
    start_helper(db_conn, &address, &alias, context).await?;
    Ok(CommandResponse::Reply("started!".to_owned()))
}
