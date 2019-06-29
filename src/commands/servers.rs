mod add_server;
use self::add_server::add_server;

mod list_servers;
use self::list_servers::*;

mod remove_server;
use self::remove_server::*;

mod register_player;
use self::register_player::*;

mod unregister_player;
use self::unregister_player::*;

mod details;
pub use self::details::*;

mod details2;
pub use self::details2::*;

mod turns;
use self::turns::*;

mod lobby;
use self::lobby::*;

mod start;
use self::start::*;

mod notifications;
use self::notifications::*;

pub mod turn_check;

mod lobbies;
pub use self::lobbies::*;

mod describe;
pub use self::describe::*;

mod turns2;
pub use self::turns2::turns2;

use crate::server::ServerConnection;
use serenity::framework::standard::{Args, StandardFramework};
use serenity::model::channel::Message;

pub trait WithServersCommands: Sized {
    fn get_standard_framework(self) -> StandardFramework;
    fn with_servers_commands<C: ServerConnection>(self, bucket: &str) -> StandardFramework {
        self.get_standard_framework()
            .command("add", |c| {
                c.bucket(bucket).exec(|cx, m, a| add_server::<C>(cx, m, a))
            })
            .command("list", |c| {
                c.bucket(bucket).exec(|cx, m, _| list_servers(cx, m))
            })
            .command("delete", |c| {
                c.bucket(bucket).exec(|cx, m, a| remove_server(cx, m, a))
            })
            .command("details", |c| {
                c.bucket(bucket).exec(|cx, m, a| details::<C>(cx, m, a))
            })
            .command("details2", |c| {
                c.bucket(bucket).exec(|cx, m, a| details2::<C>(cx, m, a))
            })
            .command("deets", |c| {
                c.bucket(bucket).exec(|cx, m, a| details::<C>(cx, m, a))
            })
            .command("deets2", |c| {
                c.bucket(bucket).exec(|cx, m, a| details2::<C>(cx, m, a))
            })
            .command("register", |c| {
                c.bucket(bucket)
                    .exec(|cx, m, a| register_player::<C>(cx, m, a))
            })
            .command("register-id", |c| {
                c.bucket(bucket)
                    .exec(|cx, m, a| register_player_id::<C>(cx, m, a))
            })
            .command("unregister", |c| {
                c.bucket(bucket)
                    .exec(|cx, m, a| unregister_player(cx, m, a))
            })
            .command("turns", |c| {
                c.bucket(bucket).exec(|cx, m, _| turns::<C>(cx, m))
            })
            .command("turns2", |c| {
                c.bucket(bucket).exec(|cx, m, _| turns2::<C>(cx, m))
            })
            .command("lobby", |c| {
                c.bucket(bucket).exec(|cx, m, a| lobby(cx, m, a))
            })
            .command("notifications", |c| {
                c.bucket(bucket).exec(|cx, m, a| notifications(cx, m, a))
            })
            .command("start", |c| {
                c.bucket(bucket).exec(|cx, m, a| start::<C>(cx, m, a))
            })
            .command("lobbies", |c| {
                c.bucket(bucket).exec(|cx, m, _| lobbies(cx, m))
            })
            .command("describe", |c| {
                c.bucket(bucket).exec(|cx, m, a| describe(cx, m, a))
            })
            .command("desc", |c| {
                c.bucket(bucket).exec(|cx, m, a| describe(cx, m, a))
            })
    }
}

impl WithServersCommands for StandardFramework {
    fn get_standard_framework(self) -> StandardFramework {
        self
    }
}

fn alias_from_arg_or_channel_name(args: &mut Args, message: &Message) -> Result<String, String> {
    let result_alias = if !args.is_empty() {
        args.single_quoted::<String>().ok()
    } else {
        message.channel_id.name()
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
