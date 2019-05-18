use super::alias_from_arg_or_channel_name;
use crate::server::ServerConnection;

use serenity::builder::CreateEmbed;
use serenity::framework::standard::{Args, CommandError};
use serenity::model::channel::Message;
use serenity::prelude::Context;

use crate::db::{DbConnection, DbConnectionKey};
use crate::model::enums::{NationStatus, Nations, SubmissionStatus};
use crate::model::{GameServerState, LobbyState, StartedState};
use log::*;
use std::collections::HashMap;

#[cfg(test)]
mod tests;

mod mod2;
pub use mod2::details_2; // this is the rewritten version

pub fn details_helper<C: ServerConnection>(
    db_conn: &DbConnection,
    alias: &str,
) -> Result<CreateEmbed, CommandError> {
    let server = db_conn.game_for_alias(&alias)?;
    info!("got server details");

    let embed_response = match server.state {
        GameServerState::Lobby(lobby_state) => lobby_details(db_conn, &lobby_state, &alias)?,
        GameServerState::StartedState(started_state, None) => {
            started_details::<C>(db_conn, &started_state, &alias)?
        }
        GameServerState::StartedState(started_state, Some(lobby_state)) => {
            if started_state.last_seen_turn == -1 {
                uploading_from_lobby_details::<C>(db_conn, &started_state, &lobby_state, &alias)?
            } else {
                started_from_lobby_details::<C>(db_conn, &started_state, &lobby_state, &alias)?
            }
        }
    };
    Ok(embed_response)
}

pub fn details<C: ServerConnection>(
    context: &mut Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let data = context.data.lock();
    let db_conn = data
        .get::<DbConnectionKey>()
        .ok_or("No DbConnection was created on startup. This is a bug.")?;
    let alias = alias_from_arg_or_channel_name(&mut args, &message)?;
    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
        ));
    }

    let embed_response = details_helper::<C>(db_conn, &alias)?;
    message
        .channel_id
        .send_message(|m| m.embed(|_| embed_response))?;
    Ok(())
}

fn lobby_details(
    db_conn: &DbConnection,
    lobby_state: &LobbyState,
    alias: &str,
) -> Result<CreateEmbed, CommandError> {
    let embed_title = format!("{} ({} Lobby)", alias, lobby_state.era);
    let players_nations = db_conn.players_with_nations_for_game_alias(&alias)?;
    let registered_player_count = players_nations.len() as i32;

    let mut player_names = String::new();
    let mut nation_names = String::new();

    for (player, nation_id) in players_nations {
        let &(nation_name, era) = Nations::get_nation_desc(nation_id);
        player_names.push_str(&format!("{} \n", player.discord_user_id.to_user()?));
        nation_names.push_str(&format!("{} {} ({})\n", era, nation_name, nation_id));
    }
    for _ in 0..(lobby_state.player_count - registered_player_count) {
        player_names.push_str(&".\n");
        nation_names.push_str(&"OPEN\n");
    }
    let owner = lobby_state.owner.to_user()?;
    let e_temp = CreateEmbed::default()
        .title(embed_title)
        .field("Nation", nation_names, true)
        .field("Player", player_names, true)
        .field("Owner", format!("{}", owner), false);
    let e = match lobby_state.description {
        Some(ref description) if !description.is_empty() => {
            e_temp.field("Description", description, false)
        }
        _ => e_temp,
    };

    Ok(e)
}

