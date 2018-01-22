use serenity::framework::standard::CommandError;
use serenity::prelude::Context;
use serenity::model::Message;

use server;
use db::DbConnectionKey;
use model::{GameServer, GameServerState};

pub fn turns(context: &mut Context, message: &Message) -> Result<(), CommandError> {
    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or_else(|| CommandError("No db connection".to_string()))?;
    let servers_and_nations_for_player = db_conn.servers_for_player(message.author.id)?;

    let mut text = "Your turns:\n".to_string();
    for (server, nation_id) in servers_and_nations_for_player {
        text.push_str(&turn_for_server(&server, nation_id).unwrap_or(format!("{} error", server.alias)));
        info!("replying with {}", text);
        let private_channel = message.author.id.create_dm_channel()?;
        private_channel.say(&text)?;
    }
    Ok(())
}

pub fn turn_for_server(server: &GameServer, nation_id: i32) -> Result<String, CommandError> {
    if let GameServerState::StartedState(ref started_state) = server.state {
        let game_data = server::get_game_data(&started_state.address)?;
        let ref nation = game_data.nations.iter().find(|&n| n.id == nation_id as usize)
            .ok_or(format!("could not find nation {} in server {}", nation_id, server.alias))?;
        let total_mins_remaining = game_data.turn_timer / (1000*60);
        let hours_remaining = total_mins_remaining/60;
        let mins_remaining = total_mins_remaining - hours_remaining*60;
        Ok(format!("{} turn {} ({}h {}m): {} (submitted: {})\n",
            server.alias,
            game_data.turn,
            hours_remaining,
            mins_remaining,
            nation.name,
            nation.submitted.show(),
        ))
    } else {
        Err(CommandError::from("lobbies not implemented yet"))
    }
}
