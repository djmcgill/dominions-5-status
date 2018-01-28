mod add_server;
use self::add_server::add_server;

mod list_servers;
use self::list_servers::*;

mod remove_server;
use self::remove_server::*;

mod register_player;
use self::register_player::*;

mod details;
use self::details::*;

mod turns;
use self::turns::*;

mod lobby;
use self::lobby::*;

mod notifications;
use self::notifications::*;

mod turn_check;
pub use self::turn_check::*;

use serenity::framework::standard::StandardFramework;
pub trait WithServersCommands: Sized {
    fn get_standard_framework(self) -> StandardFramework;
    fn with_servers_commands(self) -> StandardFramework { self.get_standard_framework()
        .command("add", |c| c
            .bucket("simple")
            .exec(|cx, m, a| add_server(cx, m, a))
        )
        .command("list", |c| c
            .bucket("simple")
            .exec(|cx, m, _| list_servers(cx, m))
        )
        .command("delete", |c| c
            .bucket("simple")
            .exec(|cx, m, a| remove_server(cx, m, a))
        )
        .command("details", |c| c
            .bucket("simple")
            .exec(|cx, m, a| details(cx, m, a))
        )
        .command("register", |c| c
            .bucket("simple")
            .exec(|cx, m, a| register_player(cx, m, a))
        )
        .command("unregister", |c| c
            .bucket("simple")
            .exec(|cx, m, a| unregister_player(cx, m, a))
        )
        .command("turns", |c| c
            .bucket("simple")
            .exec(|cx, m, _| turns(cx, m))
        )
        .command("lobby", |c| c
            .bucket("simple")
            .exec(|cx, m, a| lobby(cx, m, a))
        )
        .command("notifications", |c| c
            .bucket("simple")
            .exec(|cx, m, a| notifications(cx, m, a))
        )
    }
}

impl WithServersCommands for StandardFramework {
    fn get_standard_framework(self) -> StandardFramework {
        self
    }
}
