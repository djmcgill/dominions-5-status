use crate::commands::servers::discord_date_format;
use crate::{
    commands::servers::{alias_from_arg_or_channel_name, CommandResponse},
    db::{DbConnection, DbConnectionKey},
    model::{
        enums::{NationStatus, SubmissionStatus},
        game_data::GameData,
        game_server::*,
        game_state::*,
        nation::{BotNationIdentifier, Nation},
        player::Player,
    },
    server::get_game_data_async,
    snek::{snek_details_async, SnekGameStatus},
    DetailsCacheHandle,
};
use chrono::Utc;
use log::*;
use serenity::{
    builder::CreateEmbed,
    framework::standard::{Args, CommandError},
    model::id::{ChannelId, UserId},
    prelude::Context,
};
use std::{collections::HashMap, sync::Arc};

pub async fn details(
    context: &Context,
    channel_id: ChannelId,
    _user_id: UserId,
    mut args: Args,
) -> Result<CommandResponse, CommandError> {
    // TODO: It's a bit weird to pass the arc here and use it elsewhere
    let data_handle = DetailsCacheHandle(Arc::clone(&context.data));
    let db_conn = {
        let data = context.data.read().await;
        data.get::<DbConnectionKey>()
            .ok_or("No DbConnection was created on startup. This is a bug.")?
            .clone()
    };

    let alias = alias_from_arg_or_channel_name(context, channel_id, &mut args).await?;
    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
        ));
    }
    let embed_response = details_helper(&alias, db_conn, data_handle, context).await?;
    Ok(CommandResponse::Embed(embed_response))
}

pub async fn get_details_for_alias(
    db_conn: DbConnection,
    alias: &str,
) -> Result<GameDetails, CommandError> {
    let server = db_conn.game_for_alias(&alias)?;
    info!("got server details");

    let details = match server.state {
        GameServerState::Lobby(ref lobby_state) => lobby_details(db_conn, lobby_state, &alias)?,
        GameServerState::StartedState(ref started_state, ref option_lobby_state) => {
            started_details(db_conn, started_state, option_lobby_state.as_ref(), &alias).await?
        }
    };

    Ok(details)
}

async fn started_details(
    db_conn: DbConnection,
    started_state: &StartedState,
    option_lobby_state: Option<&LobbyState>,
    alias: &str,
) -> Result<GameDetails, CommandError> {
    let server_address = &started_state.address;
    let game_data = get_game_data_async(&server_address).await?;
    let option_snek_details = snek_details_async(server_address).await?;

    started_details_from_server(
        db_conn,
        started_state,
        option_lobby_state,
        alias,
        &game_data,
        option_snek_details.as_ref(),
    )
}

async fn details_helper(
    alias: &str,
    db_conn: DbConnection,
    read_handle: DetailsCacheHandle,
    context: &Context,
) -> Result<CreateEmbed, CommandError> {
    let server = db_conn.game_for_alias(&alias)?;
    match &server.state {
        GameServerState::Lobby(lobby_state) => {
            let details: GameDetails = lobby_details(db_conn, lobby_state, alias)?;
            let embed: CreateEmbed = details_to_embed(details, context).await?;
            Ok(embed)
        }
        GameServerState::StartedState(started_state, option_lobby_state) => {
            let cache = read_handle.get_clone(alias).await?;
            let CacheEntry {
                game_data,
                option_snek_state,
            } = cache;

            let details: GameDetails = started_details_from_server(
                db_conn,
                started_state,
                option_lobby_state.as_ref(),
                alias,
                &game_data,
                option_snek_state.as_ref(),
            )?;

            let embed: CreateEmbed = details_to_embed(details, context).await?;
            Ok(embed)
        }
    }
}

