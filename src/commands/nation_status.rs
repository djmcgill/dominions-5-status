use ::server::get_game_data;
use ::ServerList;

fn show_submitted(submitted: u8) -> &'static str {
    match submitted {
        0 => ":x:",
        1 => ":alarm_clock:",
        2 => ":white_check_mark:",
        _ => "unrecognised submission status",
    }
}

fn show_status(status: u8) -> &'static str {
    match status {
        0 => "Empty",
        1 => "Human",
        2 => "AI",
        3 => "Independent",
        253 => "Closed",
        254 => "Defeated this turn",
        255 => "Defeated",
        _ => "Unrecognised status",
    }
}

use server::Nation;
fn show_nation_details(nation: &Nation) -> (String, String, String, Option<String>) {
    if nation.status == 1 {
            (nation.name.clone(),
            nation.era.clone(),
            show_status(nation.status).to_string(),
            Some(show_submitted(nation.submitted).to_string())
            )
    } else {
        (
            nation.name.clone(),
            nation.era.clone(),
            show_status(nation.status).to_string(),
            None
        )
    }
}

command!(nation_status(context, message, args) {
    println!{"nation_status message: {:?}", message};
    let data = context.data.lock();
    let server_list = data.get::<ServerList>().ok_or("No ServerList was created on startup. This is a bug.")?;
    let alias = args.single::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?;
    let ref server = server_list.get(&alias).ok_or(format!("Could not find server {}", alias))?;
    let ref server_address = server.address;
    let game_data = get_game_data(&server_address)?;
    
    let mut response = format!("{}: turn {}\n```\n", game_data.game_name, game_data.turn);
    let mut nation_data: Vec<(String, String, String, Option<String>)> = vec![];
    for nation in game_data.nations {
        nation_data.push(show_nation_details(&nation))
    }
    let longest_name_length = nation_data.iter().map(|&(ref x, _, _, _)| x.len()).max().unwrap();

    for (name, era, status, opt_submitted) in nation_data {
        let status_str =
            if status == "Human" {
                server
                    .players
                    .iter()
                    .find(|&(_, x)| x.nation_name == name)
                    .map(|(user_id, _)| user_id.get().unwrap().name)
                    .unwrap_or("Human".to_string())
            } else {
                status
            };
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
        let opt_user_id = server.players.iter().find(|&(_, x)| x.nation_name == name);
        for (user_id, _) in opt_user_id {
            response.push_str(&user_id.get().unwrap().name);
        }
        response.push_str(&"\n");
    } 
    response.push_str(&"```\n");

    println!("responding with {}", response);
    let _ = message.reply(&response);    
});