fn uploading_from_lobby_details<C: ServerConnection>(
    db_conn: &DbConnection,
    started_state: &StartedState,
    lobby_state: &LobbyState,
    alias: &str,
) -> Result<CreateEmbed, CommandError> {
    let server_address = &started_state.address;
    let game_data = C::get_game_data(&server_address)?;

    let players_uploaded_by_nation_id = {
        let mut hash_map = HashMap::with_capacity(game_data.nations.len());
        for nation in &game_data.nations {
            let _ = hash_map.insert(nation.id, nation.clone());
        }
        hash_map
    };

    let id_player_registered_nations = db_conn.players_with_nations_for_game_alias(&alias)?;
    let players_not_uploaded = id_player_registered_nations
        .iter()
        .filter(|&&(_, nation_id)| !players_uploaded_by_nation_id.contains_key(&nation_id));

    let mut nation_names = String::new();
    let mut player_names = String::new();
    let mut submitted_status = String::new();

    for (&nation_id, _) in players_uploaded_by_nation_id.iter() {
        let player_name = id_player_registered_nations
            .iter()
            .find(|&&(_, found_nation_id)| nation_id == found_nation_id)
            .map(|&(ref p, _)| format!("**{}**\n", p.discord_user_id.to_user().unwrap()))
            .unwrap_or_else(|| format!("{}\n", NationStatus::Human.show()));
        let &(nation_name, era) = Nations::get_nation_desc(nation_id);
        nation_names.push_str(&format!("{} {} ({})\n", era, nation_name, nation_id));
        player_names.push_str(&player_name);
        submitted_status.push_str(&format!("{}\n", SubmissionStatus::Submitted.show()));
    }

    for &(ref player, nation_id) in players_not_uploaded {
        let &(nation_name, era) = Nations::get_nation_desc(nation_id);
        nation_names.push_str(&format!("{} {}\n", era, nation_name));
        player_names.push_str(&format!("**{}**\n", player.discord_user_id.to_user()?));
        submitted_status.push_str(&format!("{}\n", SubmissionStatus::NotSubmitted.show()));
    }

    let embed_title = format!(
        "{} ({}): Pretender uploading",
        game_data.game_name, started_state.address,
    );

    let owner = lobby_state.owner.to_user()?;
    let e_temp = CreateEmbed::default()
        .title(embed_title)
        .field("Nation", nation_names, true)
        .field("Player", player_names, true)
        .field("Uploaded", submitted_status, true)
        .field("Owner", format!("{}", owner), false);
    let e = match lobby_state.description {
        Some(ref description) if !description.is_empty() => {
            e_temp.field("Description", description, false)
        }
        _ => e_temp,
    };
    Ok(e)
}

fn started_from_lobby_details<C: ServerConnection>(
    db_conn: &DbConnection,
    started_state: &StartedState,
    lobby_state: &LobbyState,
    alias: &str,
) -> Result<CreateEmbed, CommandError> {
    let server_address = &started_state.address;
    let mut game_data = C::get_game_data(&server_address)?;
    game_data
        .nations
        .sort_unstable_by(|a, b| a.name.cmp(&b.name));

    let mut nation_names = String::new();
    let mut player_names = String::new();
    let mut submitted_status = String::new();

    let id_player_nations = db_conn.players_with_nations_for_game_alias(&alias)?;

    for nation in &game_data.nations {
        debug!("Creating format for nation {} {}", nation.era, nation.name);
        nation_names.push_str(&format!("{} {} ({})\n", nation.era, nation.name, nation.id));

        let nation_string = if let NationStatus::Human = nation.status {
            if let Some(&(ref player, _)) = id_player_nations
                .iter()
                .find(|&&(_, nation_id)| nation_id == nation.id)
            {
                format!("**{}**", player.discord_user_id.to_user()?)
            } else {
                nation.status.show().to_string()
            }
        } else {
            nation.status.show().to_string()
        };

        player_names.push_str(&format!("{}\n", nation_string));

        if let NationStatus::Human = nation.status {
            submitted_status.push_str(&format!("{}\n", nation.submitted.show()));
        } else {
            submitted_status.push_str(&format!("{}\n", SubmissionStatus::Submitted.show()));
        }
    }

    // TODO: yet again, not quadratic please
    let mut not_uploaded_players = id_player_nations.clone();
    not_uploaded_players.retain(|&(_, nation_id)| {
        game_data
            .nations
            .iter()
            .find(|ref nation| nation.id == nation_id)
            .is_none()
    });

    for &(ref player, nation_id) in &not_uploaded_players {
        let &(nation_name, era) = Nations::get_nation_desc(nation_id);
        nation_names.push_str(&format!("{} {} ({})\n", era, nation_name, nation_id));
        player_names.push_str(&format!("**{}**\n", player.discord_user_id.to_user()?));
        submitted_status.push_str(&format!("{}\n", SubmissionStatus::NotSubmitted.show()));
    }

    info!("Server details string created, now sending.");
    let total_mins_remaining = game_data.turn_timer / (1000 * 60);
    let hours_remaining = total_mins_remaining / 60;
    let mins_remaining = total_mins_remaining - hours_remaining * 60;

    info!("getting owner name");
    let embed_title = format!(
        "{} ({}): turn {}, {}h {}m remaining",
        game_data.game_name, started_state.address, game_data.turn, hours_remaining, mins_remaining
    );

    info!(
        "replying with embed_title {:?}\n nations {:?}\n players {:?}\n, submission {:?}",
        embed_title, nation_names, player_names, submitted_status
    );

    let owner = lobby_state.owner.to_user()?;
    let e_temp = CreateEmbed::default()
        .title(embed_title)
        .field("Nation", nation_names, true)
        .field("Player", player_names, true)
        .field("Submitted", submitted_status, true)
        .field("Owner", format!("{}", owner), false);
    let e = match lobby_state.description {
        Some(ref description) if !description.is_empty() => {
            e_temp.field("Description", description, false)
        }
        _ => e_temp,
    };
    Ok(e)
}

