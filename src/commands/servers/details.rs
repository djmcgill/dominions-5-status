use ::server::get_game_data;

use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::Message;

use model::enums::nation_status::NationStatus;
use db::DbConnectionKey;

pub fn details(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    println!{"nation_status message: {:?}", message};
    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or("No DbConnection was created on startup. This is a bug.")?;
    let alias = args.single_quoted::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?;
    if !args.is_empty() {
        return Err(CommandError::from("Too many arguments. TIP: spaces in arguments need to be quoted \"like this\""));
    }
    let server = db_conn.game_for_alias(&alias)?;
    let ref server_address = server.address;
    let mut game_data = get_game_data(&server_address)?;
    game_data.nations.sort_unstable_by(|a, b| a.name.cmp(&b.name));

    let mut nation_names = String::new();
    let mut player_names = String::new();
    let mut submitted_status = String::new();

    let id_player_nations = db_conn.players_with_nations_for_game_alias(&alias)?;

    for nation in &game_data.nations {
        nation_names.push_str(&format!("{} {}\n", nation.era, nation.name));

        let nation_string = if let NationStatus::Human = nation.status {
            if let Some(&(_, ref player, _)) = id_player_nations.iter().find(
                |&&(_, _, nation_id)| nation_id == nation.id
                ) {
                    // TODO: escape this?
                    format!("**{}**", player.discord_user_id.get()?.name)    
                } else {
                    nation.status.show().to_string()
                }

        } else {
            nation.status.show().to_string()
        };

        player_names.push_str(&format!("{}\n", nation_string));
        
        submitted_status.push_str(&format!("{}\n", nation.submitted.show()));
    }
    let res = message.channel_id.send_message(|m| m
        .embed(|e| e
            .title(format!("{}: turn {}, ??:?? remaining", game_data.game_name, game_data.turn))
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
    );

    println!("CALL RESULT: {:?}", res);
    //     let status_str = status;
    //         // if status == "Human" { // FIXME
    //         //     server
    //         //         .players
    //         //         .iter()
    //         //         .find(|&(_, x)| x.nation_name == name)
    //         //         .map(|(user_id, _)| user_id.get().unwrap().name)
    //         //         .unwrap_or("Human".to_string())
    //         // } else {
    //         //     status
    //         // };
    //     let x = format!(
    //         "{:name_len$} ({}): {}",
    //         name,
    //         era,
    //         status_str,
    //         name_len = longest_name_length,
    //     );
    //     response.push_str(&x);
    //     for submitted in opt_submitted {
    //         response.push_str(&format!(" ({}) ", submitted));
    //     }
    //     // let opt_user_id = server.players.iter().find(|&(_, x)| x.nation_name == name);
    //     // for (user_id, _) in opt_user_id {
    //     //     response.push_str(&user_id.get().unwrap().name);
    //     // }
    //     response.push_str(&"\n");
    // } 
    // response.push_str(&"```\n");

    // println!("responding with {}", response);
    // let _ = message.reply(&response);    
    Ok(())
}
