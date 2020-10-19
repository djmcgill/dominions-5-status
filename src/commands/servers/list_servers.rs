use serenity::{CacheAndHttp, builder::CreateEmbed};
use serenity::framework::standard::CommandError;
use serenity::model::channel::Message;
use serenity::prelude::Context;

use crate::db::*;
use crate::model::GameServerState;

fn list_servers_helper(db_conn: &DbConnection) -> Result<CreateEmbed, CommandError> {
    let server_list = db_conn.retrieve_all_servers().map_err(CommandError::from)?;

    if server_list.is_empty() {
        Ok(CreateEmbed::default().title("NO SERVERS").clone()) // TODO: can we avoid a clone here
    } else {
        let embed_title = "Servers:";
        let mut server_aliases = String::new();
        let mut server_addresses = String::new();

        for server in server_list {
            match server.state {
                GameServerState::Lobby(_) => {
                    server_aliases.push_str(&format!("{}\n", server.alias));
                    server_addresses.push_str(&"-\n");
                }
                GameServerState::StartedState(ref started_state, _) => {
                    server_aliases.push_str(&format!("{}\n", server.alias));
                    server_addresses.push_str(&format!("{}\n", started_state.address));
                }
            }
        }

        let embed = CreateEmbed::default()
            .title(embed_title)
            .field("Alias", server_aliases, true)
            .field("Address", server_addresses, true);

        Ok(embed)
    }
}

pub fn list_servers(context: &mut Context, message: &Message) -> Result<(), CommandError> {
    let data = context.data.read();
    let db_conn = data
        .get::<DbConnectionKey>()
        .ok_or_else(|| CommandError("No db connection".to_string()))?;
    let embed = list_servers_helper(db_conn)?;
    message.channel_id.send_message(todo!("satisfy this argument"), |m| m.embed(|_| embed))?;
    Ok(())
}
