use typemap::ShareMap;

use db::{DbConnection, DbConnectionKey};
use model::{GameServer, GameServerState, Player};
use model::enums::{NationStatus, SubmissionStatus, Nations};
use std::{thread, time};
use serenity::prelude::Mutex;
use failure::{err_msg, Error};
use server::ServerConnection;
use std::error::Error as TraitError;

pub fn check_for_new_turns_every_1_min<C: ServerConnection>(mutex: &Mutex<ShareMap>) {
    loop {
        thread::sleep(time::Duration::from_secs(60));
        info!("checking for new turns!");
        message_players_if_new_turn::<C>(&mutex).unwrap_or_else(|e| {
            error!("Checking for new turns failed with: {}", e);
        });
    }
}

pub(crate) fn message_players_if_new_turn<C: ServerConnection>(
    mutex: &Mutex<ShareMap>,
) -> Result<(), Error> {
        let data = mutex.try_lock().ok_or_else(|| err_msg("Could not obtain data mutex"))?;
    let db_conn = data.get::<DbConnectionKey>().ok_or_else(|| err_msg("no db connection"))?;
    // TODO: transactions
    let servers = db_conn.retrieve_all_servers()?;
    for server in servers {
        let server_name = server.alias.clone();
        if let Err(err) = check_server_for_new_turn::<C>(&server, db_conn) {
            println!("error checking {} for turn: {:?}", server_name, err);
        };
    }
    Ok(())
}

struct NewTurnNation {
    player: Player,
    nation_id: usize,
}

use server::cache_get;
use model::GameData;

struct NewTurnResult {
    nations_to_notify: Vec<NewTurnNation>,
    new_turn_number: i32,
    ai_this_turn: Vec<usize>,
    defeated_this_turn: Vec<usize>,
    possible_stalls: Vec<usize>,
}

fn check_server_for_new_turn_helper<C: ServerConnection>(
    server: &GameServer,
    db_conn: &DbConnection,
) -> Result<Option<NewTurnResult>, Error> {
    if let GameServerState::StartedState(ref started_state, _) = server.state {
        info!("checking {} for new turn", server.alias);
        let option_old_data: Option<GameData> = cache_get(&server.alias);
        let new_data = C::get_game_data(&server.alias)?;
        let new_turn = started_state.last_seen_turn != new_data.turn;
        if new_turn {
            let new_turn_no = new_data.turn;
            // TODO: check ret value
            let _ = db_conn.update_game_with_possibly_new_turn(&server.alias, new_turn_no)?;
            let players_nations = db_conn.players_with_nations_for_game_alias(&server.alias)?;
            if let Some(old_data) = option_old_data {
                Ok(Some(
                    new_turn_from_old(old_data, &players_nations, new_data)
                ))
            } else {
                Ok(Some(
                    new_turn_from(&players_nations, new_data)
                ))
            }
        } else {
            Ok(None)

        }
    } else {
        Ok(None)
    }
}

fn new_turn_from_old(old: GameData, players_nations: &Vec<(Player, usize)>,  new: GameData) -> NewTurnResult {
    let old_ai_nation_ids = old.nations.iter().filter(|&n| n.status == NationStatus::AI).map(|ref n| n.id).collect::<Vec<usize>>();
    let mut new_ai_nation_ids = new.nations.iter().filter(|&n| n.status == NationStatus::AI).map(|ref n| n.id).collect::<Vec<usize>>();
    new_ai_nation_ids.retain(|ref n| old_ai_nation_ids.contains(n));

    let not_submitted_nation_ids = if old.turn + 1 == new.turn && old.turn_timer <= 60 * 1000 {
        old.nations.iter()
            .filter(|&n|
                n.submitted == SubmissionStatus::NotSubmitted
                    || n.submitted == SubmissionStatus::PartiallySubmitted)
            .map(|ref n| n.id).collect::<Vec<usize>>()
    } else {
        Vec::new()
    };
    let mut new_turn_nations = new_turn_from(players_nations, new);
    new_turn_nations.ai_this_turn = new_ai_nation_ids;
    new_turn_nations.possible_stalls = not_submitted_nation_ids;
    new_turn_nations
}

fn new_turn_from(players_nations: &Vec<(Player, usize)>,  game_data: GameData) -> NewTurnResult {
    let mut ret = Vec::new();
    for &(ref player, nation_id) in players_nations {
        if player.turn_notifications {
            // TODO: quadratic is bad. At least sort it..
            if let Some(nation) = game_data
                .nations
                .iter()
                .find(|&nation| nation.id == nation_id)
                {
                    if (nation.status == NationStatus::Human
                        && nation.submitted == SubmissionStatus::NotSubmitted)
                    || nation.status == NationStatus::DefeatedThisTurn
                        {
                            ret.push(NewTurnNation{
                                player: player.clone(),
                                nation_id: nation_id,
                            });
                        }
                }
        }
    }
    NewTurnResult {
        nations_to_notify: ret,
        new_turn_number: game_data.turn,
        ai_this_turn: Vec::new(),
        defeated_this_turn: Vec::new(), // FIXME
        possible_stalls: Vec::new(),
    }
}

fn check_server_for_new_turn<C: ServerConnection>(
    server: &GameServer,
    db_conn: &DbConnection,
) -> Result<(), Error> {
    if let Some(new_turn_result) = check_server_for_new_turn_helper::<C>(server, db_conn)? {
        for new_turn_nation in new_turn_result.nations_to_notify {
            let nation_id = new_turn_nation.nation_id;
            let player = new_turn_nation.player;
            let &(name, era) = Nations::get_nation_desc(nation_id);
            let mut text = format!(
                "your nation {} {} has a new turn ({}) in {}",
                era,
                name,
                new_turn_result.new_turn_number,
                server.alias
            );
            if !new_turn_result.defeated_this_turn.is_empty() {
                let defeated_this_turn_text = nation_ids_to_comma_name_list(&new_turn_result.defeated_this_turn);
                text.push_str(&format!(
                    "\nDefeated nations this turn: {}",
                    defeated_this_turn_text
                ));
            }
            if !new_turn_result.ai_this_turn.is_empty() {
                let ai_this_turn_text = nation_ids_to_comma_name_list(&new_turn_result.ai_this_turn);
                text.push_str(&format!(
                    "\nAI nations this turn: {}",
                    ai_this_turn_text
                ));
            }
            if !new_turn_result.possible_stalls.is_empty() {
                let possible_stall_text = nation_ids_to_comma_name_list(&new_turn_result.possible_stalls);
                text.push_str(&format!(
                    "\nPossible stalls this turn: {}",
                    possible_stall_text
                ));
            }
            let private_channel = player.discord_user_id.create_dm_channel().map_err(|e| err_msg(e.description().to_owned()))?;
            private_channel.say(&text).map_err(|e| err_msg(e.description().to_owned()))?;
        }
    }
    Ok(())
}

fn nation_ids_to_comma_name_list(ids: &[usize]) -> String {
    if ids.is_empty() {
        "<none>".to_owned()
    } else {
        let mut text = {
            let &(name, era) = Nations::get_nation_desc(ids[0]);
            format!("{} {}", era, name)
        };

        for &nation_id in &ids[1..] {
            let &(name, era) = Nations::get_nation_desc(nation_id);
            text.push_str(&format!(", {} {}", era, name));
        }
        text
    }
}
