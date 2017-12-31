use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::Message;
use commands::servers::ServerList;

use server::get_game_data;
use commands::servers::Player;
use ::nations;

pub fn pm_players(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    println!("pm_players");
    let alias = args.single::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?;

    let data = context.data.lock();
    let server_list = data.get::<ServerList>().ok_or("No ServerList was created on startup. This is a bug.")?;
    let server = server_list.get(&alias).ok_or(format!("Could not find server {}", alias))?;
    
    println!("ready to start pming players");
    for (user_id, player) in &server.players {
        let text = format!("Hi, you are {} in {}", player.nation_name, alias);
        println!("telling {} {}", user_id, text);
        let private_channel = user_id.create_dm_channel()?;
        private_channel.say(&text)?;
    }
    println!("finished PMing players");
    Ok(())

}

pub fn show_registered(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    println!("show_registered");
    let alias = args.single::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?;

    let data = context.data.lock();
    let server_list = data.get::<ServerList>().ok_or("No ServerList was created on startup. This is a bug.")?;
    let server = server_list.get(&alias).ok_or(format!("Could not find server {}", alias))?;
    let text = format!("server {} has players {:?}", alias, server.players);
    println!("replying with {}", text);
    let _ = message.reply(&text);
    Ok(())
}

pub fn register_player(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    let nation_name = args.single::<String>()?.to_lowercase();   
    let alias = args.single::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?;

    let mut data = context.data.lock();
    let server_list = data.get_mut::<ServerList>().ok_or("No ServerList was created on startup. This is a bug.")?;

    let server = server_list.get_mut(&alias).ok_or(format!("Could not find server {}", alias))?;
    let server_address = server.address.clone();
    let data = get_game_data(&server_address)?;
    let mut nation_names_in_game = vec![];

    for i in 0..250 {
        let status_num = data.f[i];        
        if status_num != 0 && status_num != 3 {
            nation_names_in_game.push(nations::get_nation_desc(i-1)); // TODO: this is real gross, abstract properly
        }
    }

    let nation = nation_names_in_game.iter().find(|name| // TODO: more efficient algo
        name.to_lowercase().starts_with(&nation_name) 
    ).ok_or(
        format!("Could not find nation starting with {}", nation_name)
    )?; 

    let ref user = message.author;
    let ref user_name: String = user.name;

    println!("registering nation {} for user {} in game {}", nation, user_name, data.game_name);
    let player = Player {
        nation_name: nation.to_string(),
        allowed_pms: true,
    }; 
    let _ = server.players.insert(user.id, player);
    Ok(())
}