pub fn lobby_details(
    db_conn: DbConnection,
    lobby_state: &LobbyState,
    alias: &str,
) -> Result<GameDetails, CommandError> {
    let players_nations = db_conn.players_with_nations_for_game_alias(&alias)?;

    let mut player_nation_details: Vec<LobbyPlayer> = players_nations
        .into_iter()
        .map(|(player, nation_identifier)| {
            let name = nation_identifier.name(None);
            LobbyPlayer {
                player_id: player.discord_user_id,
                nation_identifier,
                cached_name: name,
            }
        })
        .collect();

    player_nation_details.sort_by(|n1, n2| n1.cached_name.cmp(&n2.cached_name));

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

/// Takes data from the three possible sources, and matches them together until we know
/// 1) who is in the game and the bot's record
/// 2) who is in the game but not the bot
/// 3) who is NOT in the game but is in the bot
fn join_players_with_nations(
    // from game
    nations: &[Nation],
    // from db
    players_nations: &[(Player, BotNationIdentifier)],
) -> Result<Vec<PotentialPlayer>, CommandError> {
    let mut potential_players = vec![];

    // Any players registered in the bot with a number go here
    let mut players_by_nation_id = HashMap::new();
    for (player, nation_identifier) in players_nations {
        if let Some(nation_id) = nation_identifier.id() {
            players_by_nation_id.insert(nation_id, (player, nation_identifier));
        } else {
            // The nation_identifier has no ID so it's a custom name so it's bot only
            potential_players.push(PotentialPlayer::RegisteredOnly(
                player.discord_user_id,
                nation_identifier.clone(),
            ));
        }
    }
    for nation in nations {
        match players_by_nation_id.remove(&nation.identifier.id()) {
            // Lobby and game
            Some((player, _)) => {
                let player_details = PlayerDetails {
                    nation_identifier: nation.identifier.clone(),
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
                nation_identifier: nation.identifier.clone(),
                submitted: nation.submitted,
                player_status: nation.status,
            })),
        }
    }
    // Lobby only RegisteredOnly(UserId, BotNationIdentifier),
    for (_, (player, bot_nation_identifier)) in players_by_nation_id {
        potential_players.push(PotentialPlayer::RegisteredOnly(
            player.discord_user_id,
            bot_nation_identifier.clone(),
        ));
    }
    Ok(potential_players)
}

pub fn started_details_from_server(
    db_conn: DbConnection,
    started_state: &StartedState,
    option_lobby_state: Option<&LobbyState>,
    alias: &str,
    game_data: &GameData,
    option_snek_details: Option<&SnekGameStatus>,
) -> Result<GameDetails, CommandError> {
    let id_player_nations = db_conn.players_with_nations_for_game_alias(&alias)?;
    let player_details = join_players_with_nations(&game_data.nations[..], &id_player_nations[..])?;

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
                    potential_player @ PotentialPlayer::RegisteredOnly(_, _) => {
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
        StartedStateDetails::Playing(PlayingState {
            players: player_details,
            turn_deadline: game_data.turn_deadline,
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
        owner: option_lobby_state.map(|lobby_state| lobby_state.owner),
        description: option_lobby_state.and_then(|lobby_state| lobby_state.description.clone()),
        nations: NationDetails::Started(started_details),
        cache_entry: Some(CacheEntry {
            game_data: game_data.clone(),
            option_snek_state: option_snek_details.cloned(),
        }),
    })
}

async fn details_to_embed(
    details: GameDetails,
    context: &Context,
) -> Result<CreateEmbed, CommandError> {
    let option_snek_state = details
        .cache_entry
        .and_then(|cache_entry| cache_entry.option_snek_state);

    let mut e = match details.nations {
        NationDetails::Started(started_details) => {
            match &started_details.state {
                StartedStateDetails::Playing(playing_state) => {
                    let deadline = discord_date_format(playing_state.turn_deadline);
                    let embed_title = format!(
                        "{} ({}): turn {}, {}",
                        started_details.game_name,
                        started_details.address,
                        playing_state.turn,
                        deadline,
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
                                Some(user_id) => format!(
                                    "**{}**",
                                    user_id
                                        .to_user((&context.cache, context.http.as_ref()))
                                        .await?
                                ),
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
                    let mut e = CreateEmbed::default();
                    e.title("Details")
                        .field(embed_title, embed_texts[0].clone(), false);
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
                            Some(user_id) => format!(
                                "**{}**",
                                user_id
                                    .to_user((&context.cache, context.http.as_ref()))
                                    .await?
                            ),
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
                    let mut e = CreateEmbed::default();
                    e.title("Details")
                        .field(embed_title, embed_texts[0].clone(), false);
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

            if !lobby_details.players.is_empty() {
                for (ix, lobby_player) in lobby_details.players.iter().enumerate() {
                    let discord_user = lobby_player
                        .player_id
                        .to_user((&context.cache, context.http.as_ref()))
                        .await?;
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
            let mut e = CreateEmbed::default();
            e.title("Details")
                .field(embed_title, embed_texts[0].clone(), false);
            for embed_text in &embed_texts[1..] {
                e.field("-----", embed_text, false);
            }
            e
        }
    };
    if let Some(owner) = details.owner {
        e.field(
            "Owner",
            owner
                .to_user((&context.cache, context.http.as_ref()))
                .await?
                .to_string(),
            false,
        );
    }

    if let Some(description) = details.description {
        if !description.is_empty() {
            e.field("Description", description, false);
        }
    }
    Ok(e)
}
