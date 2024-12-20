use crate::{
    commands::servers::{
        details::started_details_from_server, discord_date_format, CommandResponse,
    },
    db::*,
    model::{
        enums::*,
        game_server::GameServerState,
        game_state::{
            GameDetails, NationDetails, PlayingState, PotentialPlayer, StartedStateDetails,
            UploadingState,
        },
    },
    snek::SnekGameStatus,
    DetailsCacheHandle,
};
use log::*;
use serenity::{
    framework::standard::{Args, CommandError},
    model::id::{ChannelId, UserId},
    prelude::Context,
};
use std::sync::Arc;

async fn turns_helper(
    user_id: UserId,
    db_conn: DbConnection,
    read_handle: DetailsCacheHandle,
) -> Result<Vec<String>, CommandError> {
    debug!("Starting !turns");
    let mut lines = vec![];
    let servers_and_nations_for_player = db_conn.servers_for_player(user_id)?;

    lines.push("Your turns:".to_string());
    let db_conn = &db_conn;
    for (server, _) in servers_and_nations_for_player {
        if let GameServerState::StartedState(started_state, option_lobby_state) = server.state {
            match read_handle.get_clone(&server.alias).await {
                Ok(cache) => {
                    let details: GameDetails = started_details_from_server(
                        db_conn.clone(),
                        &started_state,
                        option_lobby_state.as_ref(),
                        &server.alias,
                        &cache.game_data,
                        cache.option_snek_state.as_ref(),
                    )?;

                    match details.nations {
                        NationDetails::Started(started_state) => match started_state.state {
                            StartedStateDetails::Uploading(uploading_state) => {
                                let mut texts = turns_for_uploading_state(
                                    &uploading_state,
                                    user_id,
                                    &server.alias,
                                    details
                                        .cache_entry
                                        .and_then(|cache_entry| cache_entry.option_snek_state)
                                        .as_ref(),
                                );
                                lines.append(&mut texts);
                            }
                            StartedStateDetails::Playing(playing_state) => {
                                let mut texts = turns_for_playing_state(
                                    &playing_state,
                                    user_id,
                                    &server.alias,
                                    details
                                        .cache_entry
                                        .and_then(|cache_entry| cache_entry.option_snek_state)
                                        .as_ref(),
                                );
                                lines.append(&mut texts);
                            }
                        },
                        NationDetails::Lobby(_) => continue,
                    }
                }
                Err(err) => {
                    lines.push(format!("{}: ERROR {}", server.alias, err));
                }
            }
        }
    }
    Ok(lines)
}

fn turns_for_uploading_state(
    uploading_state: &UploadingState,
    user_id: UserId,
    alias: &str,
    option_snek_state: Option<&SnekGameStatus>,
) -> Vec<String> {
    let mut texts = vec![];
    let player_count = uploading_state.uploading_players.len();
    let uploaded_player_count = uploading_state
        .uploading_players
        .iter()
        .filter(|uploading_player| match uploading_player.potential_player {
            PotentialPlayer::GameOnly(_) => true,
            PotentialPlayer::RegisteredAndGame(_, _) => true,
            PotentialPlayer::RegisteredOnly(_, _) => false,
        })
        .count();

    for uploading_player in &uploading_state.uploading_players {
        match &uploading_player.potential_player {
            // This isn't them - it's somebody not registered
            PotentialPlayer::GameOnly(_) => (),
            PotentialPlayer::RegisteredAndGame(registered_player, player_details) =>
            // is this them?
            {
                // FIXME: there used to be a nation_id check on here. What is this for?
                //        does it fail only when people are registered multiple times?
                if registered_player.discord_user_id == user_id {
                    let turn_str = format!(
                        "{} uploading: {} (uploaded: {}, {}/{})",
                        alias,
                        player_details.nation_identifier.name(option_snek_state),
                        SubmissionStatus::Submitted.show(),
                        uploaded_player_count,
                        player_count,
                    );
                    texts.push(turn_str);
                }
            }
            // If this is them, they haven't uploaded
            PotentialPlayer::RegisteredOnly(registered_player, registered_nation_identifier) => {
                // FIXME: there used to be a nation_id check on here. What is this for?
                //        does it fail only when people are registered multiple times?
                if registered_player.discord_user_id == user_id {
                    let turn_str = format!(
                        "{} uploading: {} (uploaded: {}, {}/{})",
                        alias,
                        registered_nation_identifier.name(option_snek_state),
                        SubmissionStatus::NotSubmitted.show(),
                        uploaded_player_count,
                        player_count,
                    );
                    texts.push(turn_str);
                }
            }
        }
    }
    texts
}

