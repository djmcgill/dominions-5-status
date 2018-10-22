use server::ServerConnection;
use super::alias_from_arg_or_channel_name;

use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::channel::Message;
use serenity::builder::CreateEmbed;

use model::{GameServerState, LobbyState, StartedState};
use model::enums::{NationStatus, Nations, SubmissionStatus};
use db::{DbConnection, DbConnectionKey};
use std::collections::HashMap;

#[cfg(test)]
mod tests;

pub fn details_helper<C: ServerConnection>(
    db_conn: &DbConnection,
    alias: &str,
) -> Result<CreateEmbed, CommandError> {
    let server = db_conn.game_for_alias(&alias)?;
    info!("got server details");

    let details: Details = match server.state {
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

    let mut nations = String::new();
    let mut players = String::new();
    let mut submitted = String::new();
    for details_line in details.lines {
        nations.push_str(&details_line.nation);
        players.push_str(&details_line.player);
        for submitted_line in details_line.submitted {
            submitted.push_str(&submitted_line);
        }
    }
    let mut embed_response: CreateEmbed = CreateEmbed::default()
        .title(details.title);

    if !submitted.is_empty() {
        embed_response = embed_response
            .field("?", submitted, true);
    }

    embed_response = embed_response
        .field("Nation", nations, true)
        .field("Player", players, true);
    for owner in details.owner {
        embed_response = embed_response.field("Owner", owner, false);
    }
    for description in details.description {
        embed_response = embed_response.field("Description", description, false);
    }
    Ok(embed_response)
}

pub fn details<C: ServerConnection>(
    context: &mut Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>()
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
) -> Result<Details, CommandError> {
    let embed_title = format!("{} ({} Lobby)", alias, lobby_state.era);
    let players_nations = db_conn.players_with_nations_for_game_alias(&alias)?;
    let registered_player_count = players_nations.len() as i32;

    let mut details_lines: Vec<DetailsLine> = vec![];

    for (player, nation_id) in players_nations {
        let &(nation_name, era) = Nations::get_nation_desc(nation_id);
        let line = DetailsLine {
            player: format!("{} \n", player.discord_user_id.to_user()?),
            submitted: None,
            nation: format!("{} {} ({})\n", era, nation_name, nation_id),
        };
        details_lines.push(line);
    }
    for _ in 0..(lobby_state.player_count - registered_player_count) {
        let line = DetailsLine {
            player: ".\n".to_string(),
            nation: "OPEN\n".to_string(),
            submitted: None,
        };
        details_lines.push(line);
    }
    let owner = lobby_state.owner.to_user()?;

    let details = Details {
        title: embed_title,
        lines: details_lines,
        owner: Some(owner.to_string()),
        description: lobby_state.description.clone(),
    };

    Ok(details)
}

fn uploading_from_lobby_details<C: ServerConnection>(
    db_conn: &DbConnection,
    started_state: &StartedState,
    lobby_state: &LobbyState,
    alias: &str,
) -> Result<Details, CommandError> {
    let server_address = &started_state.address;
    let game_data = C::get_game_data(&server_address)?;
    let mut details_lines: Vec<DetailsLine> = vec![];

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
        .filter(|&&(_, nation_id)|
            !players_uploaded_by_nation_id.contains_key(&nation_id)
        );

    for (&nation_id, _) in players_uploaded_by_nation_id.iter() {
        let player_name = id_player_registered_nations.iter()
            .find(|&&(_, found_nation_id)| nation_id == found_nation_id)
            .map(|&(ref p, _)|
                     format!("**{}**\n", p.discord_user_id.to_user().unwrap()))
            .unwrap_or_else(|| format!("{}\n", NationStatus::Human.show()));
        let &(nation_name, era) = Nations::get_nation_desc(nation_id);

        let line = DetailsLine {
            player: player_name,
            submitted: Some(format!("{}\n", SubmissionStatus::Submitted.show())),
            nation: format!("{} {} ({})\n", era, nation_name, nation_id),
        };
        details_lines.push(line);
    }

    for &(ref player, nation_id) in players_not_uploaded {
        let &(nation_name, era) = Nations::get_nation_desc(nation_id);
        let line = DetailsLine {
            player: format!("**{}**\n", player.discord_user_id.to_user()?),
            nation: format!("{} {}\n", era, nation_name),
            submitted: Some(format!("{}\n", SubmissionStatus::NotSubmitted.show())),
        };
        details_lines.push(line);
    }

    let embed_title = format!(
        "{} ({}): Pretender uploading",
        game_data.game_name,
        started_state.address,
    );

    let details = Details {
        title: embed_title,
        lines: details_lines,
        owner: Some(lobby_state.owner.to_user()?.to_string()),
        description: lobby_state.description.clone(),
    };
    Ok(details)
}

fn started_from_lobby_details<C: ServerConnection>(
    db_conn: &DbConnection,
    started_state: &StartedState,
    lobby_state: &LobbyState,
    alias: &str,
) -> Result<Details, CommandError> {
    let server_address = &started_state.address;
    let mut game_data = C::get_game_data(&server_address)?;
    let mut details_lines: Vec<DetailsLine> = vec![];
    game_data
        .nations
        .sort_unstable_by(|a, b| a.name.cmp(&b.name));

    let id_player_nations = db_conn.players_with_nations_for_game_alias(&alias)?;

    for nation in &game_data.nations {
        debug!("Creating format for nation {} {}", nation.era, nation.name);

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

        let submitted_line = if let NationStatus::Human = nation.status {
            format!("{}\n", nation.submitted.show())

        } else {
            format!("{}\n", SubmissionStatus::Submitted.show())
        };
        let line = DetailsLine {
            nation: format!("{} {} ({})\n", nation.era, nation.name, nation.id),
            player: nation_string,
            submitted: Some(submitted_line)
        };
        details_lines.push(line);
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
        let line = DetailsLine {
            nation: format!("{} {} ({})\n", era, nation_name, nation_id),
            player: format!("**{}**\n", player.discord_user_id.to_user()?),
            submitted: Some(format!("{}\n", SubmissionStatus::NotSubmitted.show())),
        };
        details_lines.push(line);
    }

    info!("Server details string created, now sending.");
    let total_mins_remaining = game_data.turn_timer / (1000 * 60);
    let hours_remaining = total_mins_remaining / 60;
    let mins_remaining = total_mins_remaining - hours_remaining * 60;

    info!("getting owner name");
    let embed_title = format!(
        "{} ({}): turn {}, {}h {}m remaining",
        game_data.game_name,
        started_state.address,
        game_data.turn,
        hours_remaining,
        mins_remaining
    );

    let owner = lobby_state.owner.to_user()?;

    let details = Details {
        title: embed_title,
        lines: details_lines,
        owner: Some(format!("{}", owner)),
        description: lobby_state.description.clone(),
    };
    Ok(details)
}

fn started_details<C: ServerConnection>(
    db_conn: &DbConnection,
    started_state: &StartedState,
    alias: &str,
) -> Result<Details, CommandError> {
    let server_address = &started_state.address;
    let mut game_data = C::get_game_data(&server_address)?;
    game_data
        .nations
        .sort_unstable_by(|a, b| a.name.cmp(&b.name));

    let id_player_nations = db_conn.players_with_nations_for_game_alias(&alias)?;

    let mut details_lines = vec![];
    for nation in &game_data.nations {
        debug!("Creating format for nation {} {}", nation.era, nation.name);

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


        let submitted = if let NationStatus::Human = nation.status {
            format!("{}\n", nation.submitted.show())
        } else {
            ".\n".to_string()
        };
        let line = DetailsLine {
            submitted: Some(submitted),
            player: format!("{}\n", nation_string),
            nation: format!("{} {} ({})\n", nation.era, nation.name, nation.id),
        };
        details_lines.push(line);
    }
    if game_data.nations.is_empty() {
        details_lines.push(
            DetailsLine {
                submitted: Some("-".to_string()),
                player: "-".to_string(),
                nation: "-".to_string(),
            }
        );
    }
    info!("Server details string created, now sending.");
    let total_mins_remaining = game_data.turn_timer / (1000 * 60);
    let hours_remaining = total_mins_remaining / 60;
    let mins_remaining = total_mins_remaining - hours_remaining * 60;

    let embed_title = format!(
        "{} ({}): turn {}, {}h {}m remaining",
        game_data.game_name,
        started_state.address,
        game_data.turn,
        hours_remaining,
        mins_remaining
    );

    let details = Details {
        title: embed_title,
        lines: details_lines,
        owner: None,
        description: None,
    };
    Ok(details)
}
// lobbies don't have submitted
// lobbies have description
struct DetailsLine {
    nation: String,
    player: String,
    submitted: Option<String>,
}
struct Details {
    title: String,
    lines: Vec<DetailsLine>,
    owner: Option<String>,
    description: Option<String>,
}
