mod add_server;
mod list_servers;
mod remove_server;
mod register_player;
mod details;

use db;

use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::Message;
use typemap::Key;
use std::collections::HashMap;
use std::io;

use serenity::model::UserId;
use server::get_game_data;

use model::game_server::GameServer;

pub fn servers(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    let command = args.single::<String>()?;
    match command.as_ref() {
        "add" => add_server::add_server(context, message, args),
        "list" => list_servers::list_servers(context, message),
        "remove" => remove_server::remove_server(context, message, args),
        "register" => register_player::register_player(context, message, args),
        "unregister" => register_player::unregister_player(context, message, args),
        "show_registered" => register_player::show_registered(context, message, args),
        "pm_players" => register_player::pm_players(context, message, args),
        "details" => details::details(context, message, args),
        _ => Ok(()),
    }
}

pub fn check_server_for_new_turn(server: &mut GameServer) -> io::Result<bool> {
    let server_address = server.address.clone();
    let game_data = get_game_data(&server_address)?;
    if server.last_seen_turn == game_data.turn {
        Ok(false)
    } else {
        server.last_seen_turn = game_data.turn;
        Ok(true)
    }
}
