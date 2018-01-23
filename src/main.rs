
extern crate byteorder;
extern crate failure;
extern crate flate2;
extern crate hex_slice;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate r2d2_sqlite;
extern crate r2d2;
extern crate rusqlite;
#[macro_use] extern crate serenity;
extern crate simplelog;
extern crate typemap;
extern crate url;
#[macro_use] extern crate enum_primitive_derive;
extern crate num_traits;

mod commands;
mod db;
mod model;
mod server;

use serenity::framework::standard::StandardFramework;
use serenity::model::*;
use serenity::prelude::*;
use simplelog::{Config, LogLevelFilter, SimpleLogger};
use typemap::ShareMap;

use std::{thread, time};
use std::error::Error;
use std::fs::File;
use std::io::Read;

use db::{DbConnection, DbConnectionKey};
use model::{GameServer, GameServerState};
use model::enums::{NationStatus, SubmissionStatus};

struct Handler;
impl EventHandler for Handler {
    fn on_ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

fn main() {
    if let Err(e) = do_main() {
        info!("server crashed with error {}", e)
    }
}

fn do_main() -> Result<(), Box<Error>> {
    SimpleLogger::init(LogLevelFilter::Info, Config::default())?;
    info!("Logger initialised");
    let token = {
        let mut token_file = File::open("resources/token")?;
        let mut temp_token = String::new();
        token_file.read_to_string(&mut temp_token)?;
        info!("Read discord bot token");
        temp_token
    };

    let db_conn = DbConnection::new(&"resources/dom5bot.db".to_string())?;
    info!("Opened database connection");

    let mut discord_client = Client::new(&token, Handler);
    info!("Created discord client");
    {
        let mut data = discord_client.data.lock();
        data.insert::<DbConnectionKey>(db_conn);
    }

    use commands::WithSearchCommands;
    discord_client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .simple_bucket("simple", 1)
        .with_search_commands()
        .command("add", |c| c
            .bucket("simple")
            .exec(|cx, m, a| commands::servers::add_server(cx, m, a))
        )
        .command("list", |c| c
            .bucket("simple")
            .exec(|cx, m, _| commands::servers::list_servers(cx, m))
        )
        .command("delete", |c| c
            .bucket("simple")
            .exec(|cx, m, a| commands::servers::remove_server(cx, m, a))
        )
        .command("details", |c| c
            .bucket("simple")
            .exec(|cx, m, a| commands::servers::details(cx, m, a))
        )
        .command("register", |c| c
            .bucket("simple")
            .exec(|cx, m, a| commands::servers::register_player(cx, m, a))
        )
        .command("unregister", |c| c
            .bucket("simple")
            .exec(|cx, m, a| commands::servers::unregister_player(cx, m, a))
        )
        .command("turns", |c| c
            .bucket("simple")
            .exec(|cx, m, _| commands::servers::turns(cx, m))
        )
        .command("lobby", |c| c
            .bucket("simple")
            .exec(|cx, m, a| commands::servers::lobby(cx, m, a))
        )
        .command("notifications", |c| c
            .bucket("simple")
            .exec(|cx, m, a| commands::servers::notifications(cx, m, a))
        )
        .command("help", |c| c
            .bucket("simple")
            .exec(commands::help))
        .before(|_, msg, _| {
            info!("received message {:?}", msg);
            true
        })
        .after(|_ctx, msg, _cmd_name, result| {
            if let Err(err) = result {
                print!("command error: ");
                let text = format!("ERROR: {}", err.0);
                info!("{}", text);
                let _ = msg.reply(&text);
            }
        })
    );
    info!("Configured discord client");

    let data_clone = discord_client.data.clone();
    thread::spawn(move || {
        check_for_new_turns_every_1_min(data_clone.as_ref());
    });
    // start listening for events by starting a single shard

    if let Err(why) = discord_client.start() {
        error!("Client error: {:?}", why);
    }
    Ok(())
}

fn check_for_new_turns_every_1_min(mutex: &Mutex<ShareMap>) {
    loop {
        thread::sleep(time::Duration::from_secs(60));
        info!("checking for new turns!");
        message_players_if_new_turn(&mutex).unwrap_or_else(|e| {
            error!("Checking for new turns failed with: {}", e);
        });
    }
}

fn message_players_if_new_turn(mutex: &Mutex<ShareMap>) -> Result<(), Box<Error>> {
    let data = mutex.lock();
    let db_conn = data.get::<db::DbConnectionKey>().ok_or("no db connection")?;
    // TODO: transactions
    let servers = db_conn.retrieve_all_servers()?;
    for server in servers {
        let server_name = server.alias.clone();
        if let Err(err) = check_server_for_new_turn(server, &db_conn) {
            println!("error checking {} for turn: {:?}", server_name, err);
        };
    }
    Ok(())
}

fn check_server_for_new_turn(server: GameServer, db_conn: &DbConnection) -> Result<(), Box<Error>> {
    if let GameServerState::StartedState(started_state) = server.state {
        info!("checking {} for new turn", server.alias);
        let game_data = server::get_game_data(&started_state.address)?;
        let new_turn = db_conn.update_game_with_possibly_new_turn(
            &server.alias,
            game_data.turn
        )?; 

        if new_turn {
            info!("new turn in game {}", server.alias);
            for (player, nation_id) in db_conn.players_with_nations_for_game_alias(&server.alias)? {
                if player.turn_notifications {
                    // TODO: quadratic is bad. At least sort it..
                    if let Some(nation) = game_data.nations.iter().find(|&nation| nation.id == nation_id) {
                        if nation.status == NationStatus::Human && nation.submitted == SubmissionStatus::NotSubmitted {
                            use model::enums::Nations;
                            let &(name, era) = Nations::get_nation_desc(nation_id);
                            let text = format!("your nation {} {} has a new turn in {}",
                                era,
                                name,
                                server.alias);
                            info!("Sending DM: {}", text);
                            let private_channel = player.discord_user_id.create_dm_channel()?;
                            private_channel.say(&text)?;
                        }
                    }
                }
            }
        }
    };
    Ok(())
}
