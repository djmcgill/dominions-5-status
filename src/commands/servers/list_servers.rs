use crate::commands::servers::CommandResponse;
use crate::db::*;
use crate::model::game_server::GameServerState;
use serenity::builder::CreateEmbed;
use serenity::framework::standard::{Args, CommandError};
use serenity::model::id::{ChannelId, UserId};
use serenity::prelude::Context;

fn list_servers_helper(db_conn: DbConnection) -> Result<CreateEmbed, CommandError> {
    let server_list = db_conn.retrieve_all_servers().map_err(CommandError::from)?;

    if server_list.is_empty() {
        Ok(CreateEmbed::default().title("NO SERVERS"))
    } else {
        let embed_title = "Servers:";
        let mut server_aliases = String::new();
        let mut server_addresses = String::new();

        for server in server_list {
            match server.state {
                GameServerState::Lobby(_) => {
                    server_aliases.push_str(&format!("{}\n", server.alias));
                    server_addresses.push_str("-\n");
                }
                GameServerState::StartedState(ref started_state, _) => {
                    server_aliases.push_str(&format!("{}\n", server.alias));
                    server_addresses.push_str(&format!("{}\n", started_state.address));
                }
            }
        }

        Ok(CreateEmbed::default()
            .title(embed_title)
            .field("Alias", server_aliases, true)
            .field("Address", server_addresses, true))
    }
}

pub async fn list_servers(
    context: &Context,
    _channel_id: ChannelId,
    _user_id: UserId,
    _args: Args,
) -> Result<CommandResponse, CommandError> {
    let db_conn = {
        let data = context.data.read().await;
        data.get::<DbConnectionKey>()
            .ok_or_else(|| CommandError::from("No db connection".to_string()))?
            .clone()
    };
    let embed = list_servers_helper(db_conn)?;
    Ok(CommandResponse::Embed(Box::new(embed)))
}
