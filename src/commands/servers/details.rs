use serenity::builder::CreateEmbed;
use serenity::framework::standard::{Args, CommandError};
use serenity::model::channel::Message;
use serenity::prelude::Context;

use crate::db::{DbConnection, DbConnectionKey};
use crate::model::enums::{NationStatus, SubmissionStatus, Nations};
use crate::server::ServerConnection;
use crate::snek::SnekGameStatus;
use crate::commands::servers::alias_from_arg_or_channel_name;
use crate::model::game_server::*;
use crate::model::game_state::*;
use crate::model::game_data::GameData;
use crate::model::nation::Nation;
use crate::model::player::Player;
use std::collections::HashMap;
use log::*;

pub fn details2<C: ServerConnection>(
    context: &mut Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let data = context.data.read();
    let db_conn = data
        .get::<DbConnectionKey>()
        .ok_or("No DbConnection was created on startup. This is a bug.")?;

    let read_handle = data
        .get::<crate::DetailsReadHandleKey>()
        .ok_or("No ReadHandle was created on startup. This is a bug.")?;

    let alias = alias_from_arg_or_channel_name(&mut args, &message, context)?;
    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
        ));
    }
    let embed_response = details_helper(&alias, db_conn, read_handle, context)?;

    message
        .channel_id
        .send_message(&context.http, |m| m.embed(|e| {*e = embed_response; e}))?;
    Ok(())
}

pub fn get_details_for_alias<C: ServerConnection>(
    db_conn: &DbConnection,
    alias: &str,
) -> Result<GameDetails, CommandError> {
    let server = db_conn.game_for_alias(&alias)?;
    info!("got server details");

    let details = match server.state {
        GameServerState::Lobby(ref lobby_state) => lobby_details(db_conn, lobby_state, &alias)?,
        GameServerState::StartedState(ref started_state, ref option_lobby_state) => {
            started_details::<C>(db_conn, started_state, option_lobby_state.as_ref(), &alias)?
        }
    };

    Ok(details)
}

fn started_details<C: ServerConnection>(
    db_conn: &DbConnection,
    started_state: &StartedState,
    option_lobby_state: Option<&LobbyState>,
    alias: &str,
) -> Result<GameDetails, CommandError> {
    let server_address = &started_state.address;
    let game_data = C::get_game_data(&server_address)?;
    let option_snek_details = C::get_snek_data(server_address)?;

    started_details_from_server(
        db_conn,
        started_state,
        option_lobby_state,
        alias,
        game_data,
        option_snek_details,
    )
}


fn details_helper(
    alias: &str,
    db_conn: &DbConnection,
    read_handle: &crate::CacheReadHandle,
    context: &Context,
) -> Result<CreateEmbed, CommandError> {
    let server = db_conn.game_for_alias(&alias)?;
    match server.state {
        GameServerState::Lobby(ref lobby_state) => {
            let details: GameDetails = lobby_details(db_conn, lobby_state, alias)?;
            let embed: CreateEmbed = details_to_embed(details, context)?;
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

                    let embed: CreateEmbed = details_to_embed(details, context)?;
                    Ok(embed)
                }
                Some(None) => Err("Got an error when trying to connect to the server".into()),
                None => Err("Not yet got a response from server, try again in 1 min".into()),
            }
        }
    }
}

pub fn lobby_details(
    db_conn: &DbConnection,
    lobby_state: &LobbyState,
    alias: &str,
) -> Result<GameDetails, CommandError> {
    let players_nations = db_conn.players_with_nations_for_game_alias(&alias)?;

    let mut player_nation_details: Vec<LobbyPlayer> = players_nations
        .into_iter()
        .map(|(player, nation_id)| -> LobbyPlayer {
            let &(nation_name, _) = Nations::get_nation_desc(nation_id);
            LobbyPlayer {
                player_id: player.discord_user_id,
                nation_id,
                nation_name: nation_name.to_owned(),
            }
        })
        .collect();
    player_nation_details.sort_by(|n1, n2| n1.nation_name.cmp(&n2.nation_name));

    let remaining_slots = std::cmp::max(
        0,
        (lobby_state.player_count - player_nation_details.len() as i32) as u32,
    );

    let lobby_details = LobbyDetails {
        players: player_nation_details,
        era: Some(lobby_state.era),
        remaining_slots,
    };

    Ok(GameDetails {
        alias: alias.to_owned(),
        owner: Some(lobby_state.owner),
        description: lobby_state.description.clone(),
        nations: NationDetails::Lobby(lobby_details),
        cache_entry: None, // lobbies have no cache entry
    })
}


fn join_players_with_nations(
    nations: &Vec<Nation>,
    players_nations: &Vec<(Player, u32)>,
    option_snek_details: &Option<SnekGameStatus>,
) -> Result<Vec<PotentialPlayer>, CommandError> {
    let mut potential_players = vec![];

    let mut players_by_nation_id = HashMap::new();
    for (player, nation_id) in players_nations {
        players_by_nation_id.insert(*nation_id, player);
    }
    for nation in nations {
        match players_by_nation_id.remove(&nation.id) {
            // Lobby and game
            Some(player) => {
                let player_details = PlayerDetails {
                    nation_id: nation.id,
                    nation_name: get_nation_string(option_snek_details, nation.id),
                    submitted: nation.submitted,
                    player_status: nation.status,
                };
                potential_players.push(PotentialPlayer::RegisteredAndGame(
                    player.discord_user_id,
                    player_details,
                ))
            }
            // Game only
            None => potential_players.push(PotentialPlayer::GameOnly(PlayerDetails {
                nation_id: nation.id,
                nation_name: get_nation_string(option_snek_details, nation.id),
                submitted: nation.submitted,
                player_status: nation.status,
            })),
        }
    }
    // Lobby only
    for (nation_id, player) in players_by_nation_id {
        let &(nation_name, _) = Nations::get_nation_desc(nation_id);
        potential_players.push(PotentialPlayer::RegisteredOnly(
            player.discord_user_id,
            nation_id,
            nation_name.to_owned(),
        ));
    }
    potential_players.sort_unstable();
    Ok(potential_players)
}


