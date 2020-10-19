//! Wraps the formerly builder-based commands in new macro attribute ones.

use serenity::{
    client::Context, framework::standard::macros::command, framework::standard::Args,
    framework::standard::CommandResult, model::channel::Message,
};

use crate::server::RealServerConnection;

//////////////////////////
// SERVER COMMANDS
//////////////////////////

#[command]
pub fn add(cx: &mut Context, m: &Message, a: Args) -> CommandResult {
    // TODO: template other kinds of servers?
    // add_server::<C>(cx, m, a))
    super::add_server::<RealServerConnection>(cx, m, a)
}

#[command]
pub fn list(cx: &mut Context, m: &Message) -> CommandResult {
    super::list_servers(cx, m)
}

#[command]
pub fn delete(cx: &mut Context, m: &Message, a: Args) -> CommandResult {
    super::remove_server(cx, m, a)
}

#[command]
pub fn details(cx: &mut Context, m: &Message, a: Args) -> CommandResult {
    // TODO: find a way to template this
    // super::details2::<C>(cx, m, a)
    super::details2::<RealServerConnection>(cx, m, a)
}

// #[command]
// // pub fn details2(cx: &Context, m: &Message, a: Args) -> CommandResult {
// //     super::details2::<C>(cx, m, a)
// // }

// #[command]
// // pub fn deets(cx: &Context, m: &Message), a: Args -> CommandResult {
// //     super::details2::<C>(cx, m, a)
// // }

// #[command]
// // pub fn deets2(cx: &Context, m: &Message, a: Args) -> CommandResult {
// //     super::details2::<C>(cx, m, a)
// // }

#[command]
pub fn register(cx: &mut Context, m: &Message, a: Args) -> CommandResult {
    super::register_player(cx, m, a)
}

#[command]
pub fn register_id(cx: &mut Context, m: &Message, a: Args) -> CommandResult {
    super::register_player_id(cx, m, a)
}

#[command]
pub fn register_custom(cx: &mut Context, m: &Message, a: Args) -> CommandResult {
    super::register_custom(cx, m, a)
}

#[command]
pub fn unregister(cx: &mut Context, m: &Message, a: Args) -> CommandResult {
    super::unregister_player(cx, m, a)
}

#[command]
pub fn turns(cx: &mut Context, m: &Message) -> CommandResult {
    // super::turns2::<C>(cx, m)
    super::turns2::<RealServerConnection>(cx, m)
}

#[command]
pub fn turns2(cx: &mut Context, m: &Message) -> CommandResult {
    // super::turns2::<C>(cx, m)
    super::turns2::<RealServerConnection>(cx, m)
}

#[command]
pub fn lobby(cx: &mut Context, m: &Message, a: Args) -> CommandResult {
    super::lobby(cx, m, a)
}

#[command]
pub fn notifications(cx: &mut Context, m: &Message, a: Args) -> CommandResult {
    super::notifications(cx, m, a)
}

#[command]
pub fn start(cx: &mut Context, m: &Message, a: Args) -> CommandResult {
    // super::start::<C>(cx, m, a)
    super::start::<RealServerConnection>(cx, m, a)
}

#[command]
pub fn lobbies(cx: &mut Context, m: &Message) -> CommandResult {
    super::lobbies(cx, m)
}

#[command]
pub fn describe(cx: &mut Context, m: &Message, a: Args) -> CommandResult {
    super::describe(cx, m, a)
}

#[command]
pub fn desc(cx: &mut Context, m: &Message, a: Args) -> CommandResult {
    super::describe(cx, m, a)
}

#[command]
pub fn unstart(cx: &mut Context, m: &Message, a: Args) -> CommandResult {
    super::unstart(cx, m, a)
}

//////////////////////////
// SEARCH COMMANDS
//////////////////////////
