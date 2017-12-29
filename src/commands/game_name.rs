use ::ServerList;
use ::server::get_game_data;

command!(game_name(context, message, args) {
    let data = context.data.lock();
    let server_list = data.get::<ServerList>().ok_or("No ServerList was created on startup. This is a bug.")?;;
    let alias = args.single::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?;
    let server_address = server_list.get(&alias).ok_or(format!("Could not find server {}", alias))?;
    let response = get_game_data(server_address)?.game_name;
    let _ = message.reply(&format!("Game name at {} is {}", server_address, response));
});
