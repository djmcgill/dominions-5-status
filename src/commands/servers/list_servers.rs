use serenity::framework::standard::CommandError;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use serenity::{builder::CreateEmbed, http::Http, CacheAndHttp};

use crate::db::*;
use crate::model::GameServerState;

fn list_servers_helper<'a>(
    db_conn: &DbConnection,
    embed: &'a mut CreateEmbed,
) -> Result<&'a mut CreateEmbed, CommandError> {
    let server_list = db_conn.retrieve_all_servers().map_err(CommandError::from)?;

    if server_list.is_empty() {
        Ok(embed.title("NO SERVERS"))
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

        embed
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
    // let mut embed = CreateEmbed::default();
    // embed = list_servers_helper(db_conn, &mut embed)?;
    // TODO: again, how do we preserve error handling here
    message.channel_id.send_message(Http::default(), |m| {
        m.embed(|emb| list_servers_helper(db_conn, emb).unwrap())
    })?;
    Ok(())
}
