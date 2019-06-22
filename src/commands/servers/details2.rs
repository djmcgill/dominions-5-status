use serenity::builder::CreateEmbed;
use serenity::framework::standard::{Args, CommandError};
use serenity::model::channel::Message;
use serenity::prelude::Context;

use crate::commands::servers::*;
use crate::db::{DbConnection, DbConnectionKey};
use crate::server::ServerConnection;
use crate::model::enums::{NationStatus, SubmissionStatus};

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
    read_handle: &crate::ReadHandle,
) -> Result<CreateEmbed, CommandError> {
    let option_option_game_details = read_handle.handle().get_and(alias, |values| {
        if values.len() != 1 {
            panic!()
        } else {
            (*values[0]).1.clone()
        }
    });

    match option_option_game_details {
        Some(Some(details)) => {
            let embed = details_to_embed(details);
            println!("DETAILS2 EMBED: {:?}", embed);
            embed
        },
        Some(None) => Err(CommandError::from("Failed to connect to the server.")),
        None => {
            if db_conn
                .retrieve_all_servers()?
                .into_iter()
                .find(|server| &server.alias == alias)
                .is_some()
            {
                Err(CommandError::from(
                    "Server starting up, please try again in 1 min.",
                ))
            } else {
                Err(CommandError::from(format!(
                    "Game with alias '{}' not found.",
                    alias
                )))
            }
        }
    }
}

fn details_to_embed(details: GameDetails) -> Result<CreateEmbed, CommandError> {
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

                    let mut embed_text = String::new();
                    for potential_player in &playing_state.players {
                        let (option_user_id, player_details) = match potential_player {
                            // If the game has started and they're not in it, too bad
                            PotentialPlayer::RegisteredOnly(_, _, _) => continue,
                            PotentialPlayer::RegisteredAndGame(user_id, player_details) => {
                                (Some(user_id), player_details)
                            }
                            PotentialPlayer::GameOnly(player_details) => (None, player_details),
                        };

                        let player_name = if let NationStatus::Human = player_details.player_status {
                            match option_user_id {
                                Some(user_id) => {
                                    format!("**{}**", user_id.to_user()?)
                                }
                                None => {
                                    player_details.player_status.show().to_owned()
                                }
                            }
                        } else {
                            player_details.player_status.show().to_owned()
                        };

                        let submission_symbol = if let NationStatus::Human = player_details.player_status {
                            player_details.submitted.show().to_owned()
                        } else {
                            SubmissionStatus::Submitted.show().to_owned()
                        };

                        embed_text.push_str(&format!(
                            "`{}` {} ({}): {}\n",
                            submission_symbol,
                            player_details.nation_name,
                            player_details.nation_id,
                            player_name,
                        ));
                    }

                    CreateEmbed::default()
                        .title("Details")
                        .field(embed_title, embed_text, true)
                }
                StartedStateDetails::Uploading(uploading_state) => {
                    Err("Uploading details not implemented yet, just !details")?
                }
            }
        }
        NationDetails::Lobby(lobby_details) => {
            Err("Lobby details not implemented yet, just !details")?
        },
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
