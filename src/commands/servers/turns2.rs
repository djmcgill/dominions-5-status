use log::*;
use serenity::framework::standard::CommandError;
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::prelude::Context;

use crate::commands::servers::*;
use crate::db::*;
use crate::model::enums::*;
use crate::server::ServerConnection;

fn turns_helper<C: ServerConnection>(
    user_id: UserId,
    db_conn: &DbConnection,
    read_handle: &crate::ReadHandle,
) -> Result<String, CommandError> {
    debug!("Starting !turns");
    let servers_and_nations_for_player = db_conn.servers_for_player(user_id)?;

    let mut text = "Your turns:\n".to_string();
    for (server, _) in servers_and_nations_for_player {
        let option_option_game_details = read_handle.handle().get_and(&server.alias, |values| {
            if values.len() != 1 {
                unreachable!() // it absolutely cannot happen that we have 2 games with the same name
            } else {
                (*values[0]).1.clone()
            }
        });

        match option_option_game_details {
            Some(Some(details)) => {
                match details.nations {
                    NationDetails::Started(started_state) => {
                        match started_state.state {
                            StartedStateDetails::Uploading(uploading_state) => {
                                let player_count = uploading_state.uploading_players.len();
                                let uploaded_player_count = uploading_state
                                    .uploading_players
                                    .iter()
                                    .filter(|uploading_player| {
                                        match uploading_player.potential_player {
                                            PotentialPlayer::GameOnly(_) => true,
                                            PotentialPlayer::RegisteredAndGame(_, _) => true,
                                            PotentialPlayer::RegisteredOnly(_, _, _) => false,
                                        }
                                    })
                                    .count();

                                for uploading_player in uploading_state.uploading_players {
                                    match uploading_player.potential_player {
                                        // This isn't them - it's somebody not registered
                                        PotentialPlayer::GameOnly(_) => (),
                                        PotentialPlayer::RegisteredAndGame(
                                            registered_user_id,
                                            player_details,
                                        ) =>
                                        // is this them?
                                        {
                                            if registered_user_id == user_id {
                                                let turn_str = format!(
                                                    "{} uploading: {} ({}) (uploaded: {}, {}/{})\n",
                                                    server.alias,
                                                    player_details.nation_name,
                                                    player_details.nation_id,
                                                    SubmissionStatus::Submitted.show(),
                                                    uploaded_player_count,
                                                    player_count,
                                                );
                                                text.push_str(&turn_str);
                                            }
                                        }
                                        // If this is them, they haven't uploaded
                                        PotentialPlayer::RegisteredOnly(
                                            registered_user_id,
                                            nation_id,
                                            nation_name,
                                        ) => {
                                            if registered_user_id == user_id {
                                                let turn_str = format!(
                                                    "{} uploading: {} ({}) (uploaded: {}, {}/{})\n",
                                                    server.alias,
                                                    nation_name,
                                                    nation_id,
                                                    SubmissionStatus::NotSubmitted.show(),
                                                    uploaded_player_count,
                                                    player_count,
                                                );
                                                text.push_str(&turn_str);
                                            }
                                        }
                                    }
                                }
                            }
                            StartedStateDetails::Playing(playing_state) => {
                                let mut playing_players = 0;
                                let mut submitted_players = 0;
                                for playing_player in &playing_state.players {
                                    match playing_player {
                                        PotentialPlayer::RegisteredOnly(_, _, _) => (),
                                        PotentialPlayer::RegisteredAndGame(_, player_details) => {
                                            if let NationStatus::Human =
                                                player_details.player_status
                                            {
                                                playing_players += 1;
                                                if let SubmissionStatus::Submitted =
                                                    player_details.submitted
                                                {
                                                    submitted_players += 1;
                                                }
                                            }
                                        }
                                        PotentialPlayer::GameOnly(player_details) => {
                                            if let NationStatus::Human =
                                                player_details.player_status
                                            {
                                                playing_players += 1;
                                                if let SubmissionStatus::Submitted =
                                                    player_details.submitted
                                                {
                                                    submitted_players += 1;
                                                }
                                            }
                                        }
                                    }
                                }

                                for playing_player in playing_state.players {
                                    match playing_player {
                                        PotentialPlayer::RegisteredOnly(_, _, _) => (),
                                        PotentialPlayer::GameOnly(_) => (),
                                        PotentialPlayer::RegisteredAndGame(
                                            potential_player_user_id,
                                            potential_player_details,
                                        ) => {
                                            if potential_player_user_id == user_id {
                                                if let NationStatus::Human =
                                                    potential_player_details.player_status
                                                {
                                                    let turn_str = format!(
                                                        "{} turn {} ({}h {}m): {} ({}) (submitted: {}, {}/{})\n",
                                                        server.alias,
                                                        playing_state.turn,
                                                        playing_state.hours_remaining,
                                                        playing_state.mins_remaining,
                                                        potential_player_details.nation_name,
                                                        potential_player_details.nation_id,
                                                        potential_player_details.submitted.show(),
                                                        submitted_players,
                                                        playing_players,
                                                    );
                                                    text.push_str(&turn_str);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    NationDetails::Lobby(_) => continue,
                }
            }
            Some(None) => {
                text.push_str(&format!("{}: Cannot connect to server!\n", server.alias));
            }
            None => {
                text.push_str(&format!(
                    "{}: Server starting up, please try again in 1 min.\n",
                    server.alias
                ));
            }
        }
    }
    Ok(text)
}

pub fn turns2<C: ServerConnection>(
    context: &mut Context,
    message: &Message,
) -> Result<(), CommandError> {
    let data = context.data.lock();
    let db_conn = data
        .get::<DbConnectionKey>()
        .ok_or_else(|| CommandError("No db connection".to_string()))?;
    let read_handle = data
        .get::<crate::DetailsReadHandleKey>()
        .ok_or("No ReadHandle was created on startup. This is a bug.")?;
    let text = turns_helper::<C>(message.author.id, db_conn, read_handle)?;
    info!("turns: replying with: {}", text);
    let private_channel = message.author.id.create_dm_channel()?;
    private_channel.say(&text)?;
    Ok(())
}
