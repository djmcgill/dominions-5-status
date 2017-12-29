use ::server::get_game_data;
use ::nations;
use ::ServerList;

command!(nation_status(context, message, args) {
    println!{"nation_status message: {:?}", message};
    let data = context.data.lock();
    let server_list = data.get::<ServerList>().ok_or("No ServerList was created on startup. This is a bug.")?;
    let alias = args.single::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?;
    let server_address = server_list.get(&alias).ok_or(format!("Could not find server {}", alias))?;
    let data = get_game_data(&server_address)?;
    let mut response = String::new();
    for i in 0..250 {
        let status_num = data.f[i];        
        if status_num != 0 && status_num != 3 {
            let submitted = data.f[i+250];
            let connected = data.f[i+500];
            let nation_name = nations::get_nation_desc(i-1); // why -1? No fucking idea
            response.push_str(&format!(
                "name: {}, status: {}, submitted: {}, connected: {}\n", nation_name, status_num, submitted, connected
            ))
        }
    }
    println!("responding with {}", response);
    let _ = message.reply(&response);    
});
