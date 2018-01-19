use serenity::framework::standard::CommandError;
use serenity::prelude::Context;
use serenity::model::Message;

use db::DbConnectionKey;
use model::GameServerState;

pub fn list_servers(context: &mut Context, message: &Message) -> Result<(), CommandError> {
    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or_else(|| CommandError("No db connection".to_string()))?;
    let server_list = db_conn.retrieve_all_servers().map_err(CommandError::from)?;
    let mut text = String::new();
    text.push_str(&"Servers:\n");
    for (_, server) in server_list {
        match server.state {
            GameServerState::Lobby => 
                text.push_str(&format!("{} (-)\n", server.alias)),
            
            GameServerState::StartedState(started_state) => 
                text.push_str(&format!("{} ({})\n", server.alias, started_state.address)),
        }

    }
    message.reply(&text)?;
    Ok(())
}
