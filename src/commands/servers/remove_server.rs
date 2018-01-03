use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::Message;

pub fn remove_server(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    // let alias = args.single::<String>().or_else(|_| {
    //     message.channel_id.name().ok_or(format!("Could not find channel name for channel {}", message.channel_id))
    // })?;

    // let mut data = context.data.lock();
    // let server_list = data.get_mut::<ServerList>().ok_or("No ServerList was created on startup. This is a bug.")?;

    // let _ = server_list.remove(&alias);
    // let _ = message.reply(&format!("successfully removed server {}", alias));
    // println!("removed, current contents is {:?}", server_list);
    Ok(())
}
