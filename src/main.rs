
#[macro_use]
extern crate lazy_static;

extern crate byteorder;

extern crate hex_slice;

extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

extern crate flate2;
extern crate url;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[macro_use]
extern crate serenity;
use serenity::prelude::*;
use serenity::model::*;
use serenity::framework::standard::StandardFramework;
use typemap::ShareMap;
use commands::servers::check_server_for_new_turn;
use std::{thread, time};

use rusqlite::Connection;

extern crate typemap;

#[macro_use]
extern crate serde_derive;
extern crate bincode;

mod model;
mod commands;
mod server;
mod db;

struct Handler;
impl EventHandler for Handler {
    fn on_ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() {
    let token = {
        let mut token_file = File::open("token").unwrap();
        let mut temp_token = String::new();
        token_file.read_to_string(&mut temp_token).unwrap();
        temp_token
    };

    use r2d2_sqlite::SqliteConnectionManager;
    let manager = SqliteConnectionManager::file("C:\\Users\\David\\Documents\\code\\dom5status\\dom5bot.db");
    let pool = r2d2::Pool::new(manager).unwrap();

    let db_conn = db::DbConnection(pool);
    db_conn.initialise().unwrap();

    let mut discord_client = Client::new(&token, Handler);

    {
        let mut data = discord_client.data.lock();
        data.insert::<db::DbConnectionKey>(db_conn);
    }

    discord_client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .on("ping", commands::ping::ping)
        .on("game_name", commands::game_name::game_name)
        .on("search", commands::inspector::search)
        .on("servers", commands::servers::servers)
        .on("help", commands::help::help)
    );

    let foo = discord_client.data.clone();
    thread::spawn(move || {
        check_for_new_turns_every_1_min(foo.as_ref());
    });
    // start listening for events by starting a single shard
    if let Err(why) = discord_client.start() {
        println!("Client error: {:?}", why);
    }
    println!("returning");
}

fn check_for_new_turns_every_1_min(mutex: &Mutex<ShareMap>) {
    loop {
        thread::sleep(time::Duration::from_secs(60));
        println!("checking for new turns!");
        message_players_if_new_turn(&mutex);
    }
}

fn message_players_if_new_turn(mutex: & Mutex<ShareMap>) {
    let mut data = mutex.lock();
    let db_conn = data.get_mut::<db::DbConnectionKey>().unwrap();
    let servers = db_conn.retrieve_all_servers().unwrap();
    for (_, mut server) in servers {

    
    // for (alias, mut server) in server_list {
        println!("checking {} for new turn", server.alias);
        let new_turn = check_server_for_new_turn(&mut server).unwrap(); 

       if new_turn {
            println!("new turn in game {}", server.alias);
            // for (user_id, player) in &server.players {
    //             if player.allowed_pms {
    //                 let text = format!("your nation {} has a new turn in {}", player.nation_name, alias);
    //                 println!("{}", text);
    //                 let private_channel = user_id.create_dm_channel().unwrap();
    //                 private_channel.say(&text).unwrap();
    //             }

    //         }
       }
    }
}
