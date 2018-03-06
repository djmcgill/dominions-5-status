#![feature(log_syntax, trace_macros)]


extern crate byteorder;
#[macro_use]
extern crate cached;
#[macro_use]
extern crate enum_primitive_derive;
extern crate failure;
extern crate flate2;
extern crate hex_slice;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate num_traits;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;
extern crate serenity;
extern crate simplelog;
extern crate typemap;
extern crate url;
extern crate migrant_lib;

#[cfg_attr(test, macro_use)]
mod db;
mod commands;
mod model;
mod server;

use serenity::framework::standard::StandardFramework;
use serenity::prelude::*;
use simplelog::{Config, LogLevelFilter, SimpleLogger};

use std::thread;
use failure::{SyncFailure, Error};
use std::fs::File;
use std::io::Read;
use std::env;

use db::*;
use server::RealServerConnection;

struct Handler;
impl EventHandler for Handler {}

fn main() {
    if let Err(e) = do_main() {
        info!("server crashed with error {}", e)
    }
}

fn do_main() -> Result<(), Error> {
    SimpleLogger::init(LogLevelFilter::Debug, Config::default())?;
    info!("Logger initialised");

    let mut discord_client = create_discord_client()?;
    if let Err(why) = discord_client.start() {
        error!("Client error: {:?}", why);
    }
    Ok(())
}

fn read_token() -> Result<String, Error> {
    let mut token_file = File::open("resources/token")?;
    let mut temp_token = String::new();
    token_file.read_to_string(&mut temp_token)?;
    info!("Read discord bot token");
    Ok(temp_token)
}

fn create_discord_client() -> Result<Client, Error> {
    let token = read_token()?;

    let path = env::current_dir()?;
    let path = path.join("resources/dom5bot.db");
    let db_conn = DbConnection::new(&path)?;
    info!("Opened database connection");

    let mut discord_client = Client::new(&token, Handler).map_err(SyncFailure::new)?;
    info!("Created discord client");
    {
        let mut data = discord_client.data.lock();
        data.insert::<DbConnectionKey>(db_conn);
    }

    use commands::WithSearchCommands;
    use commands::servers::WithServersCommands;
    discord_client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("!"))
            .simple_bucket("simple", 1)
            .with_search_commands("simple")
            .with_servers_commands::<RealServerConnection>("simple")
            .help(|_, msg, _, _, _| commands::help(msg))
            .before(|_, msg, _| {
                info!("received message {:?}", msg);
                true
            })
            .after(|_ctx, msg, _cmd_name, result| {
                if let Err(err) = result {
                    print!("command error: ");
                    let text = format!("ERROR: {}", err.0);
                    info!("replying with {}", text);
                    let _ = msg.reply(&text);
                }
            }),
    );
    info!("Configured discord client");

    let data_clone = discord_client.data.clone();
    thread::spawn(move || {
        commands::servers::check_for_new_turns_every_1_min::<RealServerConnection>(
            data_clone.as_ref(),
        );
    });
    // start listening for events by starting a single shard
    Ok(discord_client)
}