pub fn started_details_from_server(
    db_conn: &DbConnection,
    started_state: &StartedState,
    option_lobby_state: Option<&LobbyState>,
    alias: &str,
    game_data: GameData,
    option_snek_details: Option<SnekGameStatus>,
) -> Result<GameDetails, CommandError> {
    let id_player_nations = db_conn.players_with_nations_for_game_alias(&alias)?;
    let player_details =
        join_players_with_nations(&game_data.nations, &id_player_nations, &option_snek_details)?;

    let state_details = if game_data.turn < 0 {
        let uploaded_players_detail: Vec<UploadingPlayer> = player_details
            .into_iter()
            .map(|potential_player_detail| {
                match potential_player_detail {
                    potential_player @ PotentialPlayer::GameOnly(_) => {
                        UploadingPlayer {
                            potential_player,
                            uploaded: true, // all players we can see have uploaded
                        }
                    }
                    potential_player @ PotentialPlayer::RegisteredAndGame(_, _) => {
                        UploadingPlayer {
                            potential_player,
                            uploaded: true, // all players we can see have uploaded
                        }
                    }
                    potential_player @ PotentialPlayer::RegisteredOnly(_, _, _) => {
                        UploadingPlayer {
                            potential_player,
                            uploaded: false, // all players we can't see have not uploaded
                        }
                    }
                }
            })
            .collect();

        StartedStateDetails::Uploading(UploadingState {
            uploading_players: uploaded_players_detail,
        })
    } else {
        let total_mins_remaining = game_data.turn_timer / (1000 * 60);
        let hours_remaining = total_mins_remaining / 60;
        let mins_remaining = total_mins_remaining - hours_remaining * 60;
        StartedStateDetails::Playing(PlayingState {
            players: player_details,
            mins_remaining,
            hours_remaining,
            turn: game_data.turn as u32, // game_data >= 0 checked above
        })
    };

    let started_details = StartedDetails {
        address: started_state.address.clone(),
        game_name: game_data.game_name.clone(),
        state: state_details,
    };

    Ok(GameDetails {
        alias: alias.to_owned(),
        owner: option_lobby_state.map(|lobby_state| lobby_state.owner.clone()),
        description: option_lobby_state.and_then(|lobby_state| lobby_state.description.clone()),
        nations: NationDetails::Started(started_details),
        cache_entry: Some(CacheEntry {
            game_data: game_data.clone(),
            option_snek_state: option_snek_details.clone(),
        }),
    })
}

fn details_to_embed(details: GameDetails, context: &Context) -> Result<CreateEmbed, CommandError> {
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
                            PotentialPlayer::RegisteredOnly(_, _, _) => continue,
                            PotentialPlayer::RegisteredAndGame(user_id, player_details) => {
                                (Some(user_id), player_details)
                            }
                            PotentialPlayer::GameOnly(player_details) => (None, player_details),
                        };

                        let player_name = if let NationStatus::Human = player_details.player_status
                        {
                            match option_user_id {
                                Some(user_id) => format!("**{}**", user_id.to_user((&context.cache, context.http.as_ref()))?),
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
                            "`{}` {} ({}): {}\n",
                            submission_symbol,
                            player_details.nation_name,
                            player_details.nation_id,
                            player_name,
                        ));
                    }

                    // This is pretty hacky
                    let mut e = CreateEmbed::default();
                    e.title("Details").field(
                        embed_title,
                        embed_texts[0].clone(),
                        false,
                    );
                    for embed_text in &embed_texts[1..] {
                        e.field("-----", embed_text, false);
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
                            Some(user_id) => format!("**{}**", user_id.to_user((&context.cache, context.http.as_ref()))?),
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
                            "`{}` {} ({}): {}\n",
                            player_submitted_status,
                            uploading_player.nation_name(),
                            uploading_player.nation_id(),
                            player_name,
                        ));
                    }
                    // This is pretty hacky
                    let mut e = CreateEmbed::default();
                    e.title("Details").field(
                        embed_title,
                        embed_texts[0].clone(),
                        false,
                    );
                    for embed_text in &embed_texts[1..] {
                        e.field("-----", embed_text, false);
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
                    let discord_user = lobby_player.player_id.to_user((&context.cache, context.http.as_ref()))?;
                    if ix % 20 == 0 {
                        embed_texts.push(String::new());
                    }
                    let new_len = embed_texts.len();
                    embed_texts[new_len - 1].push_str(&format!(
                        "{} ({}): {}\n",
                        lobby_player.nation_name, lobby_player.nation_id, discord_user,
                    ));
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
            let mut e = CreateEmbed::default();
            e.title("Details").field(
                embed_title,
                embed_texts[0].clone(),
                false,
            );
            for embed_text in &embed_texts[1..] {
                e.field("-----", embed_text, false);
            }
            e
        }
    };
    for owner in details.owner {
        e.field("Owner", owner.to_user((&context.cache, context.http.as_ref()))?.to_string(), false);
    }

    for description in details.description {
        if !description.is_empty() {
            e.field("Description", description, false);
        }
    }
    Ok(e)
}