fn started_details<C: ServerConnection>(
    db_conn: &DbConnection,
    started_state: &StartedState,
    alias: &str,
) -> Result<CreateEmbed, CommandError> {
    let server_address = &started_state.address;
    let mut game_data = C::get_game_data(&server_address)?;
    game_data
        .nations
        .sort_unstable_by(|a, b| a.name.cmp(&b.name));

    let mut nation_names = String::new();
    let mut player_names = String::new();
    let mut submitted_status = String::new();

    let id_player_nations = db_conn.players_with_nations_for_game_alias(&alias)?;

    for nation in &game_data.nations {
        debug!("Creating format for nation {} {}", nation.era, nation.name);
        nation_names.push_str(&format!("{} {} ({})\n", nation.era, nation.name, nation.id));

        let nation_string = if let NationStatus::Human = nation.status {
            if let Some(&(ref player, _)) = id_player_nations
                .iter()
                .find(|&&(_, nation_id)| nation_id == nation.id)
            {
                format!("**{}**", player.discord_user_id.to_user()?)
            } else {
                nation.status.show().to_string()
            }
        } else {
            nation.status.show().to_string()
        };

        player_names.push_str(&format!("{}\n", nation_string));

        if let NationStatus::Human = nation.status {
            submitted_status.push_str(&format!("{}\n", nation.submitted.show()));
        } else {
            submitted_status.push_str(&".\n");
        }
    }
    if game_data.nations.is_empty() {
        nation_names.push_str(&"-");
        player_names.push_str(&"-");
        submitted_status.push_str(&"-");
    }
    info!("Server details string created, now sending.");
    let total_mins_remaining = game_data.turn_timer / (1000 * 60);
    let hours_remaining = total_mins_remaining / 60;
    let mins_remaining = total_mins_remaining - hours_remaining * 60;

    let embed_title = format!(
        "{} ({}): turn {}, {}h {}m remaining",
        game_data.game_name, started_state.address, game_data.turn, hours_remaining, mins_remaining
    );

    info!(
        "replying with embed_title {:?}\n nations {:?}\n players {:?}\n, submission {:?}",
        embed_title, nation_names, player_names, submitted_status
    );

    let e = CreateEmbed::default()
        .title(embed_title)
        .field("Nation", nation_names, true)
        .field("Player", player_names, true)
        .field("Submitted", submitted_status, true);
    Ok(e)
}
