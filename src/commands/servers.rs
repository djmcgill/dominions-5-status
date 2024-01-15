pub mod add_server;
pub mod alias;
pub mod describe;
pub mod details;
pub mod kick;
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

use chrono::{DateTime, Utc};
use serenity::all::CreateMessage;
use serenity::builder::CreateEmbed;
use serenity::framework::standard::CommandError;
use serenity::model::id::{ChannelId, UserId};
use serenity::{
    framework::standard::{macros::*, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};
use std::future::Future;

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
    server_unstart,
    server_set_alias,
    server_kick
)]
struct Server;

#[command]
#[aliases("add")]
async fn server_add(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    bang_command_wrap(ctx, msg, args, add_server::add_server).await
}

#[command]
#[aliases("list")]
async fn server_list(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    bang_command_wrap(ctx, msg, args, list_servers::list_servers).await
}

#[command]
#[aliases("delete")]
async fn server_delete(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    bang_command_wrap(ctx, msg, args, remove_server::remove_server).await
}

#[command]
#[aliases("details", "deets")]
async fn server_details(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    bang_command_wrap(ctx, msg, args, details::details).await
}

#[command]
#[aliases("register")]
async fn server_register(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    bang_command_wrap(ctx, msg, args, register_player::register_player).await
}

#[command]
#[aliases("register-id")]
async fn server_register_id(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    bang_command_wrap(ctx, msg, args, register_player::register_player_id).await
}

#[command]
#[aliases("register-custom")]
async fn server_register_custom(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    bang_command_wrap(ctx, msg, args, register_player::register_player_custom).await
}

#[command]
#[aliases("unregister")]
async fn server_unregister(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    bang_command_wrap(ctx, msg, args, unregister_player::unregister_player).await
}

#[command]
#[aliases("turns")]
async fn server_turns(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    bang_command_wrap(ctx, msg, args, turns::turns).await
}

#[command]
#[aliases("lobby")]
async fn server_lobby(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    bang_command_wrap(ctx, msg, args, lobby::lobby).await
}

#[command]
#[aliases("notifications")]
async fn server_notifications(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    bang_command_wrap(ctx, msg, args, notifications::notifications).await
}

#[command]
#[aliases("start")]
async fn server_start(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    bang_command_wrap(ctx, msg, args, start::start).await
}

#[command]
#[aliases("lobbies")]
async fn server_lobbies(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    bang_command_wrap(ctx, msg, args, lobbies::lobbies).await
}

#[command]
#[aliases("describe")]
async fn server_describe(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    bang_command_wrap(ctx, msg, args, describe::describe).await
}

#[command]
#[aliases("unstart")]
async fn server_unstart(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    bang_command_wrap(ctx, msg, args, unstart::unstart).await
}

#[command]
#[aliases("alias")]
async fn server_set_alias(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    bang_command_wrap(ctx, msg, args, alias::server_set_alias).await
}

#[command]
#[aliases("kick", "banish")]
async fn server_kick(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    bang_command_wrap(ctx, msg, args, kick::kick_player).await
}

pub enum CommandResponse {
    Embed(Box<CreateEmbed>),
    Reply(String),
}

async fn alias_from_arg_or_channel_name(
    ctx: &Context,
    channel_id: ChannelId,
    args: &mut Args,
) -> Result<String, CommandError> {
    let result_alias = if !args.is_empty() {
        args.single_quoted::<String>().ok()
    } else {
        channel_id
            .to_channel((&ctx.cache, ctx.http.as_ref()))
            .await?
            .id()
            .name((&ctx.cache, ctx.http.as_ref()))
            .await
            .ok()
    };
    result_alias
        .clone()
        .map(|s| s.to_lowercase())
        .and_then(|s| if !s.is_empty() { Some(s) } else { None })
        .ok_or_else(|| {
            CommandError::from(format!(
                "Could not get game alias from command argument or channel name \"{}\"",
                result_alias.unwrap_or_default()
            ))
        })
}

async fn bang_command_wrap<'a, F, Fut>(
    context: &'a Context,
    message: &'a Message,
    args: Args,
    f: F,
) -> Result<(), CommandError>
where
    F: FnOnce(&'a Context, ChannelId, UserId, Args) -> Fut,
    Fut: Future<Output = Result<CommandResponse, CommandError>> + 'a,
{
    let channel_id = message.channel_id;
    let user_id = message.author.id;
    let command_response = f(context, channel_id, user_id, args).await?;

    match command_response {
        CommandResponse::Reply(reply) => {
            message
                .reply((&context.cache, context.http.as_ref()), reply)
                .await?;
        }
        CommandResponse::Embed(embed) => {
            message
                .channel_id
                .send_message(&context.http, CreateMessage::default().embed(*embed))
                .await?;
        }
    }
    Ok(())
}

pub fn discord_date_format(deadline: DateTime<Utc>) -> String {
    let duration_from_now_to_deadline = deadline.signed_duration_since(Utc::now());

    let hours_remaining = duration_from_now_to_deadline.num_hours();
    let mins_remaining = duration_from_now_to_deadline.num_minutes() % 60;
    format!(
        "{}h {}m/<t:{}>",
        hours_remaining,
        mins_remaining,
        deadline.timestamp_millis() / 1000
    )
}
