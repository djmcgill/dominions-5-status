mod add_server;
mod list_servers;
mod remove_server;
mod register_player;

use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::Message;
use typemap::Key;
use std::collections::HashMap;

use serenity::model::UserId;

#[derive(Debug)]
pub struct Server {
    pub address: String,
    pub players: HashMap<UserId, Player>
}

#[derive(Debug)]
pub struct Player {
    pub nation_id: usize,
    pub allowed_pms: bool, 
}

impl Server {
    pub fn new(address: String) -> Self {
        Server {
            address: address,
            players: HashMap::default(),
        }
    }
}

pub struct ServerList;
impl Key for ServerList {
    type Value = HashMap<String, Server>;
}


pub fn servers(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    let command = args.single::<String>()?;
    match command.as_ref() {
        "add" => add_server::add_server(context, message, args),
        "list" => list_servers::list_servers(context, message),
        "remove" => remove_server::remove_server(context, message, args),
        "register" => register_player::register_player(context, message, args),
        _ => Ok(()),
    }
}
