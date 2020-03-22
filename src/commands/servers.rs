// FIXME
pub mod add_server;
pub mod describe;
pub mod details;
// pub mod details;
pub mod list_servers;
pub mod lobbies;
pub mod lobby;
pub mod notifications;
pub mod register_player;
pub mod remove_server;
pub mod start;
pub mod turn_check;
pub mod turns;
pub mod unregister_player;
pub mod unstart;

use crate::server::{RealServerConnection};
use serenity::framework::standard::{Args, macros::*, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;

// FIXME: buckets
#[group]
#[commands(
    server_add, server_list, server_delete,
    server_details, server_register, server_register_id, server_unregister,
    server_turns, server_lobby, server_notifications, server_start,
    server_lobbies, server_describe, server_unstart
)]
struct Server;

#[command]
#[aliases("add")]
fn server_add(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    add_server::add_server::<RealServerConnection>(ctx, msg, args)
}

#[command]
#[aliases("list")]
fn server_list(ctx: &mut Context, msg: &Message) -> CommandResult {
    list_servers::list_servers(ctx, msg)
}

#[command]
#[aliases("delete")]
fn server_delete(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    remove_server::remove_server(ctx, msg, args)
}

#[command]
#[aliases("details", "deets")]
fn server_details(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    details::details2::<RealServerConnection>(ctx, msg, args)
}

#[command]
#[aliases("register")]
fn server_register(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    register_player::register_player::<RealServerConnection>(ctx, msg, args)
}

#[command]
#[aliases("register-id")]
fn server_register_id(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    register_player::register_player_id::<RealServerConnection>(ctx, msg, args)
}

#[command]
#[aliases("unregister")]
fn server_unregister(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    unregister_player::unregister_player(ctx, msg, args)
}

#[command]
#[aliases("turns")]
fn server_turns(ctx: &mut Context, msg: &Message) -> CommandResult {
    turns::turns2::<RealServerConnection>(ctx, msg)
}

#[command]
#[aliases("lobby")]
fn server_lobby(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    lobby::lobby(ctx, msg, args)
}

#[command]
#[aliases("notifications")]
fn server_notifications(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    notifications::notifications(ctx, msg, args)
}

#[command]
#[aliases("start")]
fn server_start(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    start::start::<RealServerConnection>(ctx, msg, args)
}

#[command]
#[aliases("lobbies")]
fn server_lobbies(ctx: &mut Context, msg: &Message) -> CommandResult {
    lobbies::lobbies(ctx, msg)
}

#[command]
#[aliases("describe")]
fn server_describe(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    describe::describe(ctx, msg, args)
}

#[command]
#[aliases("unstart")]
fn server_unstart(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    unstart::unstart(ctx, msg, args)
}

fn alias_from_arg_or_channel_name(args: &mut Args, message: &Message, ctx: &Context) -> Result<String, String> {
    let result_alias = if !args.is_empty() {
        args.single_quoted::<String>().ok()
    } else {
        message.channel_id.name(ctx.cache.clone())
    };
    result_alias
        .clone()
        .map(|s| s.to_lowercase())
        .and_then(|s| if !s.is_empty() { Some(s) } else { None })
        .ok_or_else(|| {
            format!(
                "Could not game alias from command argument or channel name \"{}\"",
                result_alias.unwrap_or_default()
            )
        })
}
