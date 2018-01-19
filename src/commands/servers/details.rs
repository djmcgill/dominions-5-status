use ::server::get_game_data;

use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::Message;

use model::GameServerState;
use model::enums::NationStatus;
use db::DbConnectionKey;

pub fn details(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or("No DbConnection was created on startup. This is a bug.")?;
    let alias = args.single_quoted::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?;
    if !args.is_empty() {
        return Err(CommandError::from("Too many arguments. TIP: spaces in arguments need to be quoted \"like this\""));
    }
    let server = db_conn.game_for_alias(&alias)?;

    match server.state {
        GameServerState::Lobby => Err(CommandError::from("lobbies not yet supported")),
        GameServerState::StartedState(started_state) => {
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
                    if let Some(&(_, ref player, _)) = id_player_nations.iter().find(
                        |&&(_, _, nation_id)| nation_id == nation.id
                        ) {
                            // TODO: escape this?
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

            message.channel_id.send_message(|m| m
                .embed(|e| e
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
                    )
                )        
            )?;
            Ok(())
        }
    }
}
