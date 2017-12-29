extern crate byteorder;

extern crate hex_slice;

extern crate flate2;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[macro_use]
extern crate serenity;
use serenity::prelude::*;
use serenity::model::*;
use serenity::framework::standard::StandardFramework;

extern crate typemap;
use typemap::Key;

mod nations;
mod commands;
mod server;

struct ServerList;
impl Key for ServerList {
    type Value = HashMap<String, String>;
}

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
        .on("spell", commands::spell::spell)
        .on("item", commands::item::item)
        .on("add_server", commands::add_server::add_server)
        .on("list_servers", commands::list_servers::list_servers)
        .on("remove_server", commands::remove_server::remove_server)
        .on("help", commands::help::help)
    );

    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
    println!("returning");
}
