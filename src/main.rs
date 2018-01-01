
#[macro_use]
extern crate lazy_static;

extern crate byteorder;

extern crate hex_slice;

extern crate flate2;
extern crate url;
extern crate futures;
extern crate tokio_core;
extern crate tiberius;

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

extern crate typemap;

#[macro_use]
extern crate serde_derive;
extern crate bincode;

mod nations;
mod commands;
mod server;

use commands::servers::ServerList;

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
    
    let mut client = Client::new(&token, Handler);

    {
        let mut data = client.data.lock();
        data.insert::<ServerList>(HashMap::default());
    }

    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .on("ping", commands::ping::ping)
        .on("game_name", commands::game_name::game_name)
        .on("nation_status", commands::nation_status::nation_status)
        .on("search", commands::inspector::search)
        .on("servers", commands::servers::servers)
        .on("help", commands::help::help)
    );

    let foo = client.data.clone();
    thread::spawn(move || {
        check_for_new_turns_every_1_min(foo.as_ref());
    });
    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
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
    let server_list = data.get_mut::<ServerList>().unwrap();
    for (alias, mut server) in server_list {
        println!("checking {} for new turn", alias);
        let new_turn = check_server_for_new_turn(&mut server).unwrap(); 

       if new_turn {
            println!("new turn in game {}", alias);
            for (user_id, player) in &server.players {
                if player.allowed_pms {
                    let text = format!("your nation {} has a new turn in {}", player.nation_name, alias);
                    println!("{}", text);
                    let private_channel = user_id.create_dm_channel().unwrap();
                    private_channel.say(&text).unwrap();
                }

            }
       }
    }
}
