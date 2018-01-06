use ::server::get_game_data;

use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::Message;

use model::nation::Nation;
use model::enums::nation_status::NationStatus;
use db::DbConnectionKey;

fn show_nation_details(nation: &Nation) -> (String, String, String, Option<String>) {
    if nation.status == NationStatus::Human {
            (nation.name.clone(),
            nation.era.clone(),
            nation.status.show().to_string(),
            Some(nation.submitted.show().to_string())
            )
    } else {
        (
            nation.name.clone(),
            nation.era.clone(),
            nation.status.show().to_string(),
            None
        )
    }
}

pub fn details(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    println!{"nation_status message: {:?}", message};
    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or("No DbConnection was created on startup. This is a bug.")?;
    let alias = args.single::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?;
    let server = db_conn.game_for_alias(&alias)?;
    let ref server_address = server.address;
    let game_data = get_game_data(&server_address)?;
    
    let mut response = format!("{}: turn {}\n```\n", game_data.game_name, game_data.turn);
    let mut nation_data: Vec<(String, String, String, Option<String>)> = vec![];
    for nation in game_data.nations {
        nation_data.push(show_nation_details(&nation))
    }
    let longest_name_length = nation_data.iter().map(|&(ref x, _, _, _)| x.len()).max().unwrap();

    for (name, era, status, opt_submitted) in nation_data {
        let status_str = status;
            // if status == "Human" { // FIXME
            //     server
            //         .players
            //         .iter()
            //         .find(|&(_, x)| x.nation_name == name)
            //         .map(|(user_id, _)| user_id.get().unwrap().name)
            //         .unwrap_or("Human".to_string())
            // } else {
            //     status
            // };
        let x = format!(
            "{:name_len$} ({}): {}",
            name,
            era,
            status_str,
            name_len = longest_name_length,
        );
        response.push_str(&x);
        for submitted in opt_submitted {
            response.push_str(&format!(" ({}) ", submitted));
        }
        // let opt_user_id = server.players.iter().find(|&(_, x)| x.nation_name == name);
        // for (user_id, _) in opt_user_id {
        //     response.push_str(&user_id.get().unwrap().name);
        // }
        response.push_str(&"\n");
    } 
    response.push_str(&"```\n");

    println!("responding with {}", response);
    let _ = message.reply(&response);    
    Ok(())
}