fn turns_for_playing_state(
    playing_state: &PlayingState,
    user_id: UserId,
    alias: &str,
    option_snek_state: Option<&SnekGameStatus>,
) -> Vec<String> {
    let mut texts = vec![];
    let (playing_players, submitted_players) =
        count_playing_and_submitted_players(&playing_state.players[..]);

    for playing_player in &playing_state.players {
        match playing_player {
            // if they're registered, and there's bot nations, notify them anyway
            PotentialPlayer::RegisteredOnly(potential_player, bot_nation)
                if playing_state.modded_nations =>
            {
                if potential_player.discord_user_id == user_id {
                    let deadline = discord_date_format(playing_state.turn_deadline);
                    let turn_str = format!(
                        "{} turn {} ({}): {} (submitted: {}, {}/{})",
                        alias,
                        playing_state.turn,
                        deadline,
                        bot_nation.name(option_snek_state),
                        SubmissionStatus::NotSubmitted.show(),
                        submitted_players,
                        playing_players,
                    );
                    texts.push(turn_str);
                }
            }
            PotentialPlayer::RegisteredAndGame(potential_player, potential_player_details) => {
                // FIXME: there used to be a nation_id check on here. What is this for?
                //        does it fail only when people are registered multiple times?
                if potential_player.discord_user_id == user_id
                    && potential_player_details.player_status.is_human()
                {
                    let deadline = discord_date_format(playing_state.turn_deadline);

                    let turn_str = format!(
                        "{} turn {} ({}): {} (submitted: {}, {}/{})",
                        alias,
                        playing_state.turn,
                        deadline,
                        potential_player_details
                            .nation_identifier
                            .name(option_snek_state),
                        potential_player_details.submitted.show(),
                        submitted_players,
                        playing_players,
                    );
                    texts.push(turn_str);
                }
            }
            _ => (),
        }
    }
    texts
}

fn count_playing_and_submitted_players(players: &[PotentialPlayer]) -> (u32, u32) {
    let mut playing_players = 0;
    let mut submitted_players = 0;
    for playing_player in players {
        match playing_player {
            PotentialPlayer::RegisteredOnly(_, _) => (),
            PotentialPlayer::RegisteredAndGame(_, player_details) => {
                if player_details.player_status.is_human() {
                    playing_players += 1;
                    if let SubmissionStatus::Submitted = player_details.submitted {
                        submitted_players += 1;
                    }
                }
            }
            PotentialPlayer::GameOnly(player_details) => {
                if player_details.player_status.is_human() {
                    playing_players += 1;
                    if let SubmissionStatus::Submitted = player_details.submitted {
                        submitted_players += 1;
                    }
                }
            }
        }
    }
    (playing_players, submitted_players)
}

pub async fn turns(
    context: &Context,
    _channel_id: ChannelId,
    user_id: UserId,
    _args: Args,
) -> Result<CommandResponse, CommandError> {
    let read_handle = DetailsCacheHandle(Arc::clone(&context.data));
    let db_conn = {
        let data = context.data.read().await;
        data.get::<DbConnectionKey>()
            .ok_or_else(|| CommandError::from("No db connection"))?
            .clone()
    };
    let lines = turns_helper(user_id, db_conn, read_handle).await?;
    let messages = lines.chunks(20);
    let private_channel = user_id.create_dm_channel(&context.http).await?;

    for message_lines in messages {
        let text = message_lines.join("\n");
        info!("turns: replying with: {}", text);
        private_channel.say(&context.http, &text).await?;
    }
    Ok(CommandResponse::Reply("DM sent".to_owned()))
}
