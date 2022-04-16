use crate::{
    commands::servers::{details::started_details_from_server, discord_date_format},
    db::*,
    model::{
        enums::*,
        game_server::GameServerState,
        game_state::{
            CacheEntry, GameDetails, NationDetails, PlayerDetails, PlayingState, PotentialPlayer,
            StartedDetails, StartedStateDetails,
        },
        nation::Nation,
    },
    server::get_game_data_async,
    snek::{snek_details_async, SnekGameStatus},
    DetailsCacheHandle, DetailsCacheKey,
};
use anyhow::{anyhow, Context};
use chrono::Utc;
use futures::future::{self, FutureExt};
use log::*;
use serenity::{http::CacheHttp, model::id::UserId, CacheAndHttp};
use std::{sync::Arc, time::Duration};

pub async fn update_details_cache_loop(
    db_conn: DbConnection,
    write_handle_mutex: DetailsCacheHandle,
    cache_and_http: Arc<CacheAndHttp>,
) {
    loop {
        info!("Checking for new turns!");

        match update_details_cache_for_all_games(db_conn.clone(), write_handle_mutex.clone()).await
        {
            Err(e) => error!("Error updating all games: {:#?}", e),
            Ok(new_turn_nations) => {
                future::join_all(new_turn_nations.into_iter().map(|new_turn_nation| {
                    let user_id = new_turn_nation.user_id;
                    // Intentionally no `await` or `?`
                    notify_player_for_new_turn(new_turn_nation, cache_and_http.clone()).map(
                        move |r| {
                            r.unwrap_or_else(|e| {
                                error!(
                                    "Failed to notify new turn for user {:?} with error: {:#?}",
                                    user_id, e
                                );
                            })
                        },
                    )
                }))
                .await;
            }
        }

        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

pub async fn notify_player_for_new_turn(
    new_turn: NewTurnNation,
    cache_and_http: impl CacheHttp + Clone,
) -> anyhow::Result<()> {
    let private_channel = new_turn
        .user_id
        .create_dm_channel(cache_and_http.clone())
        .await?;
    private_channel
        .say(cache_and_http.http(), &new_turn.message)
        .await?;
    Ok(())
}

/// If the game is not still a lobby, connect to the server and get the new state. Then,
///   1) update the db
///   2) work out which players need to be notified (but don't actually send any messages yet)
///   3) update the in-mem cache with the new details
async fn update_details_cache_for_game(
    alias: &str,
    db_conn: DbConnection,
    write_handle_mutex: DetailsCacheHandle,
) -> anyhow::Result<Vec<NewTurnNation>> {
    info!("Checking turn for {}", alias);

    let details = db_conn.game_for_alias(alias)?;

    let messages = if let GameServerState::StartedState(started_state, option_lobby_state) =
        &details.state
    {
        let new_game_data = get_game_data_async(&started_state.address).await?;
        let option_new_snek_data = snek_details_async(&started_state.address).await?;

        let is_new_turn = db_conn.update_game_with_possibly_new_turn(alias, new_game_data.turn)?;
        let messages = if is_new_turn {
            let new_game_details: GameDetails = started_details_from_server(
                db_conn,
                started_state,
                option_lobby_state.as_ref(),
                alias,
                &new_game_data,
                option_new_snek_data.as_ref(),
            )
            .map_err(|e| anyhow!(e))
            .with_context(|| format!("Error when checking turn for {}", alias))?;
            if let NationDetails::Started(new_started_details) = &new_game_details.nations {
                let option_old_nations = (|| async {
                    let guard = write_handle_mutex.0.read().await;
                    let old_state = guard.get::<DetailsCacheKey>()?;
                    let (_, old_cache) = &**(old_state.get(&alias.to_owned())?);
                    Some(old_cache.as_ref()?.game_data.nations.clone())
                })()
                .await;

                create_messages_for_new_turn(
                    alias,
                    new_started_details,
                    option_new_snek_data.as_ref(),
                    option_old_nations.as_ref(),
                )
            } else {
                // this should never happen, `started_details_from_server` cannot return a lobby
                vec![]
            }
        } else {
            // not a new turn
            vec![]
        };

        let cache_entry = CacheEntry {
            game_data: new_game_data,
            option_snek_state: option_new_snek_data,
        };
        let mut guard = write_handle_mutex.0.write().await;
        let write_handle = guard
            .get_mut::<DetailsCacheKey>()
            .ok_or_else(|| anyhow!("Cache somehow not initialised, this should never happen!!"))?;
        write_handle.insert(alias.to_owned(), Box::new((Utc::now(), Some(cache_entry))));

        messages
    } else {
        // game is still a lobby
        vec![]
    };
    info!("Checking turn for {}: SUCCESS", alias);
    Ok(messages)
}

#[derive(Debug)]
pub struct NewTurnNation {
    pub user_id: UserId,
    pub message: String,
}

async fn update_details_cache_for_all_games(
    db_conn: DbConnection,
    write_handle_mutex: DetailsCacheHandle,
) -> Result<Vec<NewTurnNation>, anyhow::Error> {
    let servers = db_conn
        .retrieve_all_servers()
        .context("Could not query the db for all servers")?;
    info!("retrieve_all_servers done, found {} entries", servers.len());
    let write_handle_mutex = &write_handle_mutex;
    let db_conn = &db_conn;
    let futs = servers.into_iter().map(|server| async move {
        debug!("starting to update: '{}'", server.alias);
        match update_details_cache_for_game(
            &server.alias,
            db_conn.clone(),
            write_handle_mutex.clone(),
        )
        .await
        {
            Ok(new_turns) => new_turns,
            Err(e) => {
                error!("Could not update game {} with error {:?}", server.alias, e);
                vec![]
            }
        }
    });
    Ok(future::join_all(futs).await.into_iter().flatten().collect())
}

pub fn create_messages_for_new_turn(
    alias: &str,
    new_started_details: &StartedDetails,
    option_snek_state: Option<&SnekGameStatus>,
    option_old_nations: Option<&Vec<Nation>>,
) -> Vec<NewTurnNation> {
    let possible_stales = if let StartedStateDetails::Playing(_) = &new_started_details.state {
        if let Some(old_nations) = option_old_nations {
            old_nations
                .iter()
                .filter(|old_nation| old_nation.submitted == SubmissionStatus::NotSubmitted)
                .collect::<Vec<_>>()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    match &new_started_details.state {
        StartedStateDetails::Playing(new_playing_details) => {
            new_playing_details.players.iter().flat_map(|potential_player| {
                match potential_player {
                    PotentialPlayer::GameOnly(_) => None, // Don't know who they are, can't message them
                    PotentialPlayer::RegisteredOnly(_, _) => None, // Looks like they got left out, too bad
                    PotentialPlayer::RegisteredAndGame(user_id, details) => create_playing_message(
                        alias,
                        new_playing_details,
                        option_snek_state,
                        user_id,
                        details,
                        &possible_stales,
                    ),
                }
            }).collect()
        }
        StartedStateDetails::Uploading(ref new_uploading_details) => {
            new_uploading_details.uploading_players.iter().flat_map(|player| {
                player.option_player_id().filter(|_| !player.uploaded).map(|user_id|  {
                        NewTurnNation {
                                user_id: *user_id,
                                message: format!(
                                    "Uploading has started in {}! You registered as {}. Server address is '{}'.",
                                    alias, player.nation_name(option_snek_state), new_started_details.address
                                ),
                            }
                    })
            }).collect()
        }
    }
}

fn create_playing_message(
    alias: &str,
    new_playing_details: &PlayingState,
    option_snek_state: Option<&SnekGameStatus>,
    user_id: &UserId,
    details: &PlayerDetails,
    possible_stales: &[&Nation],
) -> Option<NewTurnNation> {
    // Only message them if they haven't submitted yet
    if let SubmissionStatus::NotSubmitted = details.submitted {
        // and if they're actually playing
        if details.player_status.is_human() {
            let deadline = discord_date_format(new_playing_details.turn_deadline);
            let possible_stale_message = if possible_stales.is_empty() {
                "".to_owned()
            } else {
                let mut msg = ". Possible stales: ".to_owned();
                for player in possible_stales {
                    msg.push_str(player.identifier.name(option_snek_state).as_ref());
                    msg.push_str(", ");
                }
                msg
            };

            return Some(NewTurnNation {
                user_id: *user_id,
                message: format!(
                    "Turn {} in {}! You are {} and timer is in {}{}",
                    new_playing_details.turn,
                    alias,
                    details.nation_identifier.name(option_snek_state),
                    deadline,
                    possible_stale_message,
                ),
            });
        }
    }
    None
}
