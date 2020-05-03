use serenity::builder::CreateEmbed;
use serenity::framework::standard::{Args, CommandError};
use serenity::model::channel::Message;
use serenity::prelude::Context;

use crate::commands::servers::lobby_details;
use crate::commands::servers::*;
use crate::db::{DbConnection, DbConnectionKey};
use crate::model::enums::{NationStatus, SubmissionStatus};
use crate::model::GameServerState;
use crate::server::ServerConnection;

pub fn details2<C: ServerConnection>(
    context: &mut Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let data = context.data.lock();
    let db_conn = data
        .get::<DbConnectionKey>()
        .ok_or("No DbConnection was created on startup. This is a bug.")?;

    let read_handle = data
        .get::<crate::DetailsReadHandleKey>()
        .ok_or("No ReadHandle was created on startup. This is a bug.")?;

    let alias = alias_from_arg_or_channel_name(&mut args, &message)?;
    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
        ));
    }
    let embed_response = details_helper(&alias, db_conn, read_handle)?;

    message
        .channel_id
        .send_message(|m| m.embed(|_| embed_response))?;
    Ok(())
}

fn details_helper(
    alias: &str,
    db_conn: &DbConnection,
    read_handle: &crate::CacheReadHandle,
) -> Result<CreateEmbed, CommandError> {
    let server = db_conn.game_for_alias(&alias)?;
    match server.state {
        GameServerState::Lobby(ref lobby_state) => {
            let details: GameDetails = lobby_details(db_conn, lobby_state, alias)?;
            let embed: CreateEmbed = details_to_embed(details)?;
            Ok(embed)
        }
        GameServerState::StartedState(ref started_state, ref option_lobby_state) => {
            let option_option_game_cache = read_handle.get_clone(alias);

            match option_option_game_cache {
                Some(Some(cache)) => {
                    let CacheEntry {
                        game_data,
                        option_snek_state,
                    } = cache;

                    let details: GameDetails = started_details_from_server(
                        db_conn,
                        started_state,
                        option_lobby_state.as_ref(),
                        alias,
                        game_data,
                        option_snek_state,
                    )?;

                    let embed: CreateEmbed = details_to_embed(details)?;
                    Ok(embed)
                }
                Some(None) => Err("Got an error when trying to connect to the server".into()),
                None => Err("Not yet got a response from server, try again in 1 min".into()),
            }
        }
    }
}

fn details_to_embed(details: GameDetails) -> Result<CreateEmbed, CommandError> {
    let option_snek_state = details
        .cache_entry
        .and_then(|cache_entry| cache_entry.option_snek_state);

    let mut e = match details.nations {
        NationDetails::Started(started_details) => {
            match &started_details.state {
                StartedStateDetails::Playing(playing_state) => {
                    let embed_title = format!(
                        "{} ({}): turn {}, {}h {}m remaining",
                        started_details.game_name,
                        started_details.address,
                        playing_state.turn,
                        playing_state.hours_remaining,
                        playing_state.mins_remaining
                    );

                    // we can't have too many players per embed it's real annoying
                    let mut embed_texts = vec![];
                    for (ix, potential_player) in playing_state.players.iter().enumerate() {
                        let (option_user_id, player_details) = match potential_player {
                            // If the game has started and they're not in it, too bad
                            PotentialPlayer::RegisteredOnly(_, _) => continue,
                            PotentialPlayer::RegisteredAndGame(user_id, player_details) => {
                                (Some(user_id), player_details)
                            }
                            PotentialPlayer::GameOnly(player_details) => (None, player_details),
                        };

                        let player_name = if let NationStatus::Human = player_details.player_status
                        {
                            match option_user_id {
                                Some(user_id) => format!("**{}**", user_id.to_user()?),
                                None => player_details.player_status.show().to_owned(),
                            }
                        } else {
                            player_details.player_status.show().to_owned()
                        };

                        let submission_symbol = if player_details.player_status.is_human() {
                            player_details.submitted.show().to_owned()
                        } else {
                            SubmissionStatus::Submitted.show().to_owned()
                        };

                        if ix % 20 == 0 {
                            embed_texts.push(String::new());
                        }
                        let new_len = embed_texts.len();

                        embed_texts[new_len - 1].push_str(&format!(
                            "`{}` {}: {}\n",
                            submission_symbol,
                            player_details
                                .nation_identifier
                                .name(option_snek_state.as_ref()),
                            player_name,
                        ));
                    }

                    // This is pretty hacky
                    let mut e = CreateEmbed::default().title("Details").field(
                        embed_title,
                        embed_texts[0].clone(),
                        false,
                    );
                    for embed_text in &embed_texts[1..] {
                        e = e.field("-----", embed_text, false);
                    }
                    e
                }
                StartedStateDetails::Uploading(uploading_state) => {
                    let embed_title = format!(
                        "{} ({}): Pretender uploading",
                        started_details.game_name, started_details.address,
                    );

                    let mut embed_texts = vec![];
                    for (ix, uploading_player) in
                        uploading_state.uploading_players.iter().enumerate()
                    {
                        let player_name = match uploading_player.option_player_id() {
                            Some(user_id) => format!("**{}**", user_id.to_user()?),
                            None => NationStatus::Human.show().to_owned(),
                        };

                        let player_submitted_status = if uploading_player.uploaded {
                            SubmissionStatus::Submitted.show()
                        } else {
                            SubmissionStatus::NotSubmitted.show()
                        };

                        if ix % 20 == 0 {
                            embed_texts.push(String::new());
                        }
                        let new_len = embed_texts.len();
                        embed_texts[new_len - 1].push_str(&format!(
                            "`{}` {}: {}\n",
                            player_submitted_status,
                            uploading_player.nation_name(option_snek_state.as_ref()),
                            player_name,
                        ));
                    }
                    // This is pretty hacky
                    let mut e = CreateEmbed::default().title("Details").field(
                        embed_title,
                        embed_texts[0].clone(),
                        false,
                    );
                    for embed_text in &embed_texts[1..] {
                        e = e.field("-----", embed_text, false);
                    }
                    e
                }
            }
        }
        NationDetails::Lobby(lobby_details) => {
            let embed_title = match lobby_details.era {
                Some(era) => format!("{} ({} Lobby)", details.alias, era),
                None => format!("{} (Lobby)", details.alias),
            };
            let mut embed_texts = vec![];

            if lobby_details.players.len() != 0 {
                for (ix, lobby_player) in lobby_details.players.iter().enumerate() {
                    let discord_user = lobby_player.player_id.to_user()?;
                    if ix % 20 == 0 {
                        embed_texts.push(String::new());
                    }
                    let new_len = embed_texts.len();
                    embed_texts[new_len - 1]
                        .push_str(&format!("{}: {}\n", lobby_player.cached_name, discord_user,));
                }
            } else {
                embed_texts.push(String::new());
            }

            // We don't increase the number of fields any more
            let new_len = embed_texts.len();
            for _ in 0..lobby_details.remaining_slots {
                embed_texts[new_len - 1].push_str("OPEN\n");
            }
            // This is pretty hacky
            let mut e = CreateEmbed::default().title("Details").field(
                embed_title,
                embed_texts[0].clone(),
                false,
            );
            for embed_text in &embed_texts[1..] {
                e = e.field("-----", embed_text, false);
            }
            e
        }
    };
    for owner in details.owner {
        e = e.field("Owner", owner.to_user()?.to_string(), false);
    }

    for description in details.description {
        if !description.is_empty() {
            e = e.field("Description", description, false);
        }
    }
    Ok(e)
}
