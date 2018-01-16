use serenity::framework::standard::CommandError;
use serenity::prelude::Context;
use serenity::model::Message;

use server;
use db::DbConnectionKey;

pub fn turns(context: &mut Context, message: &Message) -> Result<(), CommandError> {
    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or_else(|| CommandError("No db connection".to_string()))?;
    let servers_and_nations_for_player = db_conn.servers_for_player(message.author.id)?;

    let mut text = "Your turns:\n".to_string();
    for (server, nation_id) in servers_and_nations_for_player {
        let game_data = server::get_game_data(&server.address)?;
        let ref nation = game_data.nations.iter().find(|&n| n.id == nation_id as usize).unwrap();
        let total_mins_remaining = game_data.turn_timer / (1000*60);
        let hours_remaining = total_mins_remaining/60;
        let mins_remaining = total_mins_remaining - hours_remaining*60;
        text.push_str(&format!("{} turn {} ({}h {}m): {} (submitted: {})\n",
            game_data.game_name,
            game_data.turn,
            hours_remaining,
            mins_remaining,
            nation.name,
            nation.submitted.show(),
        ));
    }
    info!("replying with {}", text);
    let private_channel = message.author.id.create_dm_channel()?;
    private_channel.say(&text)?;
    Ok(())
}
