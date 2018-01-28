use serenity::framework::standard::CommandError;
use serenity::prelude::Context;
use serenity::model::Message;

use db::DbConnectionKey;
use model::GameServerState;

pub fn list_servers(context: &mut Context, message: &Message) -> Result<(), CommandError> {
    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or_else(|| CommandError("No db connection".to_string()))?;
    let server_list = db_conn.retrieve_all_servers().map_err(CommandError::from)?;
    let embed_title = "Servers:";
    let mut server_aliases = String::new();
    let mut server_addresses = String::new();
    for server in server_list {
        match server.state {
            GameServerState::Lobby(_) => {
                server_aliases.push_str(&format!("{}\n", server.alias));
                server_addresses.push_str(&"-\n");
            }
            GameServerState::StartedState(started_state, _) => {
                server_aliases.push_str(&format!("{}\n", server.alias));
                server_addresses.push_str(&format!("{}\n", started_state.address));
            }
        }
    }
    message.channel_id.send_message(|m| m
        .embed(|e| e
            .title(embed_title)
            .field( |f| f
                .name("Alias")
                .value(server_aliases)
            )
            .field ( |f| f
                .name("Address")
                .value(server_addresses)
            )
        )
    )?;
    Ok(())
}
