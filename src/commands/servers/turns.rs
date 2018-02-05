use serenity::framework::standard::CommandError;
use serenity::prelude::Context;
use serenity::model::{Message, UserId};

use server::ServerConnection;
use db::*;
use model::{GameServerState, Nation};
use model::enums::*;

fn turns_helper<C: ServerConnection>(user_id: UserId, db_conn: &DbConnection) -> Result<String, CommandError> {
    let servers_and_nations_for_player = db_conn.servers_for_player(user_id)?;

    let mut text = "Your turns:\n".to_string();
    for (server, nation_id) in servers_and_nations_for_player {
        // TODO: iflet macro crate
        if let GameServerState::StartedState(ref started_state, _) = server.state {
            if let Ok(game_data) = C::get_game_data(&started_state.address) {
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
    Ok(text)
}

pub fn turns<C: ServerConnection>(context: &mut Context, message: &Message) -> Result<(), CommandError> {
    let data = context.data.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or_else(|| CommandError("No db connection".to_string()))?;
    let text = turns_helper::<C>(message.author.id, db_conn)?;
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
