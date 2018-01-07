mod add_server;
mod list_servers;
mod remove_server;
mod register_player;
mod details;

use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::Message;

pub fn servers(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    let command = args.single_quoted::<String>()?;
    match command.as_ref() {
        "add" => add_server::add_server(context, message, args),
        "list" => list_servers::list_servers(context, message),
        "remove" => remove_server::remove_server(context, message, args),
        "register" => register_player::register_player(context, message, args),
        "unregister" => register_player::unregister_player(context, message, args),
        "details" => details::details(context, message, args),
        _ => Ok(()),
    }
}
