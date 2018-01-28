use serenity::framework::standard::CommandError;
use serenity::prelude::Context;
use serenity::model::Message;

use server;
use db::DbConnectionKey;
use model::{GameServerState, Nation};
use model::enums::*;

pub fn turns(context: &mut Context, message: &Message) -> Result<(), CommandError> {
    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or_else(|| CommandError("No db connection".to_string()))?;
    let servers_and_nations_for_player = db_conn.servers_for_player(message.author.id)?;

    let mut text = "Your turns:\n".to_string();
    for (server, nation_id) in servers_and_nations_for_player {
        if let GameServerState::StartedState(ref started_state, _) = server.state {
            if let Ok(game_data) = server::get_game_data(&started_state.address) {
                if let Some(nation) = game_data.nations.iter().find(|&n| n.id == nation_id as usize) {
                    if nation.status == NationStatus::Human {
                        let (hours_remaining, mins_remaining) = hours_mins_remaining(game_data.turn_timer);
                        let human_count = human_nations(&game_data.nations);
                        let submitted_count = submitted_nations(&game_data.nations);
                        let turn_str = format!("{} turn {} ({}h {}m): {} (submitted: {}, {}/{})\n",
                            server.alias,
                            game_data.turn,
                            hours_remaining,
                            mins_remaining,
                            nation.name,
                            nation.submitted.show(),
                            submitted_count,
                            human_count,
                        );
                        text.push_str(&turn_str);
                    }
                } else {
                    text.push_str(&format!("{}: ERROR\n", server.alias));    
                }
            } else {
                text.push_str(&format!("{}: ERROR\n", server.alias));
            }
        }
    }
    info!("replying with {}", text);
    let private_channel = message.author.id.create_dm_channel()?;
    private_channel.say(&text)?;
    Ok(())
}

fn human_nations(nations: &Vec<Nation>) -> i32 {
    nations.iter().fold(0, |t, i| 
        if i.status == NationStatus::Human
        {
            t+1
        } else {
            t
        })
}

fn submitted_nations(nations: &Vec<Nation>) -> i32 {
    nations.iter().fold(0, |t, i| 
        if i.submitted == SubmissionStatus::Submitted && i.status == NationStatus::Human
        {
            t+1
        } else {
            t
        })
}

fn hours_mins_remaining(turn_timer: i32) -> (i32, i32) {
    let total_mins_remaining = turn_timer / (1000*60);
    let hours_remaining = total_mins_remaining/60;
    let mins_remaining = total_mins_remaining - hours_remaining*60;
    (hours_remaining, mins_remaining)
}
