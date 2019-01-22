use serenity::framework::standard::CommandError;
use serenity::prelude::Context;
use serenity::model::channel::Message;
use serenity::builder::CreateEmbed;

use crate::db::*;

use crate::model::{GameServer, GameServerState};

pub fn lobbies(context: &mut Context, message: &Message) -> Result<(), CommandError> {
    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>()
        .ok_or_else(|| CommandError("No db connection".to_string()))?;

    let lobbies_and_player_count = db_conn.select_lobbies()?;
    if lobbies_and_player_count.is_empty() {
        message.reply(&"No available lobbies")?;
    } else {
        let embed = lobbies_helper(lobbies_and_player_count)?;
        message.channel_id.send_message(|m| m.embed(|_| embed))?;
    }
    Ok(())
}

fn lobbies_helper(lobbies_and_player_count: Vec<(GameServer, i32)>) -> Result<CreateEmbed, CommandError> {
    let mut aliases = String::new();
    let mut player_counts = String::new();

    for (lobby, registered_count) in lobbies_and_player_count {
        aliases.push_str(&format!("{}\n", lobby.alias));
        if let GameServerState::Lobby(state) = lobby.state {
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