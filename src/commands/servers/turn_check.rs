use crate::commands::servers::*;
use crate::db::*;
use crate::model::enums::*;
use crate::server::RealServerConnection;
use crate::WriteHandle;
use chrono::{Duration, Utc};
use log::*;
use serenity::framework::standard::CommandError;
use serenity::model::id::UserId;
use serenity::prelude::*;
use std::sync::Arc;
use std::thread;
use std::time;

pub fn update_details_cache_loop(
    db_conn: DbConnection,
    write_handle_mutex: Arc<Mutex<WriteHandle>>,
) {
    loop {
        info!("Checking for new turns!");
        let mut option_new_turn_nations = None;
        for mut write_handle in write_handle_mutex.try_lock() {
            let new_turn_nations = update_details_cache_for_all_games(&db_conn, &mut write_handle);
            option_new_turn_nations = Some(new_turn_nations);
        }
        for new_turn_nation in option_new_turn_nations.unwrap_or(Vec::new()) {
            match notify_player_for_new_turn(&new_turn_nation) {
                Ok(()) => {}
                Err(e) => {
                    error!(
                        "Failed to notify new turn {:?} with error: {:?}",
                        new_turn_nation, e
                    );
                }
            }
        }
        thread::sleep(time::Duration::from_secs(60));
    }
}

pub fn notify_player_for_new_turn(new_turn: &NewTurnNation) -> Result<(), CommandError> {
    let private_channel = new_turn.user_id.create_dm_channel()?;
    private_channel.say(&new_turn.message)?;
    Ok(())
}

// FIXME: should just be regular error
fn update_details_cache_for_game(
    alias: &str,
    db_conn: &DbConnection,
    write_handle: &mut WriteHandle,
) -> Result<Vec<NewTurnNation>, CommandError> {
    info!("Checking turn for {}", alias);
    let mut ret = vec![];

    let option_old_details: Option<Option<GameDetails>> =
        write_handle.get_and(&alias.to_owned(), |results| (*results[0]).1.clone());

    let result_details = get_details_for_alias::<RealServerConnection>(db_conn, alias);
    let now = Utc::now();
    match result_details {
        Err(e) => {
            error!(
                "Got an error when checking for details for alias {}: {:?}",
                alias, e
            );
            write_handle.update(alias.to_owned(), Box::new((now, None)));
        }
        Ok(details) => {
            // It's a bit of a hack to have 2 ways to check for turns
            let updated = if let NationDetails::Started(started) = &details.nations {
                let turn = if let StartedStateDetails::Playing(playing) = &started.state {
                    playing.turn as i32
                } else {
                    -1
                };
                db_conn.update_game_with_possibly_new_turn(alias, turn)?
            } else {
                false
            };

            if updated {
                if let NationDetails::Started(started_details) = &details.nations {
                    ret.extend(create_messages_for_new_turn(alias, started_details));
                }
            } else {
                for old_details in option_old_details.and_then(|x| x) {
                    if was_updated(&old_details, &details) {
                        if let NationDetails::Started(started_details) = &details.nations {
                            ret.extend(create_messages_for_new_turn(alias, started_details));
                        }
                    }
                }
            }

            write_handle.update(alias.to_owned(), Box::new((now, Some(details))));
        }
    }

    // FIXME: might just want to store the hash instead of cloning the string a bunch
    info!("Checking turn for {}: SUCCESS", alias);

    Ok(ret)
}

#[derive(Debug)]
pub struct NewTurnNation {
    pub user_id: UserId,
    pub message: String,
}

fn update_details_cache_for_all_games(
    db_conn: &DbConnection,
    write_handle: &mut WriteHandle,
) -> Vec<NewTurnNation> {
    let mut ret = vec![];
    match db_conn.retrieve_all_servers() {
        Err(e) => {
            error!("Could not query the db for all servers with error: {:?}", e);
        }
        Ok(servers) => {
            // FIXME: might want to parallelise
            for server in servers {
                match update_details_cache_for_game(&server.alias, db_conn, write_handle) {
                    Ok(updates) => {
                        ret.extend(updates.into_iter());
                    }
                    Err(e) => {
                        error!("Could not update game {} with error {:?}", server.alias, e);
                    }
                }
            }
            write_handle.refresh();
        }
    }
    ret
}

pub fn remove_old_entries_from_cache_loop(write_handle_mutex: Arc<Mutex<WriteHandle>>) {
    loop {
        info!("Removing old entries from cache");
        for mut write_handle in write_handle_mutex.try_lock() {
            remove_old_entries_from_cache(&mut write_handle);
        }
        thread::sleep(time::Duration::from_secs(60 * 60 * 24)); // whatever doesn't need to be exactly 24 hours
    }
}

fn remove_old_entries_from_cache(write_handle: &mut WriteHandle) {
    let values: Vec<String> = write_handle.map_into(|key, _| key.clone());

    for value in values {
        write_handle.retain(value.clone(), move |box_value| {
            let (ref last_updated, _) = **box_value;
            let two_hours_ago = Utc::now().checked_sub_signed(Duration::hours(2)).unwrap();

            // If we've updated successfully more recently than 2 hours ago, keep it.
            let out_of_date = last_updated > &two_hours_ago;
            if out_of_date {
                info!("Removing game {} from cache", value);
            };
            !out_of_date
        });
    }
}

pub fn was_updated(old_details: &GameDetails, new_details: &GameDetails) -> bool {
    match (&old_details.nations, &new_details.nations) {
        (NationDetails::Lobby(_), NationDetails::Started(_)) => {
            true // Lobby has started
        }
        (
            NationDetails::Started(ref old_started_details),
            NationDetails::Started(ref new_started_details),
        ) => match (&old_started_details.state, &new_started_details.state) {
            (StartedStateDetails::Uploading(_), StartedStateDetails::Playing(_)) => {
                true // Pretender upload has finished
            }
            (
                StartedStateDetails::Playing(ref old_playing_details),
                StartedStateDetails::Playing(ref new_playing_details),
            ) => {
                old_playing_details.turn < new_playing_details.turn // New turn?
            }
            _ => false,
        },
        _ => false,
    }
}

fn create_messages_for_new_turn(
    alias: &str,
    new_started_details: &StartedDetails,
) -> Vec<NewTurnNation> {
    let mut ret = vec![];
    match new_started_details.state {
        StartedStateDetails::Playing(ref new_playing_details) => {
            for potential_player in &new_playing_details.players {
                match potential_player {
                    PotentialPlayer::GameOnly(_) => {} // Don't know who they are, can't message them
                    PotentialPlayer::RegisteredOnly(_, _, _) => {} // Looks like they got left out, too bad
                    PotentialPlayer::RegisteredAndGame(user_id, details) => {
                        // Only message them if they haven't submitted yet
                        if let SubmissionStatus::NotSubmitted = details.submitted {
                            ret.push(
                                NewTurnNation {
                                    user_id: *user_id,
                                    message: format!("New turn in {}! You are {} and you have {}h {}m remaining for turn {}.",
                                                     alias,
                                                     details.nation_name,
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
        StartedStateDetails::Uploading(ref new_uploading_details) => {
            for ref player in &new_uploading_details.uploading_players {
                if let Some(user_id) = player.option_player_id() {
                    if !player.uploaded {
                        ret.push(NewTurnNation {
                            user_id: *user_id,
                            message: format!(
                                "Uploading has started in {}! You registered as {}. Server address is '{}'.",
                                alias, player.nation_name(), new_started_details.address
                            ),
                        });
                    }
                }
            }
        }
    }
    ret
}
