use typemap::ShareMap;

use db::{DbConnection, DbConnectionKey};
use model::{GameServer, GameServerState, Player};
use model::enums::{NationStatus, SubmissionStatus};
use std::{thread, time};
use serenity::prelude::Mutex;
use std::error::Error;
use server::ServerConnection;

pub fn check_for_new_turns_every_1_min<C: ServerConnection>(mutex: &Mutex<ShareMap>) {
    loop {
        thread::sleep(time::Duration::from_secs(60));
        info!("checking for new turns!");
        message_players_if_new_turn::<C>(&mutex).unwrap_or_else(|e| {
            error!("Checking for new turns failed with: {}", e);
        });
    }
}

fn message_players_if_new_turn<C: ServerConnection>(mutex: &Mutex<ShareMap>) -> Result<(), Box<Error>> {
    let data = mutex.lock();
    let db_conn = data.get::<DbConnectionKey>().ok_or("no db connection")?;
    // TODO: transactions
    let servers = db_conn.retrieve_all_servers()?;
    for server in servers {
        let server_name = server.alias.clone();
        if let Err(err) = check_server_for_new_turn::<C>(server, &db_conn) {
            println!("error checking {} for turn: {:?}", server_name, err);
        };
    }
    Ok(())
}

fn check_server_for_new_turn_helper<C: ServerConnection>(server: GameServer, db_conn: &DbConnection)
    -> Result<Vec<(Player, String)>, Box<Error>> {

    let mut ret = Vec::new();
    if let GameServerState::StartedState(started_state, _) = server.state {
        info!("checking {} for new turn", server.alias);
        let game_data = C::get_game_data(&started_state.address)?;
        let new_turn = db_conn.update_game_with_possibly_new_turn(
            &server.alias,
            game_data.turn
        )?;

        if new_turn {
            info!("new turn in game {}", server.alias);
            for (player, nation_id) in db_conn.players_with_nations_for_game_alias(&server.alias)? {
                if player.turn_notifications {
                    // TODO: quadratic is bad. At least sort it..
                    if let Some(nation) = game_data.nations.iter().find(|&nation| nation.id == nation_id) {
                        if nation.status == NationStatus::Human && nation.submitted == SubmissionStatus::NotSubmitted {
                            use model::enums::Nations;
                            let &(name, era) = Nations::get_nation_desc(nation_id);
                            let text = format!("your nation {} {} has a new turn ({}) in {}",
                                               era,
                                               name,
                                               game_data.turn,
                                               server.alias);
                            info!("Sending DM: {}", text);
                            ret.push((player, text));
                        }
                    }
                }
            }
        }
    };


    Ok(ret)
}

fn check_server_for_new_turn<C: ServerConnection>(server: GameServer, db_conn: &DbConnection) -> Result<(), Box<Error>> {
    // TODO: group players
    for (player, text) in check_server_for_new_turn_helper::<C>(server, db_conn)? {
        let private_channel = player.discord_user_id.create_dm_channel()?;
        private_channel.say(&text)?;
    }
    Ok(())
}
