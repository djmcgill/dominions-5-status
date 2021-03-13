use crate::commands::servers::details::{
    get_details_for_alias, lobby_details, started_details_from_server,
};
use crate::{
    db::*,
    model::{
        enums::*,
        game_server::GameServerState,
        game_state::{
            CacheEntry, GameDetails, NationDetails, PotentialPlayer, StartedDetails,
            StartedStateDetails,
        },
    },
    server::get_game_data_async,
    snek::{snek_details_async, SnekGameStatus},
    DetailsCache, DetailsCacheHandle, DetailsCacheKey,
};

use anyhow::anyhow;
use chrono::Utc;
use futures::{
    future::{self, FutureExt},
    stream::{self, StreamExt},
};
use im::hashmap::Entry;
use log::*;
use serenity::{
    framework::standard::CommandError, http::Http, model::id::UserId, prelude::*, CacheAndHttp,
};
use std::time::Duration;
use std::{sync::Arc, thread, time};

pub async fn update_details_cache_loop(
    db_conn: DbConnection,
    write_handle_mutex: DetailsCacheHandle,
    cache_and_http: Arc<CacheAndHttp>,
) {
    stream::repeat(())
        .for_each(|()| async {
            info!("Checking for new turns!");

            let new_turn_nations =
                update_details_cache_for_all_games(&db_conn, write_handle_mutex.clone()).await;
            let mut futures_vec = vec![];
            for new_turn_nation in new_turn_nations {
                let user_id = new_turn_nation.user_id;
                // Intentionally no `await` or `?`
                let future = notify_player_for_new_turn(new_turn_nation, cache_and_http.clone())
                    .map(move |r| {
                        r.unwrap_or_else(|e| {
                            error!(
                                "Failed to notify new turn for user {:?} with error: {:#?}",
                                user_id, e
                            );
                        })
                    });

                futures_vec.push(future);
            }
            future::join_all(futures_vec).await;
            tokio::time::sleep(Duration::from_secs(60)).await;
        })
        .await;
}

pub async fn notify_player_for_new_turn(
    new_turn: NewTurnNation,
    cache_and_http: Arc<CacheAndHttp>,
) -> Result<(), CommandError> {
    let private_channel = new_turn
        .user_id
        .create_dm_channel(cache_and_http.as_ref())
        .await?;
    private_channel
        .say(cache_and_http.http.as_ref(), &new_turn.message)
        .await?;
    Ok(())
}

async fn update_details_cache_for_game(
    alias: &str,
    db_conn: &DbConnection,
    write_handle: &mut DetailsCache,
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
            .map_err(|e| anyhow!("Error when checking turn for {}: {:#?}", alias, e))?;
            if let NationDetails::Started(new_started_details) = &new_game_details.nations {
                create_messages_for_new_turn(
                    alias,
                    new_started_details,
                    option_new_snek_data.as_ref(),
                )
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        let cache_entry = CacheEntry {
            game_data: new_game_data,
            option_snek_state: option_new_snek_data,
        };
        write_handle.insert(alias.to_owned(), Box::new((Utc::now(), Some(cache_entry))));
        messages
    } else {
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
    db_conn: &DbConnection,
    write_handle_mutex: DetailsCacheHandle,
) -> Vec<NewTurnNation> {
    let mut guard = write_handle_mutex.0.write().await;
    match guard.get_mut::<DetailsCacheKey>() {
        None => {
            todo!()
        }
        Some(details_cache) => {
            let mut ret = vec![];
            match db_conn.retrieve_all_servers() {
                Err(e) => {
                    error!("Could not query the db for all servers with error: {:?}", e);
                }
                Ok(servers) => {
                    // TODO: might want to parallelise if I could figure out the parallel cache updates
                    //        (partitioned by key) - or just await on the write handle when it comes in?
                    for server in servers {
                        match update_details_cache_for_game(&server.alias, db_conn, details_cache)
                            .await
                        {
                            Ok(new_turns) => {
                                ret.extend(new_turns.into_iter());
                            }
                            Err(e) => {
                                error!("Could not update game {} with error {:?}", server.alias, e);
                            }
                        }
                    }
                }
            }
            ret
        }
    }
}

pub fn create_messages_for_new_turn(
    alias: &str,
    new_started_details: &StartedDetails,
    option_snek_state: Option<&SnekGameStatus>,
) -> Vec<NewTurnNation> {
    let mut ret = vec![];
    match &new_started_details.state {
        StartedStateDetails::Playing(new_playing_details) => {
            for potential_player in &new_playing_details.players {
                match potential_player {
                    PotentialPlayer::GameOnly(_) => {} // Don't know who they are, can't message them
                    PotentialPlayer::RegisteredOnly(_, _) => {} // Looks like they got left out, too bad
                    PotentialPlayer::RegisteredAndGame(user_id, details) => {
                        // Only message them if they haven't submitted yet
                        if let SubmissionStatus::NotSubmitted = details.submitted {
                            // and if they're actually playing
                            if details.player_status.is_human() {
                                ret.push(
                                        NewTurnNation {
                                            user_id: *user_id,
                                            message: format!("New turn in {}! You are {} and you have {}h {}m remaining for turn {}.",
                                                             alias,
                                                             details.nation_identifier.name(option_snek_state),
                                                             new_playing_details.hours_remaining,
                                                             new_playing_details.mins_remaining,
                                                             new_playing_details.turn,
                                            )
                                        }
                                    );
                            }
                        }
                    }
                }
            }
        }
        StartedStateDetails::Uploading(ref new_uploading_details) => {
            for player in &new_uploading_details.uploading_players {
                if let Some(user_id) = player.option_player_id() {
                    if !player.uploaded {
                        ret.push(NewTurnNation {
                                user_id: *user_id,
                                message: format!(
                                    "Uploading has started in {}! You registered as {}. Server address is '{}'.",
                                    alias, player.nation_name(option_snek_state), new_started_details.address
                                ),
                            });
                    }
                }
            }
        }
    }
    ret
}
