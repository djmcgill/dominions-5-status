mod add_server;
mod list_servers;
mod remove_server;

use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::Message;
use typemap::Key;
use std::collections::HashMap;

pub struct ServerList;
impl Key for ServerList {
    type Value = HashMap<String, String>;
}


pub fn servers(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    let command = args.single::<String>()?;
    match command.as_ref() {
        "add" => add_server::add_server(context, message, args),
        "list" => list_servers::list_servers(context, message),
        "remove" => remove_server::remove_server(context, message, args),
        _ => Ok(()),
    }
}
