use serenity::framework::standard::CommandError;
use serenity::prelude::Context;
use serenity::model::Message;
use commands::servers::ServerList;
pub fn list_servers(context: &mut Context, message: &Message) -> Result<(), CommandError> {
    let data = context.data.lock();
    let server_list = data.get::<ServerList>().ok_or("No ServerList was created on startup. This is a bug.")?;
    let _ = message.reply(&format!("{:?}", server_list));
    Ok(())
}
