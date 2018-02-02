use ::server::get_game_data;

use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::Message;
use serenity::builder::CreateEmbed;

use model::{GameServerState, LobbyState, StartedState};
use model::enums::{Nations, NationStatus};
use db::{DbConnection, DbConnectionKey};

pub fn details_helper(db_conn: &DbConnection, alias: &str) -> Result<CreateEmbed, CommandError> {
    let server = db_conn.game_for_alias(&alias)?;

    let embed_response = match server.state {
        GameServerState::Lobby(lobby_state) => {
            lobby_details(
                db_conn,
                lobby_state,
                &alias,
            )?
        }
        GameServerState::StartedState(started_state, None) => {
            started_details(
                db_conn,
                started_state,
                &alias,
            )?
        }
        GameServerState::StartedState(started_state, Some(lobby_state)) => {
            started_from_lobby_details(
                db_conn,
                started_state,
                lobby_state,
                &alias,
            )?
        }
    };
    Ok(embed_response)
}

pub fn details(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or("No DbConnection was created on startup. This is a bug.")?;
    let alias = args.single_quoted::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?.to_lowercase();
    if !args.is_empty() {
        return Err(CommandError::from("Too many arguments. TIP: spaces in arguments need to be quoted \"like this\""));
    }

    let embed_response = details_helper(db_conn, &alias)?;
    message.channel_id.send_message(|m| m
        .embed(|_| embed_response)
    )?;
    Ok(())   
}

fn lobby_details(
    db_conn: &DbConnection,
    lobby_state: LobbyState,
    alias: &str,
) -> Result<CreateEmbed, CommandError> {
    let embed_title = format!("{} (Lobby)", alias);
    let players_nations = db_conn.players_with_nations_for_game_alias(&alias)?;
    let registered_player_count = players_nations.len() as i32;

    let mut player_names = String::new();
    let mut nation_names = String::new();

    for (player, nation_id) in players_nations {
        let &(nation_name, era) = Nations::get_nation_desc(nation_id);
        player_names.push_str(&format!("{}\n", player.discord_user_id.get()?));
        nation_names.push_str(&format!("{} {}\n", era, nation_name));
    }
    for _ in 0..(lobby_state.player_count - registered_player_count) {
        player_names.push_str(&".\n");
        nation_names.push_str(&"OPEN\n");
    }
    let e = CreateEmbed::default()
        .title(embed_title)
        .field( |f| f
            .name("Nation")
            .value(nation_names)
        )
        .field ( |f| f
            .name("Player")
            .value(player_names)
        );
    Ok(e)
}

fn started_from_lobby_details(
    db_conn: &DbConnection,
    started_state: StartedState,
    _lobby_state: LobbyState,
    alias: &str,
) -> Result<CreateEmbed, CommandError> {
    let ref server_address = started_state.address;
    let mut game_data = get_game_data(&server_address)?;
    game_data.nations.sort_unstable_by(|a, b| a.name.cmp(&b.name));

    let mut nation_names = String::new();
    let mut player_names = String::new();
    let mut submitted_status = String::new();

    let id_player_nations = db_conn.players_with_nations_for_game_alias(&alias)?;

    for nation in &game_data.nations {
        debug!("Creating format for nation {} {}", nation.era, nation.name);
        nation_names.push_str(&format!("{} {}\n", nation.era, nation.name));

        let nation_string = if let NationStatus::Human = nation.status {
            if let Some(&(ref player, _)) = id_player_nations.iter().find(
                |&&(_, nation_id)| nation_id == nation.id
                ) {
                    format!("**{}**", player.discord_user_id.get()?)    
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

    // TODO: yet again, not quadratic please
    let mut not_uploaded_players = id_player_nations.clone();
    not_uploaded_players.retain(
        |&(_, nation_id)| game_data.nations.iter().find(
            |ref nation| nation.id == nation_id
        ).is_none()
    );

    for &(ref player, _) in &not_uploaded_players {
        nation_names.push_str(&"NOT UPLOADED\n");
        player_names.push_str(&format!("**{}**\n", player.discord_user_id.get()?));
        submitted_status.push_str(&".\n");
    }

    info!("Server details string created, now sending.");
    let total_mins_remaining = game_data.turn_timer / (1000*60);
    let hours_remaining = total_mins_remaining/60;
    let mins_remaining = total_mins_remaining - hours_remaining*60;

    let embed_title = format!("{}: turn {}, {}h {}m remaining",
                game_data.game_name,
                game_data.turn,
                hours_remaining,
                mins_remaining);

    info!("replying with embed_title {:?}\n nations {:?}\n players {:?}\n, submission {:?}",
    embed_title, nation_names, player_names, submitted_status);

    let e = CreateEmbed::default()
        .title(embed_title)
        .field( |f| f
            .name("Nation")
            .value(nation_names)
        )
        .field ( |f| f
            .name("Player")
            .value(player_names)
        )
        .field ( |f| f
            .name("Submitted")
            .value(submitted_status)
        );
    Ok(e)
}

fn started_details(
    db_conn: &DbConnection,
    started_state: StartedState,
    alias: &str,
) -> Result<CreateEmbed, CommandError> {
    let ref server_address = started_state.address;
    let mut game_data = get_game_data(&server_address)?;
    game_data.nations.sort_unstable_by(|a, b| a.name.cmp(&b.name));

    let mut nation_names = String::new();
    let mut player_names = String::new();
    let mut submitted_status = String::new();

    let id_player_nations = db_conn.players_with_nations_for_game_alias(&alias)?;

    for nation in &game_data.nations {
        debug!("Creating format for nation {} {}", nation.era, nation.name);
        nation_names.push_str(&format!("{} {}\n", nation.era, nation.name));

        let nation_string = if let NationStatus::Human = nation.status {
            if let Some(&(ref player, _)) = id_player_nations.iter().find(
                |&&(_, nation_id)| nation_id == nation.id
                ) {
                    format!("**{}**", player.discord_user_id.get()?)    
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
    info!("Server details string created, now sending.");
    let total_mins_remaining = game_data.turn_timer / (1000*60);
    let hours_remaining = total_mins_remaining/60;
    let mins_remaining = total_mins_remaining - hours_remaining*60;

    let embed_title = format!("{}: turn {}, {}h {}m remaining",
                game_data.game_name,
                game_data.turn,
                hours_remaining,
                mins_remaining);

    info!("replying with embed_title {:?}\n nations {:?}\n players {:?}\n, submission {:?}",
    embed_title, nation_names, player_names, submitted_status);

    let e = CreateEmbed::default()
        .title(embed_title)
        .field( |f| f
            .name("Nation")
            .value(nation_names)
        )
        .field ( |f| f
            .name("Player")
            .value(player_names)
        )
        .field ( |f| f
            .name("Submitted")
            .value(submitted_status)
        );
    Ok(e)
}
