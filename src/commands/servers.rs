pub mod add_server;
pub mod describe;
pub mod details;
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

use serenity::builder::CreateEmbed;
use serenity::framework::standard::CommandError;
use serenity::model::id::{ChannelId, UserId};
use serenity::{
    framework::standard::{macros::*, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};
use std::future::Future;
use std::pin::Pin;

// FIXME: buckets
#[group]
#[commands(
    server_add,
    server_list,
    server_delete,
    server_details,
    server_register,
    server_register_id,
    server_register_custom,
    server_unregister,
    server_turns,
    server_lobby,
    server_notifications,
    server_start,
    server_lobbies,
    server_describe,
    server_unstart
)]
struct Server;

#[command]
#[aliases("add")]
async fn server_add(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    add_server::add_server(ctx, msg, args).await
}

#[command]
#[aliases("list")]
async fn server_list(ctx: &Context, msg: &Message) -> CommandResult {
    list_servers::list_servers(ctx, msg).await
}

#[command]
#[aliases("delete")]
async fn server_delete(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    remove_server::remove_server(ctx, msg, args).await
}

#[command]
#[aliases("details", "deets")]
async fn server_details(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    details::details2(ctx, msg, args).await
}

#[command]
#[aliases("register")]
async fn server_register(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    register_player::register_player(ctx, msg, args).await
}

#[command]
#[aliases("register-id")]
async fn server_register_id(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    register_player::register_player_id(ctx, msg, args).await
}

#[command]
#[aliases("register-custom")]
async fn server_register_custom(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    register_player::register_player_custom(ctx, msg, args).await
}

#[command]
#[aliases("unregister")]
async fn server_unregister(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    unregister_player::unregister_player(ctx, msg, args).await
}

#[command]
#[aliases("turns")]
async fn server_turns(ctx: &Context, msg: &Message) -> CommandResult {
    turns::turns2(ctx, msg).await
}

#[command]
#[aliases("lobby")]
async fn server_lobby(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    lobby::lobby(ctx, msg, args).await
}

#[command]
#[aliases("notifications")]
async fn server_notifications(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    notifications::notifications(ctx, msg, args).await
}

#[command]
#[aliases("start")]
async fn server_start(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    start::start(ctx, msg, args).await
}

#[command]
#[aliases("lobbies")]
async fn server_lobbies(ctx: &Context, msg: &Message) -> CommandResult {
    lobbies::lobbies(ctx, msg).await
}

#[command]
#[aliases("describe")]
async fn server_describe(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    describe::describe(ctx, msg, args).await
}

#[command]
#[aliases("unstart")]
async fn server_unstart(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    unstart::unstart(ctx, msg, args).await
}

async fn alias_from_arg_or_channel_name(
    args: &mut Args,
    message: &Message,
    ctx: &Context,
) -> Result<String, String> {
    let result_alias = if !args.is_empty() {
        args.single_quoted::<String>().ok()
    } else {
        message.channel_id.name(ctx.cache.clone()).await
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

// TODO: the idea is that all commands (that don't use embeds), both slash and bang (normal) go
//       through this and then the command helpers get changed to fit this format
// TODO: also, debox
async fn bang_command_reply_wrap<F>(
    context: &Context,
    message: &Message,
    args: Args,
    f: F,
) -> Result<(), CommandError>
where
    F: Fn(
        &Context,
        ChannelId,
        UserId,
        Args,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, CommandError>>>>,
{
    let channel_id = message.channel_id;
    let user_id = message.author.id;
    let option_string = f(context, channel_id, user_id, args).await?;

    if let Some(reply) = option_string {
        message
            .reply((&context.cache, context.http.as_ref()), reply)
            .await?;
    }
    Ok(())
}

async fn bang_command_embed_wrap<F>(
    context: &Context,
    message: &Message,
    args: Args,
    f: F,
) -> Result<(), CommandError>
where
    F: Fn(
        &Context,
        ChannelId,
        UserId,
        Args,
    ) -> Pin<Box<dyn Future<Output = Result<CreateEmbed, CommandError>>>>,
{
    let channel_id = message.channel_id;
    let user_id = message.author.id;
    let embed = f(context, channel_id, user_id, args).await?;
    message
        .channel_id
        .send_message(&context.http, |m| {
            m.embed(|e| {
                *e = embed;
                e
            })
        })
        .await?;
    Ok(())
}
