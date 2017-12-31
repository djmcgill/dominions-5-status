use ::server::get_game_data;
use ::nations;
use ::ServerList;

/*
status = {
    0: "Empty",
    1: "Human",
    2: "AI",
    3: "Independent",
    253: "Closed",
    254: "Defeated this turn",
    255: "Defeated"
    }

self.submitted = 0  # 0 = not, 1 = partially, 2 = submitted

ns.connected = dataArray[i + PACKET_NUM_NATIONS * 2] == 1

*/

fn show_submitted(submitted: u8) -> &'static str {
    match submitted {
        0 => "No",
        1 => "Partial",
        2 => "Yes",
        _ => "unrecognised submission status"
    }
}

fn show_connected(connected: u8) -> &'static str {
    match connected {
        0 => "No",
        1 => "Yes",
        _ => "unrecognised submission status"
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

command!(nation_status(context, message, args) {
    println!{"nation_status message: {:?}", message};
    let data = context.data.lock();
    let server_list = data.get::<ServerList>().ok_or("No ServerList was created on startup. This is a bug.")?;
    let alias = args.single::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?;
    let server_address = &server_list.get(&alias).ok_or(format!("Could not find server {}", alias))?.address;
    let data = get_game_data(server_address)?;
    let mut response = String::new();
    for i in 0..250 {
        let status_num = data.f[i];        
        if status_num != 0 && status_num != 3 {
            let submitted = data.f[i+250];
            let connected = data.f[i+500];
            let nation_name = nations::get_nation_desc(i-1); // why -1? No fucking idea
            response.push_str(&format!(
                "name: {}, status: {}, submitted: {}, connected: {}\n",
                    nation_name,
                    show_status(status_num),
                    show_submitted(submitted),
                    show_connected(connected),
            ))
        }
    }
    println!("responding with {}", response);
    let _ = message.reply(&response);    
});
