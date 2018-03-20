#![allow(dead_code)]
// hyssop snek.earth:30097

use std::{env, fs};
use db::*;
use model::*;
use serenity::prelude::*;
use super::*;
use server::ServerConnection;
use std::path::Path;
use std::thread;

use simplelog::{Config, LogLevelFilter, SimpleLogger};

// Note this is a test trying to replicate a crash. It requires a real db and a
// real server connection/address for now. Not to be run as part of normal unit tests.
//#[test]
fn run_details_and_turns_simultaneously() {
    SimpleLogger::init(LogLevelFilter::Debug, Config::default()).unwrap();

    // Create app
    let server_address = "snek.earth:30097";
    let game_alias = "hyssop";
    let file_loc = "resources/it.db";
    if Path::new(file_loc).is_file() {
        fs::remove_file(file_loc).unwrap();
    }

    let token = read_token().unwrap();

    let path = env::current_dir().unwrap();
    let path = path.join(file_loc);
    let db_conn = DbConnection::new(&path).unwrap();

    let mut discord_client = Client::new(&token, Handler).unwrap();
    {
        let mut data = discord_client.data.lock();
        data.insert::<DbConnectionKey>(db_conn);
    }

    // Set up db state
    {
        let data_clone = discord_client.data.clone();
        let data = data_clone.lock();
        let db_connection = data.get::<DbConnectionKey>().unwrap();
        let game_data = RealServerConnection::get_game_data(server_address).unwrap();

        let server = GameServer {
            alias: game_alias.to_string(),
            state: GameServerState::StartedState(
                StartedState {
                    address: server_address.to_string(),
                    last_seen_turn: game_data.turn,
                },
                None,
            ),
        };

        db_connection.insert_game_server(&server).unwrap();
        debug!("Successfully set up db state");
    }
    {
        let data_clone = discord_client.data.clone();
        debug!("successfully cloned 1");
        // Call details and turn-check at the same time
        thread::spawn(move || {
            debug!("starting details");
            let data = data_clone.lock();
            let db_connection = data.get::<DbConnectionKey>().unwrap();
            let res = commands::servers::details_helper::<RealServerConnection>(&db_connection, game_alias);
            debug!("DETAILS RESULT: {:?}", res);
            res.unwrap();
        });
    }
    {
        let data = discord_client.data.clone();
        debug!("successfully cloned 2");
        thread::spawn(move || {
            debug!("starting turn_check");
            let res = commands::servers::message_players_if_new_turn::<RealServerConnection>(
                data.as_ref()
            );
            debug!("TURN_CHECK RESULT: {:?}", res);
            res.unwrap();
        });
    }

    println!("SUCCESS");
    panic!();

}