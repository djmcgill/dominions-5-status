use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::Message;

use commands::servers::Server;
use commands::servers::ServerList;

pub fn add_server(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    let server_address = args.single::<String>()?;
    let alias = args.single::<String>().or_else(|_| {
        message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    })?;

    let mut data = context.data.lock();
    let server_list = data.get_mut::<ServerList>().ok_or("No ServerList was created on startup. This is a bug.")?;

    let server = Server::new(server_address);
    match server_list.insert(alias.clone(), server) {
        None => {
            let _ = message.reply(&format!("successfully inserted with name {}", alias));
        },
        Some(old) => {
            let _ = message.reply(&format!("successfully overwrote {} from {}", alias, old.address));
        },
    }
    println!("inserted, current contents is {:?}", server_list);
    Ok(())
}
