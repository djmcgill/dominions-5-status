use serenity::framework::standard::CommandError;
use serenity::prelude::Context;
use serenity::model::channel::Message;
use serenity::builder::CreateEmbed;

use db::*;

use model::GameServerState;

pub fn lobbies(context: &mut Context, message: &Message) -> Result<(), CommandError> {
    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>()
        .ok_or_else(|| CommandError("No db connection".to_string()))?;
    let embed = lobbies_helper(db_conn)?;
    message.channel_id.send_message(|m| m.embed(|_| embed))?;
    Ok(())
}

fn lobbies_helper(db_conn: &DbConnection) -> Result<CreateEmbed, CommandError> {
    let game_servers_and_player_count = db_conn.select_lobbies()?;

    let mut aliases = String::new();
    let mut player_counts = String::new();

    for (game_server, registered_count) in game_servers_and_player_count {
        aliases.push_str(&format!("{}\n", game_server.alias));
        if let GameServerState::Lobby(state) = game_server.state {
            player_counts.push_str(&format!(
                "{}/{}\n",
                registered_count,
                state.player_count
            ));
        } else {
            player_counts.push_str(&"ERROR");
        }
    }

    let embed = CreateEmbed::default()
        .title("Lobbies")
        .field("Alias", aliases, true)
        .field("Players", player_counts, true);

    Ok(embed)
}